#!/usr/bin/env sh
set -eu

task="${1:?DIM task is required}"
shift

case "$task" in
    bash|codex|claude|bootstrap|doctor|shell|fmt|check|test|docs|verify|check-gui-container) ;;
    *)
        echo "unknown DIM project task: $task" >&2
        echo "available tasks: bash, codex, claude, bootstrap, doctor, shell, fmt, check, test, docs, verify, check-gui-container" >&2
        exit 2
        ;;
esac

exec docker compose --project-name "dim-${DIM_WORKSPACE_NAME}" \
    --file .dim/docker-compose.yml exec \
    --user "$(id -u):$(id -g)" \
    --env HOME=/tmp/dim-agent-home \
    agent sh .dim/agent-entrypoint.sh "$task" "$@"
