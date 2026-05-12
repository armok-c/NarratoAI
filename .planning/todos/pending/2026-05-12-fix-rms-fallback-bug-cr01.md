---
created: 2026-05-12T06:13:14.835Z
title: Fix RMS fallback bug CR-01 in normalizer
area: audio
files:
  - src/audio/normalizer.rs:288
---

## Problem

RMS calculation in `src/audio/normalizer.rs:288` uses `i16::MAX` as reference level for f32 audio samples. Since f32 samples are in [-1.0, 1.0] range, dividing by 32767 causes ~90 dB over-amplification on the fallback normalization path. This is a BLOCKER-level bug identified in the v1.0 milestone audit (see `.planning/v1.0-MILESTONE-AUDIT.md`, Phase 11 tech debt, CR-01).

## Solution

Replace `20.0 * (rms / (i16::MAX as f64)).log10()` with `20.0 * rms.log10()` to compute dBFS correctly for floating-point samples. The i16::MAX reference only applies to integer PCM; f32 samples are already in normalized [-1.0, 1.0] range where 0 dBFS = 1.0.
