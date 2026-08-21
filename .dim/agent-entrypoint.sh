#!/usr/bin/env sh
set -eu

task="${1:?DIM task is required}"
shift

run_tool() {
    if command -v mise >/dev/null 2>&1; then
        exec mise exec -- "$@"
    fi

    exec "$@"
}

require_cargo_project() {
    if [ ! -f Cargo.toml ]; then
        echo "Cargo.toml does not exist." >&2
        exit 2
    fi
}

case "$task" in
    bash)
        exec bash "$@"
        ;;
    codex)
        exec codex --dangerously-bypass-approvals-and-sandbox "$@"
        ;;
    claude)
        exec claude --dangerously-skip-permissions "$@"
        ;;
    bootstrap)
        rustc --version
        cargo --version
        node --version
        just --version
        docker --version
        ;;
    doctor)
        echo "workspace=${DIM_WORKSPACE_NAME:-unknown}"
        echo "backend=${DIM_WORKSPACE_BACKEND:-unknown}"
        rustc --version
        cargo --version
        node --version
        just --version
        docker --version
        docker info --format 'dind_server={{.ServerVersion}}'
        ;;
    shell)
        shell="${SHELL:-/bin/bash}"
        exec "$shell" "$@"
        ;;
    fmt)
        require_cargo_project
        run_tool cargo fmt --all --check "$@"
        ;;
    check)
        require_cargo_project
        run_tool cargo check --all-targets "$@"
        ;;
    test)
        require_cargo_project
        run_tool cargo test --all-targets "$@"
        ;;
    docs)
        exec sh scripts/check-doc-parity.sh "$@"
        ;;
    verify)
        require_cargo_project
        cargo fmt --all --check
        cargo check --all-targets
        cargo clippy --all-targets -- -D warnings
        cargo test --all-targets
        exec sh scripts/check-doc-parity.sh "$@"
        ;;
    check-gui-container)
        exec docker build \
            --file docker/tauri-linux.Dockerfile \
            --tag piu-rise-controller-tauri-check . "$@"
        ;;
    *)
        echo "unknown DIM agent task: $task" >&2
        exit 2
        ;;
esac
