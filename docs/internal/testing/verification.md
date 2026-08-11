# Verification Strategy

## Test layers

### Unit tests

- MIDI parsing, including malformed/truncated messages.
- Model address maps and coordinate rotations.
- Physical-to-logical mapping.
- Reference-counted press/release transitions.
- Profile changes and Release All ordering.
- Serial framing and checksums when the hardware backend is introduced.

### Property and state-machine tests

- No event sequence leaves an output pressed after Release All.
- Duplicate releases do not underflow reference counts.
- Device loss and profile switches are idempotent.
- LED failures cannot alter output state.

### Windows integration tests

- Elevated launch and configuration access.
- Key down/up and simultaneous-key observation in a dedicated test receiver.
- MIDI hot-plug, reconnect, and port-pairing behavior.
- Clean shutdown, recoverable failure, and panic cleanup where execution continues long enough to clean up.

OS integration tests cannot prove cleanup after unconditional process termination or power loss. This limitation must remain documented.

### Hardware bench tests

- Identify exact Launchpad model and firmware.
- Capture raw input for every surface control.
- Validate LED commands, palette limitations, and restoration behavior.
- Measure practical input-to-output latency and verify holds/chords.

### RISE tests

Only the owner can perform these on the real machine. Each test record should include:

- application commit/version;
- Windows version and privilege state;
- controller model/firmware and device role;
- profile and mapping config hash;
- output backend;
- menu, tap, hold, chord, disconnect, and recovery results.

## Compatibility claims

CI success permits no stronger claim than software verification. Hardware bench success permits `Bench verified`. Only a recorded real-machine result permits `RISE verified`.
