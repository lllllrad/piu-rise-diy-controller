# Contributing

This project accepts changes that follow the repository working agreement in [AGENTS.md](AGENTS.md).

## Language

Technical discussion, code, code comments, commit messages, ADRs, and internal documentation are written in English. User documentation must be updated in both English and Korean when user-visible behavior changes.

## Verification labels

Use one of these labels in protocol notes, compatibility tables, and pull request descriptions:

- `Unverified`: derived from documentation, research, or implementation assumptions.
- `Bench verified`: tested with the controller and Windows application, without PUMP IT UP RISE.
- `RISE verified`: tested on the real PUMP IT UP RISE setup by the owner.

Do not upgrade a label without recording the hardware model, firmware when known, application version/commit, and test result.

## Change expectations

- Preserve release semantics for every output backend.
- Add unit tests for parsers, coordinate transforms, mappings, and state machines.
- Keep hardware-specific protocol code out of logical profile code.
- Document user-visible changes in both user-documentation trees.
- Record important or difficult-to-reverse design decisions as ADRs.

## Commits

Prefer focused commits with English imperative messages. Documentation-only scaffolding, protocol implementation, and behavior changes should normally be separate commits.
