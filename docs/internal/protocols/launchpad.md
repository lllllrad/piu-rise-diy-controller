# Launchpad Protocol Strategy

## Problem statement

The first physical device is an older Launchpad whose exact model is unknown. Legacy products do not share one LED protocol:

- Original Launchpad/Mk1 and Launchpad S use legacy palette/bi-color behavior.
- Launchpad Mini Mk1/Mk2 have limited colors and are not equivalent to Launchpad Mk2.
- Launchpad Mk2 supports RGB through a legacy protocol distinct from modern devices.
- Launchpad X, Mini Mk3, and Pro Mk3 use the modern Programmer Mode family with model capabilities.

All entries remain `Unverified` until tested with the corresponding device.

## Safe identification workflow

1. Ask the user for the underside product name and, if available, serial/firmware information.
2. Record Windows MIDI input and output port names and USB VID/PID where accessible.
3. Pair ports explicitly and let the user confirm identity by pressing a pad.
4. Observe incoming Note/CC messages before sending model-specific SysEx.
5. Send only documented, reversible probes for candidate models.
6. Persist a stable device binding plus a user-assigned role; never depend only on enumeration order.

## Adapter families

```text
LaunchpadAdapter
  LegacyOriginalAdapter
  LegacyMiniAdapter
  LegacyMk2RgbAdapter
  ModernProgrammerAdapter
    capabilities(model)
```

The exact split may change after hardware identification, but logical mappings must remain independent of it.

## Implemented address assumptions

The application sends no automatic layout-selection SysEx. The user selects a
usable device mode and confirms addresses with `monitor` before live output.
For an explicitly selected `mk2`, live mode sends palette-index Note On
messages to the 64 grid LEDs. This does not run in the input dispatch path.

| Adapter choice | Grid assumption | Primary UI | Secondary UI | Status |
|---|---|---|---|---|
| `original` / `launchpad-s` / `mini-legacy` | Top-left is Note 0; rows advance by 16 | CC 104..111 | Scene Notes 8, 24, ..., 120 | Unverified |
| `mk2` | Bottom-left is Note 11; rows advance by 10 | Top CC 104..111; grid Notes 81..88 unused | Right CC 19, 29, ..., 89 | Unverified |
| `modern` | Bottom-left is Note 11; rows advance by 10 | CC 91..98 | Right CC 19, 29, ..., 89 | Unverified |

Five-key uses a spatial panel layout. Six-key uses vertical lanes. Ten-key
assigns device 0 to P1 and device 1 to P2 and routes both through one shared
output reference-count state. Configuration bindings can replace every
default assumption.

Original Launchpad coordinates are based on the official X-Y mapping. Mk2 and
modern defaults remain bench-unverified in this repository even where their
address maps come from official documents.

The Mk2 five-key grid leaves Notes 81..88 unused and assigns the eight primary
UI actions to the round top buttons at CC 104..111. The remaining rows use
3-by-3 corner regions and a 3-row-by-4-column center region. Each of the four
overlap controls maps to both its corner action and the center action. The LED
uses a darker shade of that corner's red or blue rather than a mixed color.
Mk2 palette Note On and CC output is cleared on normal
shutdown. Runtime pressed-state animation is not implemented.

For the supported two-device setup, device 0 is the physical left Launchpad
rotated 90 degrees counter-clockwise and device 1 is the upright right/main
Launchpad. A clockwise compensation transform is applied to device 0 grid input
and LED addresses. Mk2 side CCs are unused. Device 1 top CC 104..111 owns
the primary UI actions, while device 0 top CC 104..111 owns the secondary UI
actions and appears on the physical left edge after rotation.

Runtime layout selection is inferred from device count rather than a CLI
profile: one input uses the five-panel layout, while two inputs use all five
spatial panels on each device. The ten-panel layout can be assigned to RISE 6K
through output-key configuration without changing the physical layout.

## Required normalized events

```text
ControlPressed(device_id, control_id)
ControlReleased(device_id, control_id)
DeviceConnected(device_id, capabilities)
DeviceDisconnected(device_id)
```

Both MIDI Note Off and Note On with velocity zero are release candidates. Model documentation and bench traces determine channel/address interpretation.

## Evidence record template

For every protocol behavior promoted from research, record:

- model and firmware if known;
- authoritative document or captured trace;
- exact request/input and response/output bytes;
- verification label;
- date and application commit;
- deviations from published documentation.

The Korean research documents under `.local/initial-prompts-and-references/` are useful leads but are not themselves implementation contracts.
