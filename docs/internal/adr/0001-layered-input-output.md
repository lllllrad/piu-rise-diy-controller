# ADR-0001: Separate Logical Actions from Device and Output Backends

- Status: Accepted
- Date: 2026-08-11

## Context

The MVP receives MIDI from an unidentified legacy Launchpad and injects Windows keyboard input. Later versions must support modern Launchpads, two-device layouts, and an RP2030 + FT232RL serial hardware path. Raw MIDI-to-key mapping would couple every new device to every output method.

## Decision

Use four boundaries: model-specific device adapters, normalized controls, logical action/profile state, and replaceable output backends. LED rendering consumes state but is not part of the output transition path.

## Consequences

The MVP has more interfaces than a direct mapper, but protocol and transport changes do not rewrite profile logic. State ownership and Release All behavior can be tested once for every backend.

## Verification

Run the same mapping/state-machine test suite against a fake Windows backend and a fake serial backend. Device adapter tests must not mention OS key codes.
