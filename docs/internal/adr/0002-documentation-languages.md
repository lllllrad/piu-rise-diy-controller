# ADR-0002: Maintain Bilingual User Docs and English-only Internal Docs

- Status: Accepted
- Date: 2026-08-11

## Context

Users need English and Korean instructions. Internal engineering work needs one normative language to avoid duplicated architecture and protocol specifications.

## Decision

Maintain paired English and Korean user documents with matching relative paths and structure. Maintain internal technical documentation, contributor policy, ADRs, code comments, and identifiers only in English. Do not translate stable configuration and diagnostic identifiers.

## Consequences

User documentation requires a parity check and deliberate translation updates. Technical decisions have one source of truth. Contributors must be able to work with English technical material.

## Verification

Add a documentation check that compares file paths and heading/key coverage between `docs/user/en/` and `docs/user/ko/` once the build tooling exists.
