# Verification Status

Last updated: 2026-08-11

## Automated evidence

| Check | Result | Scope |
|---|---|---|
| `cargo fmt --all --check` | Passed | Rust source formatting |
| `cargo test --all-targets --locked` | Passed: 19 tests | Linux build; parsers, config validation/round-trip, profile maps, key/action reference counts, explicit/emergency Release All and partial output failure |
| `cargo clippy --all-targets --locked -- -D warnings` | Passed | Linux targets |
| `sh scripts/check-doc-parity.sh` | Passed | English/Korean user-document paths |
| Generated `original` 10K config | Passed | Schema serialization with 104 device-qualified bindings |
| CLI help and config writer | Passed | Linux command dispatch without MIDI hardware |

The managed Linux execution image lacked a C compiler, ALSA development
metadata, and a normal linker driver. The test run used only temporary files
under `/tmp` to drive the bundled Rust LLD and the installed ALSA runtime. No
temporary validation files are part of the repository or required on normal
developer/CI images.

## Missing evidence

- Windows MSVC compilation and release-manifest embedding: pending CI.
- Windows scan-code `SendInput`, elevation, and console control-handler behavior: Unverified.
- MIDI enumeration and hot-unplug polling with a physical device: Unverified.
- Original/Mk1 or Mk2 address assumptions on the owner's device: Unverified.
- PUMP IT UP RISE menu/gameplay behavior: Unverified; owner test required.
- LED output: not implemented in the passive MVP.
- RP2030 + FT232RL serial output: not implemented.

This file must be updated from actual command output; it is not a release
compatibility claim.
