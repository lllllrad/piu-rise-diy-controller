# PIU RISE DIY Controller

A Windows-only Rust controller application for using one or two Novation Launchpads with PUMP IT UP RISE. Legacy Launchpad support is part of the MVP, and the architecture reserves an output path for a future RP2030 + FT232RL hardware bridge.

This repository contains an unverified engineering MVP. Compatibility claims
are separated into desktop/bench verification and verification on a real PUMP
IT UP RISE machine.

## Documentation

- [English user documentation](docs/user/en/README.md)
- [한국어 사용자 문서](docs/user/ko/README.md)
- [Internal technical documentation](docs/internal/README.md)
- [Development environments](docs/internal/DEVELOPMENT.md)
- [Project roadmap](docs/internal/ROADMAP.md)
- [Contributing](CONTRIBUTING.md)

## Current scope

- Windows application with normal, non-elevated execution
- MIDI input from legacy and modern Novation Launchpads
- 5K, 6K, and later 10K logical profiles
- Windows keyboard output first
- Future serial hardware output through RP2030 + FT232RL

No verified release is available yet. Build and test instructions are in the
user guides.

## Layout editor preview

Open `web/index.html` in a current browser to inspect the shared layout editor
without installing anything. Browser mode uses a demo backend and never sends
MIDI or keyboard output.

The native host is an independent Tauri crate. On Windows, build its elevated
keyboard-output helper and run the normal-privilege GUI with:

```text
just gui-dev
```

The GUI owns MIDI discovery, live mapping, layout changes, and persistent
layout state directly; it does not invoke the CLI executable. Physical GUI and
RISE behavior remains unverified.

On Linux, keep WebKitGTK, D-Bus, and the other Tauri build dependencies out of
the host by running `just check-gui-container`. The required system packages
and Rust crates remain in Docker-managed image and cache layers.

Other common entry points are `just gui-check`, `just gui-build`,
`just gui-release`, and `just web-preview`. Run `just --list` for the complete
command index. Direct Cargo commands remain supported as a fallback.

## Development environment

No single environment manager is mandatory. A system Rust installation can be
used directly. The development-only `justfile` provides a compact command
index (`just`, `just build`, `just run`, and `just verify`), while the
checked-in `mise.toml` can install the pinned Rust toolchain and `just` for
environments that use [mise](https://mise.jdx.dev/). Linux-based isolated
workspaces can optionally use
[DIM](https://github.com/slop-lab/dev-infra-manager); see the development
environment guide for its scope and commands.
