#!/usr/bin/env sh
set -eu

git_name="$(dim-host-input builtin.git-author name)"
git_email="$(dim-host-input builtin.git-author email)"

export GIT_AUTHOR_NAME="$git_name"
export GIT_AUTHOR_EMAIL="$git_email"
export GIT_COMMITTER_NAME="$git_name"
export GIT_COMMITTER_EMAIL="$git_email"

origin_url="$(git remote get-url origin)"
origin_repository="$(basename "$origin_url")"
git remote set-url origin "${DIM_GIT_BASE_URL:?DIM_GIT_BASE_URL is required}/$origin_repository"

proxy_dir="/run/dim/dev-controller"
proxy_socket="$proxy_dir/controller.sock"
proxy_log="$proxy_dir/external-url.log"
external_url_ingress="${DIM_EXTERNAL_URL_INGRESS:-local-http}"

if ! curl --fail --silent --unix-socket "$proxy_socket" \
    http://dim-controller/api/urls >/dev/null 2>&1; then
    sudo install --directory \
        --owner "$(id -u)" --group "$(id -g)" --mode 0755 "$proxy_dir"
    rm -f "$proxy_socket"
    dim-controller-proxy external-url \
        --listen "$proxy_socket" \
        --directory-mode 0755 \
        --socket-mode 0666 \
        --ingress "$external_url_ingress" >"$proxy_log" 2>&1 &
    for attempt in $(seq 1 30); do
        [ -S "$proxy_socket" ] && break
        if [ "$attempt" -eq 30 ]; then
            cat "$proxy_log" >&2
            exit 1
        fi
        sleep 1
    done
fi

compose() {
    docker compose --project-name "dim-${DIM_WORKSPACE_NAME}" \
        --file .dim/docker-compose.yml "$@"
}

# An outer workspace stop terminates nested containers without letting their
# daemon preserve a restartable process state. Recreate Project containers on
# every setup while retaining their named data and home volumes.
compose build agent agent-dind
compose up --detach --force-recreate agent-dind agent
compose exec --no-TTY agent \
    chown -R "$(id -u):$(id -g)" /home/dim-agent
