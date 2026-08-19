# Getting Started and Real-game Test

The current build is an engineering MVP. It does not send model-selection
SysEx or LED commands automatically, so identifying an old Launchpad starts
with passive MIDI observation.

## Browser layout-editor preview

Open `web/index.html` directly in a current browser. It requires no install and
shows the same two-device Mk2 surface editor embedded by the Tauri host. Browser
mode validates supported single- and dual-panel assignments and persists the
demo layout in browser local storage. It does not access MIDI, LEDs, or keyboard
output.

The Tauri host is in `src-tauri/`. It repeats validation in Rust and stores
versioned layout JSON in the application configuration directory. Its live
MIDI and keyboard behavior remains `Unverified` until tested on Windows with
the physical Mk2 pair and RISE.

## Run the Windows GUI

From the repository root, build the normal-privilege output helper and start the
normal-privilege Tauri application:

```powershell
just gui-dev
```

Use `just gui-check`, `just gui-build`, or `just gui-release` for the
corresponding non-running workflows. `just web-preview` serves the browser-only
preview at `http://127.0.0.1:8000/`. Direct Cargo commands are fallback paths
when `just` is unavailable.

In a DIM workspace with the External URLs plugin and a configured ingress,
`just web-preview-external INGRESS` starts the browser preview in DIND and
prints the externally reachable URL response. The ingress defaults to
`local-http`. Run `just web-preview-external-stop` to stop its container.

The GUI discovers and opens the two MIDI inputs itself; it does not invoke the
CLI executable. The keyboard output helper also runs with normal privileges,
so starting the controller does not display UAC. Stopping, output failure, and applying a new layout request
Release All. Select a surface control and use the arrow keys to move,
`Ctrl+C`/`Ctrl+V` to copy and paste its assignment, and `Delete` to clear it.

## 1. Build on Windows

Install the Visual Studio 2022 C++ Build Tools with a Windows SDK, then install
Rust 1.97.1 directly or run `mise install`. Build from a normal, non-elevated
developer shell:

```powershell
cargo build --release --locked
```

The release executable is `target\release\piu-rise-controller.exe`. It runs
with the current user's normal privileges and does not request UAC elevation.

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
piu-rise-controller write-default-config --model original
piu-rise-controller write-default-config --model mk2 --force
piu-rise-controller write-default-config --model mk2 --two-devices --force
```

Run `piu-rise-controller doctor` to print the exact config and log paths. Edit
`device.input_port` so it uniquely matches the MIDI input port. Review every
key and binding before enabling Windows output. For the two-device layout, also set
`device.input_port_right`; bindings with `device = 0` are the left side and
`device = 1` are the right side. The default left-side P2 keys are `Z Q S E C`.
The non-overlapping right-side P1 keys use the matching letter-key shape:
`V R G Y N`. Match both sets in RISE key settings.

## 4. Dry-run the complete mapping

### Launchpad Mk2 five-key layout

With one configured input and `--model mk2`, the physical top grid row is unused.
The eight round buttons above it provide `W`, `S`, `A`, `D`, `Enter`, `Esc`,
`Space`, and `Tab`, from left to right.
The Mk2 right-side buttons are not used. In a two-device 6K or 10K setup, the
right upright device's top buttons provide the primary UI keys. The left device
is rotated 90 degrees counter-clockwise; its original top buttons (physically
on the left after rotation) provide `Q`, `E`, `F1`, `F2`, `F3`, `F5`, `F6`, and
`F7`.
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

### Two-device 10-panel layout

Pass the upright right/main device with `--input`/`--input-index` and the
counter-clockwise-rotated additional device with `--input-left`/
`--input-left-index`.
Providing a second input always selects the 10-panel layout; it may also be
used for RISE 6K by editing the ten output key assignments. The left device
uses the full P2 five-panel layout and the right uses the full P1 layout.
A clockwise coordinate compensation is applied to both input and LED addresses
for the physically counter-clockwise left device.

When identical devices have identical names, use the indices printed by
`list`. Main and left LED outputs can likewise be selected with
`--output-index` and `--output-left-index`. Indices can change after reconnecting USB, so inspect
`list` before launch.

Dry-run receives MIDI and exercises all press/reference/release state without
injecting keys:

```powershell
piu-rise-controller -vv run --input "Launchpad" --model original --dry-run
```

Check taps, holds, two pads belonging to the same logical panel, chords, and
Ctrl+C shutdown. Every press must have a corresponding release in the log.

## 5. Test Windows output away from the game

Open a trusted key-event viewer or text editor, then run the application from
a normal, non-elevated console:

```powershell
piu-rise-controller output-test --key F --hold-ms 100
piu-rise-controller run --input "Launchpad" --model original
```

Keep the console accessible. Ctrl+C requests Release All before shutdown. Do
not use Task Manager to terminate the application while a pad is held.

If a previous run ended abnormally, run:

```powershell
piu-rise-controller release-all
```

## 6. Test with PUMP IT UP RISE

Start with a low-risk menu and low-difficulty chart:

1. Confirm that RISE is also running non-elevated. Windows blocks `SendInput`
   into a process running at a higher integrity level.
2. Confirm menu keys and `Esc` placement before gameplay.
3. Test single taps, then holds, then simultaneous holds.
4. Press Ctrl+C and confirm that no game key remains active.
5. Save the log and exact config used for the test.

The result remains `Unverified` until this process is completed on the real
RISE setup and recorded with the executable version and device model.
