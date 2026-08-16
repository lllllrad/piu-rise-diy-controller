#!/usr/bin/env sh
set -eu

git_name="$(dim-host-input builtin.git-author name)"
git_email="$(dim-host-input builtin.git-author email)"

export GIT_AUTHOR_NAME="$git_name"
export GIT_AUTHOR_EMAIL="$git_email"
export GIT_COMMITTER_NAME="$git_name"
export GIT_COMMITTER_EMAIL="$git_email"

proxy_dir="/tmp/dim-controller-proxy"
proxy_socket="$proxy_dir/external-url.sock"
proxy_log="$proxy_dir/external-url.log"

if discovery="$(dim external-url discover --json 2>/dev/null)" && \
    ingresses="$(printf '%s' "$discovery" | node -e '
let input = "";
process.stdin.on("data", chunk => input += chunk);
process.stdin.on("end", () => {
  for (const ingress of JSON.parse(input)) console.log(ingress.name);
});
')" && [ -n "$ingresses" ]; then
    if ! curl --fail --silent --unix-socket "$proxy_socket" \
        http://dim-controller/api/urls >/dev/null 2>&1; then
        mkdir -p "$proxy_dir"
        rm -f "$proxy_socket"
        set --
        for ingress in $ingresses; do
            set -- "$@" --ingress "$ingress"
        done
        dim-controller-proxy external-url \
            --listen "$proxy_socket" \
            --directory-mode 0755 \
            --socket-mode 0666 \
            "$@" >"$proxy_log" 2>&1 &
        for attempt in $(seq 1 30); do
            [ -S "$proxy_socket" ] && break
            if [ "$attempt" -eq 30 ]; then
                cat "$proxy_log" >&2
                exit 1
            fi
            sleep 1
        done
    fi
fi

docker compose --project-name "dim-${DIM_WORKSPACE_NAME}" \
    --file .dim/docker-compose.yml up --detach --build agent
