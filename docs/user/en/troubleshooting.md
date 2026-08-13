# Troubleshooting and Diagnostic Logs

## Diagnostic command

Run this before reporting a problem:

```powershell
piu-rise-controller -vv doctor
```

It reports application version, operating system, administrator state,
configuration path, log path, configuration schema, and MIDI ports. It does
not record unrelated keyboard input.

## Logs

The application writes daily files under the path printed as `logs=` by
`doctor`. Normal files contain lifecycle, mapping, output-transition, and error
details. `-vv monitor` also prints raw MIDI bytes needed to identify old
devices. Review files before sharing because MIDI port names may contain a
device or account-specific suffix.

## No MIDI port or ambiguous selector

- Disconnect other MIDI devices and run `list` again.
- Use a longer unique substring with `--input`.
- Select the MIDI port rather than a DAW port when both exist.

## A key remains pressed

1. Release every physical pad.
2. Press Ctrl+C in the controller console.
3. If the process is gone, run `piu-rise-controller release-all`. Press and
   release the affected physical keyboard key
   once if Windows still reports it active.
4. Save the log and note whether shutdown, disconnect, or forced termination
   occurred.

No application can send Release All after unconditional process termination or
power loss. Avoid forced termination while controls are active.

## RISE does not receive keys

- Confirm RISE and the controller run at the same integrity level; normally
  both should be non-elevated.
- Confirm `output-test` in another non-elevated application.
- Check the configured RISE key bindings.
- Verify the controller is not still running with `--dry-run`.
- Save both controller logs and the exact config file.
