# System Architecture

## Data flow

```text
MIDI ports
  -> model-specific Launchpad adapter
  -> normalized physical controls
  -> device role and coordinate transform
  -> logical action/profile state machine
  -> output-state coordinator
       -> Windows keyboard backend
       -> future serial hardware backend

Logical/physical state
  -> asynchronous LED renderer
  -> model-specific Launchpad adapter
  -> MIDI output
```

## Component boundaries

### Device surfaces and editable layouts

Persistent layouts address controls by a stable, model-specific `ControlId`,
not by MIDI note, enumeration index, or an assumed rectangular grid. A device
adapter supplies optional geometry for GUI rendering and the protocol address
used when the validated layout is compiled. This supports the Mk2 gameplay
surface currently confirmed as 72 usable controls (the 8-by-8 pad matrix plus
eight right-edge controls) and future non-grid devices.

A physical control maps to one logical action or one explicitly allowed
two-action set. Allowed pairs are the four diagonal-plus-center overlaps on
each side and the upper/lower pairs across the device boundary. All other
multi-action bindings are rejected before active output state is changed.

### Device adapters

Decode MIDI and encode device LEDs. They own model capabilities and protocol details, but do not know Windows keys or RISE profiles.

Legacy families must be separate from modern SysEx families. A capability model is preferred over growing model checks throughout the application.

### Logical action layer

Defines actions such as `PLAY_5K.CENTER`, `UI.CONFIRM`, and `CONTROLLER.RELEASE_ALL`. Profiles map normalized controls to actions. Multiple controls mapping to one action share state safely.

### Output-state coordinator

Owns the set of active logical outputs and performs edge-triggered press/release calls. On profile changes, device loss, recoverable errors, and shutdown it releases all outputs before resetting input state.

### Output backends

The initial backend uses Windows `SendInput` keyboard injection. Configured
virtual-key names are converted to scan codes, including extended-key flags,
so gameplay input resembles physical key transitions and is independent of the
active keyboard layout. It is isolated behind a trait so the future serial
bridge can transmit the same logical output transitions.

The serial backend must eventually include protocol version negotiation, sequence/framing protection, heartbeat timeout, and an explicit Release All command. FT232RL is a transport detail; USB input presentation is owned by downstream firmware/hardware.

### LED renderer

Consumes state snapshots asynchronously. LED failures and latency must never block input dispatch. Rendering is limited by the connected model's palette, address space, and SysEx capability.

## Concurrency invariants

- Each physical press has at most one matching active physical state entry.
- A logical output is pressed on reference transition `0 -> 1` and released on `1 -> 0`.
- Output state is serialized through one owner even if MIDI callbacks are concurrent.
- LED work does not run in the latency-sensitive output transition path.
- Device disconnect clears only after active output release has been requested.

## Privilege boundary

The distributed Windows application runs at the invoking user's integrity level and does not request administrator elevation. Windows `SendInput` cannot target a process at a higher integrity level, so RISE and the controller should normally run non-elevated together. MIDI and configuration inputs must still be treated as untrusted data.
