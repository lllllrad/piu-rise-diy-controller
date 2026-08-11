# Getting Started and Real-game Test

The current build is an engineering MVP. It does not send model-selection
SysEx or LED commands automatically, so identifying an old Launchpad starts
with passive MIDI observation.

## 1. Build on Windows

Install the Visual Studio 2022 C++ Build Tools with a Windows SDK, then install
Rust 1.97.1 directly or run `mise install`. Build from a normal, non-elevated
developer shell:

```powershell
cargo build --release --locked
```

The release executable is `target\release\piu-rise-controller.exe`. Its
embedded manifest requests administrator privileges.

## 2. Identify the MIDI port and protocol family

Connect one Launchpad and run:

```powershell
piu-rise-controller list
piu-rise-controller -vv monitor --input "Launchpad"
```

Press a bottom-left grid pad, a top button, and a right-side button. Save the
output. Typical protocol-family choices are:

- `original`: Original/Mk1 grid addresses `0..119` in rows of 16, with `0`
  at the top-left grid pad.
- `launchpad-s`: the Original/Mk1 address family.
- `mini-legacy`: Mini Mk1/Mk2 legacy address family.
- `mk2`: RGB Launchpad Mk2 grid addresses such as `11..88`.

If observed addresses disagree, do not run live output yet. Generate a config
and edit its `bindings` from the monitor evidence.

An old Launchpad may need a User/standalone layout selected on the device before
it emits the documented grid addresses. The application deliberately does not
change that mode until the exact model is known. Use `monitor` to confirm the
mode after every reconnect.

## 3. Generate and edit configuration

Choose one example:

```powershell
piu-rise-controller write-default-config --model original --profile five-key
piu-rise-controller write-default-config --model mk2 --profile six-key --force
piu-rise-controller write-default-config --model mk2 --profile ten-key --force
```

Run `piu-rise-controller doctor` to print the exact config and log paths. Edit
`device.input_port` so it uniquely matches the MIDI input port. Review every
key and binding before enabling Windows output. For `ten-key`, also set
`device.input_port_right`; bindings with `device = 0` are the left side and
`device = 1` are the right side. The example P2 keys are `Z X C V B` and must
be matched in RISE key settings.

## 4. Dry-run the complete mapping

### Launchpad Mk2 five-key layout

With `--model mk2 --profile five-key`, the physical top grid row is unused.
The eight round buttons above it provide `W`, `S`, `A`, `D`, `Enter`, `Esc`,
`Space`, and `Tab`, from left to right.
The eight round buttons on the right provide `Q`, `E`, `F1`, `F2`, `F3`, `F5`,
`F6`, and `F7`, from bottom to top. Both button groups are illuminated.
The other seven rows contain two red upper 3-by-3 panels, two blue lower
3-by-3 panels, and a yellow 3-row-by-4-column center panel. Shared upper cells
are dark red and shared lower cells are dark blue. Each shared cell activates
both its corner panel and the center action and releases both together.
The application opens the Mk2 MIDI output and
lights this layout when live output starts; set `device.output_port` when the
output port cannot be selected by the input-port substring. LEDs are cleared
on a normal Ctrl+C shutdown.

This MIDI address and palette behavior is `Unverified` until it is checked on
the owner's Launchpad Mk2. `--dry-run` deliberately sends no LED output.

Dry-run receives MIDI and exercises all press/reference/release state without
injecting keys:

```powershell
piu-rise-controller -vv run --input "Launchpad" --model original --profile five-key --dry-run
```

Check taps, holds, two pads belonging to the same logical panel, chords, and
Ctrl+C shutdown. Every press must have a corresponding release in the log.

## 5. Test Windows output away from the game

Open a trusted key-event viewer or text editor, then run the application as
administrator:

```powershell
piu-rise-controller output-test --key F --hold-ms 100
piu-rise-controller run --input "Launchpad" --model original --profile five-key
```

Keep the console accessible. Ctrl+C requests Release All before shutdown. Do
not use Task Manager to terminate the application while a pad is held.

If a previous run ended abnormally, start an elevated console and run:

```powershell
piu-rise-controller release-all
```

## 6. Test with PUMP IT UP RISE

Start with a low-risk menu and low-difficulty chart:

1. Confirm that the application reports `elevated=true` in `doctor`.
2. Confirm menu keys and `Esc` placement before gameplay.
3. Test single taps, then holds, then simultaneous holds.
4. Press Ctrl+C and confirm that no game key remains active.
5. Save the log and exact config used for the test.

The result remains `Unverified` until this process is completed on the real
RISE setup and recorded with the executable version and device model.
