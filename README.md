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

- Windows application, run with administrator privileges
- MIDI input from legacy and modern Novation Launchpads
- 5K, 6K, and later 10K logical profiles
- Windows keyboard output first
- Future serial hardware output through RP2030 + FT232RL

No verified release is available yet. Build and test instructions are in the
user guides.

## Development environment

No single environment manager is mandatory. A system Rust installation can be
used directly. The development-only `justfile` provides a compact command
index (`just`, `just build`, `just run`, and `just verify`), while the
checked-in `mise.toml` can install the pinned Rust toolchain and `just` for
environments that use [mise](https://mise.jdx.dev/). Linux-based isolated
workspaces can optionally use
[DIM](https://github.com/slop-lab/dev-infra-manager); see the development
environment guide for its scope and commands.
