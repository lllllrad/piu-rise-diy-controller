# Compatibility

Hardware and game compatibility are not yet verified. The table distinguishes planned implementation from bench and actual-hardware testing.

| Device or environment | Planned support | Verification |
|---|---:|---|
| Original Launchpad / Mk1 | MVP | Unverified |
| Launchpad S | MVP fallback | Unverified |
| Launchpad Mini Mk1/Mk2 | MVP fallback | Unverified |
| Launchpad Mk2 | MVP | Unverified |
| Launchpad X | Planned | Unverified |
| Launchpad Mini Mk3 | Planned | Unverified |
| Launchpad Pro Mk3 | Planned | Unverified |
| Windows keyboard output | MVP | Bench verified in QEMU Windows PE; desktop/game focus remains Unverified |
| PUMP IT UP RISE | Target | Unverified; real-machine testing required |
| RP2030 + FT232RL bridge | Future | Not implemented |

The owner's legacy Launchpad model has not yet been identified. First-run diagnostics will therefore need to report MIDI port names and observed protocol behavior without sending unsafe model-specific commands blindly.
