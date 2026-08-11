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
