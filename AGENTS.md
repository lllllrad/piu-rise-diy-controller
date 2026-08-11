# Repository Working Agreement

## Project purpose

Build a Windows-only Rust application that turns one or two Novation Launchpads into a dedicated controller for PUMP IT UP RISE. The design must also allow a future hardware output path using an RP2030 and FT232RL so the PC can communicate with a device that presents USB input to the game machine.

## Current product constraints

- The Windows application is expected to run as administrator.
- The MVP must support the owner's older Launchpad even if it is an Original/Mk1, Launchpad S, Mini Mk1/Mk2, or Launchpad Mk2. Do not assume modern RGB SysEx is available.
- Keep modern Launchpad X, Mini Mk3, and Pro Mk3 support in the architecture, but do not let it replace legacy MVP support.
- Actual PUMP IT UP RISE integration can only be verified by the owner on the real machine. Automated tests must validate everything up to the OS/device boundary.
- Keyboard injection is the first Windows output path. A serial hardware bridge and other output backends must remain possible without rewriting input mapping.

## Architecture rules

- Keep device input, logical actions, mapping/profile state, and output backends separate.
- Never map raw MIDI messages directly to Windows keys in device code.
- Represent press and release as persistent state, not timed pulses.
- Use reference counting or an equivalent invariant when multiple physical controls map to one logical action.
- Releasing all active outputs is mandatory on device loss, profile change, recoverable failure, and normal shutdown.
- Input dispatch must not wait for LED rendering or slow device I/O.
- Model-specific MIDI/SysEx behavior belongs behind capability-based device adapters.
- Do not identify two identical devices solely by enumeration order.

## Documentation policy

- User-facing documentation is maintained in English under `docs/user/en/` and Korean under `docs/user/ko/`.
- Paired user documents must use the same relative filename and equivalent structure.
- Internal technical documentation, ADRs, protocol notes, code comments, identifiers, and contributor instructions are English only.
- Stable identifiers such as config keys, profile IDs, CLI flags, log fields, and error codes are not translated.
- Research notes in `.local/initial-prompts-and-references/` are leads, not normative specifications. Promote verified facts into `docs/internal/` and record provenance.
- If a behavior has not been tested on real hardware or RISE, label it `Unverified`, `Bench verified`, or `RISE verified` as appropriate.

## Development workflow

- Keep commits small enough to review and revert independently.
- Commit messages and branch names are English.
- Do not mix mechanical formatting, documentation translation, and behavior changes in one commit unless inseparable.
- Before committing, run the relevant formatter, linter, unit tests, and documentation checks available in the repository.
- Never claim RISE compatibility from simulated or desktop-only testing.
- Preserve user changes and unrelated untracked files.

## Rust and Windows conventions

- Pin the Rust toolchain used by CI and releases; do not rely on `latest` for reproducible builds.
- Isolate unsafe Windows FFI and document its safety invariants.
- Log enough device identity and event-state information to diagnose field failures without recording unrelated user input.
- Treat privilege elevation, driver installation, and firmware flashing as explicit user-visible operations.
- Prefer configuration schema evolution with versioning and migrations over ad hoc config changes.

## Definition of done

A change is done when its relevant tests pass, failure/release behavior is covered, user-facing behavior is documented in both languages, internal design changes are documented in English, and its verification level is stated honestly.
