# Phase 11 — Deferred Items

## Pre-existing Compilation Error in `tests/tts_test.rs`

**Discovered during:** Plan 11-05 execution (pre-existing, not caused by this plan)

**Issue:** `tests/tts_test.rs:113` — `tts::synthesize()` function signature changed to require an 8th argument (`Option<&AppConfig>`), but the test only passes 7 arguments:

```rust
error[E0061]: this function takes 8 arguments but 7 arguments were supplied
   --> tests\tts_test.rs:113:13
    |
113 |     let _ = tts::synthesize(engine, text, voice_name, 1.0, 0.0, Path::new("/tmp/_narratoai_sig_check.mp3"), None);
    |             ^^^^^^^^^^^^^^^
```

**Root cause:** Phase 12 TTS engine work added an `AppConfig` parameter to `tts::synthesize()` but did not update the corresponding integration test.

**Impact:** `cargo test` full suite fails to compile. The 3 new test files in Plan 11-05 compile correctly when built individually.

**Recommendation:** Fix in a subsequent plan (or commit patch to `tests/tts_test.rs:113` to pass an 8th `None` argument).

**Discovered:** 2026-04-30
