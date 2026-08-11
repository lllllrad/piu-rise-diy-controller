# Goals and Non-goals

## Goals

- Provide reliable low-latency press/release input for PUMP IT UP RISE on Windows.
- Support one Launchpad for 5K/6K and two Launchpads for 10K-oriented layouts.
- Include legacy Launchpad support in the MVP because the initial test device is an unidentified older model.
- Run the Windows application elevated so it can inject input into an elevated game process.
- Make all output backends consume the same logical action stream.
- Allow a future RP2030 + FT232RL serial bridge that presents input to the target as USB input hardware.
- Fail safely by releasing active outputs whenever the application still has an opportunity to recover.
- Maintain equivalent English and Korean user documentation.

## Non-goals for the first MVP

- Shipping a custom Windows kernel driver.
- Supporting every Launchpad feature or every Novation product.
- Claiming RISE compatibility without owner-performed real-machine testing.
- Hardware-synchronized LED animation.
- Automatic firmware modification or flashing.

## Open product questions

- Exact model and firmware of the owner's legacy Launchpad.
- Exact key bindings for the owner's RISE setup, especially 10K.
- Whether `RP2030` identifies the intended MCU/module name or a project-specific board.
- Wire protocol and firmware ownership for the RP2030 + FT232RL bridge.
- Packaging and elevation flow: manifest-requested elevation versus an elevated launcher.
