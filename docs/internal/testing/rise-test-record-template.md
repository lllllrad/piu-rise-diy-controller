# RISE Test Record: YYYY-MM-DD / Device

- Verification result: Unverified
- Tester:
- Application commit/version:
- Executable source: local build / CI artifact
- Windows version:
- `doctor` elevated value:
- Launchpad product label:
- Launchpad firmware, if known:
- Left MIDI port:
- Right MIDI port, if used:
- Layout: one-device five-panel / two-device ten-panel
- Config SHA-256:
- Log filename:
- RISE station/mode and version:

## Passive identification

- [ ] Bottom-left, top-left, center, top button, and side button raw MIDI captured.
- [ ] Selected model address family agrees with the captured messages.
- [ ] Dry-run produces one press and release per logical tap.
- [ ] Two cells in one panel preserve the action until both are released.
- [ ] Ctrl+C clears every dry-run output.

## Windows output outside RISE

- [ ] `output-test` produces one down/up transition.
- [ ] Multiple simultaneous actions are visible.
- [ ] `release-all` clears configured keys.
- [ ] USB disconnect while holding input clears output within approximately one second.

## RISE result

- [ ] Menu navigation
- [ ] Confirm and Back/Esc safety
- [ ] Single taps
- [ ] Holds
- [ ] Simultaneous holds/chords
- [ ] Profile-specific gameplay
- [ ] Clean Ctrl+C shutdown
- [ ] Disconnect recovery

## Observations

Record missed input, stuck input, unexpected repeats, address differences,
latency observations, and reproduction steps.

## Evidence and conclusion

Attach or reference the config, controller log, raw MIDI sample, and any video.
Change the result to `Bench verified` or `RISE verified` only when the required
scope actually passed.
