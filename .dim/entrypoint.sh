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
        echo "Cargo.toml does not exist yet; the repository is still in its documentation stage." >&2
        exit 2
    fi
}

case "$task" in
    bootstrap)
        if ! command -v mise >/dev/null 2>&1; then
            echo "mise is required for the bootstrap task: https://mise.jdx.dev/" >&2
            exit 2
        fi
        exec mise install
        ;;
    doctor)
        echo "workspace=${DIM_WORKSPACE_NAME:-unknown}"
        echo "backend=${DIM_WORKSPACE_BACKEND:-unknown}"
        if command -v mise >/dev/null 2>&1; then
            mise --version
            mise current || true
        else
            echo "mise=not-installed"
        fi
        if command -v rustc >/dev/null 2>&1; then
            rustc --version
        else
            echo "rustc=not-installed"
        fi
        if command -v cargo >/dev/null 2>&1; then
            cargo --version
        else
            echo "cargo=not-installed"
        fi
        ;;
    shell)
        shell="${SHELL:-/bin/sh}"
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
        if command -v mise >/dev/null 2>&1; then
            exec mise run verify "$@"
        fi
        echo "The verify task requires mise; run fmt, check, and test separately otherwise." >&2
        exit 2
        ;;
    *)
        echo "unknown DIM project task: $task" >&2
        echo "available tasks: bootstrap, doctor, shell, fmt, check, test, docs, verify" >&2
        exit 2
        ;;
esac
