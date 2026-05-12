---
created: 2026-05-12T06:15:25.665Z
title: Fix Tauri State read locks held across long pipelines
area: tauri
resolves_phase: 18
files:
  - src-tauri/src/commands/
---

## Problem

Tauri commands acquire read locks on shared State (registry, prompt_manager, config) and hold them for the entire duration of pipeline execution. Since pipelines can take minutes (LLM calls, TTS generation, FFmpeg processing), this blocks all other Tauri commands from accessing state during that window.

Identified in v1.0 milestone audit (`.planning/v1.0-MILESTONE-AUDIT.md`, Phase 10 tech debt, WR-01).

## Solution

Extract needed data from state under short-lived read locks, then drop the lock before starting long-running pipeline work. Pass cloned/extracted data into the pipeline instead of holding guards across the entire call.
