# PIU RISE DIY Controller

A Windows-only Rust controller application for using one or two Novation Launchpads with PUMP IT UP RISE. Legacy Launchpad support is part of the MVP, and the architecture reserves an output path for a future RP2030 + FT232RL hardware bridge.

This repository is at the design and scaffolding stage. Compatibility claims will be separated into desktop/bench verification and verification on a real PUMP IT UP RISE machine.

## Documentation

- [English user documentation](docs/user/en/README.md)
- [한국어 사용자 문서](docs/user/ko/README.md)
- [Internal technical documentation](docs/internal/README.md)
- [Project roadmap](docs/internal/ROADMAP.md)
- [Contributing](CONTRIBUTING.md)

## Current scope

- Windows application, run with administrator privileges
- MIDI input from legacy and modern Novation Launchpads
- 5K, 6K, and later 10K logical profiles
- Windows keyboard output first
- Future serial hardware output through RP2030 + FT232RL

No release is available yet.
