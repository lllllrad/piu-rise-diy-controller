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
| Windows MSVC release cross-build | Passed | Rust 1.97.1, `cargo-xwin` 0.23.0, static CRT, embedded `requireAdministrator` manifest |
| KVM/QEMU Windows boundary run | Passed: 19 tests | QEMU 10.2.1 and Windows 11 Enterprise Evaluation 25H2 WinPE; elevated CLI, MIDI enumeration with no attached device, Original 5K config write/reload, Release All, and `SendInput` press/release |

The managed Linux execution image lacked a C compiler, ALSA development
metadata, and a normal linker driver. The test run used only temporary files
under `/tmp` to drive the bundled Rust LLD and the installed ALSA runtime. No
temporary validation files are part of the repository or required on normal
developer/CI images.

The Windows boundary run used the official English x64 evaluation ISO with
SHA-256 `a61adeab895ef5a4db436e0a7011c92a2ff17bb0357f58b13bbc4062e535e7b9`.
The release executable ran as `SYSTEM` and reported `os=windows` and
`elevated=true`. It generated an `original` profile with 52 bindings, released
25 configured keys, completed a 25 ms `F` key output test, and passed 18 library
plus 1 binary test. This is `Bench verified` evidence for the Windows API
boundary only. WinPE is not a full desktop session and the run included no MIDI
hardware, Launchpad, or game.

## Missing evidence

- Native Windows MSVC compilation in GitHub Actions: pending CI; the equivalent
  release was cross-built and executed in Windows PE.
- Windows scan-code `SendInput` and elevation: Bench verified in QEMU Windows
  PE. Desktop focus behavior, UAC prompting, and the console control handler
  remain Unverified.
- MIDI enumeration and hot-unplug polling with a physical device: Unverified.
- Original/Mk1 or Mk2 address assumptions on the owner's device: Unverified.
- PUMP IT UP RISE menu/gameplay behavior: Unverified; owner test required.
- LED output: not implemented in the passive MVP.
- RP2030 + FT232RL serial output: not implemented.

This file must be updated from actual command output; it is not a release
compatibility claim.
