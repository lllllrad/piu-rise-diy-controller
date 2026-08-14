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

# Backward-compatible name for the full GUI development launcher.
run-gui: gui-dev

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

# Build the elevated output helper, then run the normal-privilege Tauri GUI.
gui-dev:
    cargo build --locked --bin piu-rise-output-helper
    cargo run --locked --manifest-path src-tauri/Cargo.toml

# Check the native GUI and its output helper without running them.
gui-check:
    cargo check --locked --bin piu-rise-output-helper
    cargo check --locked --manifest-path src-tauri/Cargo.toml

# Build debug GUI and helper executables.
gui-build:
    cargo build --locked --bin piu-rise-output-helper
    cargo build --locked --manifest-path src-tauri/Cargo.toml

# Build optimized GUI and helper executables.
gui-release:
    cargo build --release --locked --bin piu-rise-output-helper
    cargo build --release --locked --manifest-path src-tauri/Cargo.toml

# Serve the install-free browser preview at http://127.0.0.1:8000/.
web-preview port="8000":
    node scripts/serve-web.mjs {{port}}
