# Development command shortcuts. `just` is not required to build or run a release.

[private]
default:
    @just --list

# Build a debug executable.
build:
    cargo build --locked

# Build an optimized release executable.
release:
    cargo build --release --locked

# Run the development executable; defaults to CLI help.
run command="--help" *args:
    cargo run --locked -- {{command}} {{args}}

# Format Rust source files.
fmt:
    cargo fmt --all

# Run all unit tests.
test:
    cargo test --all-targets --locked

# Run formatting, compiler, linter, test, and documentation checks.
verify:
    cargo fmt --all --check
    cargo check --all-targets --locked
    cargo clippy --all-targets --locked -- -D warnings
    cargo test --all-targets --locked
    sh scripts/check-doc-parity.sh

# Check the Tauri host without installing Linux GUI dependencies on the host.
check-gui-container:
    docker build --file docker/tauri-linux.Dockerfile --tag piu-rise-controller-tauri-check .
