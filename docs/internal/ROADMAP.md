# Roadmap

This roadmap describes ordering, not release dates.

## Phase 0: Repository and evidence foundation

- Establish documentation, ADR, verification, and compatibility structures.
- Identify the physical legacy Launchpad from labels, USB IDs, port names, and safe MIDI observations.
- Pin the Rust toolchain and establish Windows CI.
- Define configuration schema versioning and diagnostic logging.

Exit condition: the device model is known or a safe capability probe can distinguish the supported legacy families.

## Phase 1: Legacy Launchpad MVP

- MIDI port discovery and explicit input/output pairing.
- Legacy adapters for Original/Mk1-family behavior and Launchpad Mk2 RGB behavior as required by the identified device.
- Logical 5K and 6K profiles.
- Reference-counted press state and Release All.
- Non-elevated Windows keyboard output.
- Minimal status/pressed LEDs within device capabilities.
- Headless diagnostic mode and event trace suitable for remote debugging.

Exit condition: bench-verified press/release behavior, disconnect recovery, profile switching, and simultaneous input on the owner's device.

Implementation status: the passive monitor, legacy address-family defaults,
5K/6K/10K mappings, shared reference-counted key state, Windows SendInput
backend, non-elevated manifest, disconnect polling, daily logs, and dry-run CLI
exist but remain `Unverified` until Windows CI and owner hardware tests pass.

## Phase 2: Real RISE validation

- Owner tests installation, integrity-level matching, menus, gameplay holds, simultaneous inputs, and recovery on the real machine.
- Record results as `RISE verified` per model/profile/output backend.
- Adjust default mappings without embedding game keys in device adapters.

Exit condition: a documented, repeatable RISE-verified configuration exists.

## Phase 3: Two-device and modern support

- Stable LEFT/RIGHT assignment and identification UX.
- 10K logical profile and two-device state handling.
- Launchpad X, Mini Mk3, and Pro Mk3 capability-based adapters.
- RGB batching and LED shadow framebuffer.

## Phase 4: Hardware output bridge

- Specify a framed, versioned, checksummed serial protocol.
- Implement the RP2030 + FT232RL output backend without changing logical profiles.
- Define heartbeat, timeout, Release All, reconnect, and firmware compatibility behavior.
- Validate how the downstream hardware presents USB keyboard/gamepad input.

## Later possibilities

- Alternative HID/gamepad output backends.
- Additional physical controllers.
- Configuration UI and profile editor.
- Signed installer and automatic updates.
