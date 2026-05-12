---
created: 2026-05-12T06:16:41.502Z
title: Replace integration test stubs using assert!(true)
area: testing
resolves_phase: 14
files:
  - tests/
---

## Problem

Phase 11 integration tests contain stubs that use `assert!(true)` — these pass unconditionally and provide zero actual verification. They give a false sense of test coverage while testing nothing.

Identified in v1.0 milestone audit (`.planning/v1.0-MILESTONE-AUDIT.md`, Phase 11 tech debt, WR-09).

## Solution

Locate all `assert!(true)` stubs in integration tests and replace with meaningful assertions that verify actual behavior (output values, state changes, error conditions). If a proper integration test requires external dependencies (FFmpeg, network), use `#[ignore]` with a descriptive comment instead of a vacuous assert.
