# Phase 4: Prompt System + Visual Analyzer - Research

**Researched:** 2026-04-30
**Domain:** Regex-based template engine + FFmpeg frame extraction + async-openai vision batch analysis
**Confidence:** HIGH

## Summary

Phase 4 implements two independent subsystems (Prompt System and Visual Analyzer) that together enable LLM-based video frame analysis for automated narration script generation. The Prompt System provides a compile-time embedded, versioned template registry with regex-based `${variable}` replacement and 6 built-in filters. The Visual Analyzer extracts keyframes from video via FFmpeg (with 4-level fallback), encodes them as base64 data URLs, sends batches to a visual LLM via the existing Phase 2 LlmProvider, and deserializes structured JSON analysis reports.

Both subsystems follow patterns already established in Phases 1-2: thiserror error enums, `Arc<RwLock<T>>` global instances, explicit initialization functions, spawn_blocking for FFmpeg operations, and Semaphore for concurrency control. The Prompt System adds `regex = "1.11"` as the sole new dependency.

**Primary recommendation:** Build Prompt System first (types, template renderer, registry), then Visual Analyzer (frame extractor, analyzer orchestrator). The two subsystems share no internal dependencies but both are consumed by Phase 6.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

D-01: compile-time embedding via include_str!()
D-02: Rust function registration — each Prompt through Rust function registering to Registry
D-03: templates in src/prompt/templates/{category}/{name}_v{version}.md, registrations in src/prompt/registry/{category}.rs
D-04: version as String, format "v1.0", "v2.1"
D-05: self-implemented regex replacement (not minijinja/tera)
D-06: all 6 filters — upper, lower, title, strip, truncate, json — via HashMap<String, fn>
D-07: missing variables return PromptError::TemplateRender with the missing param name
D-08: support both ${variable} and $variable syntax
D-09: new src/visual/ module — frame extraction and visual analysis in one module
D-10: full 4-level fallback — 1) hardware accelerated JPEG, 2) software decode JPEG, 3) PNG format (convert to JPEG), 4) BMP/MJPEG
D-11: extract_frames() receives explicit params (interval_seconds, quality, output_dir), defaults from config.toml [frames]
D-12: fast path (single ffmpeg fps=1/{interval} filter) + per-frame fallback
D-13: disk files -> load -> batch send to LLM (Phase 2 image preprocessing)
D-14: collect errors and continue — single frame failure -> placeholder, LLM call failure retry -> partial results + error stats
D-15: FrameObservation + BatchAnalysisResult structs, serde deserialization
D-16: reuse ProgressCallback pattern for visual analysis
D-17: register_all_prompts() explicit init function
D-18: Arc<RwLock<PromptRegistry>> global instance
D-19: hybrid prompt mode — frame analysis prompt from Registry, JSON format constraint hardcoded in analyzer
D-20: PromptManager provides validate_output() method
D-21: two error enums — PromptError (prompt domain) and VisualError (visual domain)
D-22: PromptError variants: TemplateRender, NotFound, Registration, Validation, Version. VisualError variants: FrameExtraction, Analysis, BatchPartial
D-23: caller specifies output_dir for frame extraction
D-24: frame naming keyframe_{frame_number:06d}_{HHMMSSmmm}.jpg

### Claude's Discretion

None — all decisions made by user.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PRMP-01 | Prompt registry — category/name/version 3-level index | Python PromptRegistry pattern maps directly to `HashMap<String, HashMap<String, HashMap<String, Prompt>>>`; D-18 uses Arc<RwLock<>> for thread safety |
| PRMP-02 | Template rendering — Jinja-style variable substitution | D-05 chooses regex; Python uses simple str.replace for `${var}`/`$var` + regex for filters; regex `find_iter` pre-validates all variables before replace_all (solves the "closure can't return Result" problem) |
| PRMP-03 | Prompt categories — documentary/short_drama_editing/short_drama_narration | D-03 maps to 3 registry categories; template files in 3 subdirectories under src/prompt/templates/ |
| PRMP-04 | Prompt validation — template integrity check on load | D-17/20: register_all_prompts() loads all templates, validate_output() + validate_template() check parameter completeness |
| VISL-01 | Video frame extraction — keyframes at time intervals | D-10/11/12 define 4-level fallback + fast path; reuses Phase 1 ffmpeg spawn_blocking pattern (src/ffmpeg/command.rs `run_ffmpeg()`) |
| VISL-02 | Batch frame analysis — frames to vision LLM | D-13/19: disk -> load -> base64 -> LlmProvider::analyze_images() + hardcoded JSON format prompt |
| VISL-03 | Structured analysis result — JSON report | D-15: FrameObservation + BatchAnalysisResult structs with serde deserialization; parse_and_retry() for markdown code block stripping |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Template storage | Compile-time (code) | — | D-01: include_str!() embeds .md files at compile time; no runtime file reading |
| Template rendering | Library (sync) | — | Pure regex operation, microseconds, no IO, no async needed |
| Prompt registry | Library (sync RwLock) | — | HashMap lookups read-lock, milliseconds, concurrent reads OK |
| Output validation | Library (sync) | — | serde JSON validation, pure Rust, no external dependencies |
| Frame extraction | Library (sync, spawn_blocking) | — | ffmpeg-sidecar spawn is blocking; uses Phase 1 run_ffmpeg() wrapper |
| Image preprocessing | Library (sync, spawn_blocking) | — | image crate decode + JPEG encode is CPU-heavy; Phase 2 already uses spawn_blocking |
| Vision LLM batch analysis | Library (async) | — | async-openai HTTP calls; non-blocking; Semaphore concurrency control |
| Frame directory management | Caller (Phase 6) | — | D-23: visual module receives output_dir, does not create or clean up |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| async-openai | 0.36 | Vision LLM API client | Phase 2 established; features=["chat-completion"] gets CreateChatCompletionRequestArgs, ImageUrl, ResponseFormat |
| regex | 1.11 | Template rendering engine | D-05: self-implemented regex replacement; only viable Rust regex crate |
| ffmpeg-sidecar | 2.5.1 | FFmpeg subprocess management | Phase 1 established; FfmpegCommand + spawn + iter pattern for frame extraction |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde | 1.0.228 | FrameObservation/BatchAnalysisResult deserialization | Parse LLM JSON responses; features=["derive"] for #[derive(Deserialize)] |
| serde_json | 1.0.140 | JSON parsing | from_str::<FrameObservation>() for LLM response parsing |
| image | 0.25 | Frame image preprocessing | Already used by Phase 2 image_utils.rs for thumbnail + JPEG encode |
| base64 | 0.22 | data URL encoding | Already used by Phase 2 image_utils.rs; STANDARD encoder |
| thiserror | 2.0.18 | PromptError + VisualError derive | Phase 1 pattern; #[derive(Error, Debug)] with Chinese #[error("...")] |
| tracing | 0.1.44 | Structured logging | Phase 1 pattern; #[instrument] on render/analyze for monitoring |
| tokio | 1.52.1 | Async runtime + Semaphore | Phase 1 pattern; Semaphore for concurrency control in batch analysis |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| regex + self-implement | minijinja/tera/handlebars | Self-implement ensures `${variable}` syntax compatibility with Python v0.7.8; alternatives use `{{ }}` requiring template rewrites |
| ffmpeg-sidecar | video-rs, rav1e | video-rs/rav1e do not support 4-level output format fallback (JPEG->PNG->BMP/MJPEG); ffmpeg-sidecar gives full ffmpeg CLI flexibility |

**Installation:**
```bash
cargo add regex@1.11
```

**Version verification:** regex 1.11 was verified in AI-SPEC as the current stable version. serde 1.0.228, serde_json 1.0.140, tokio 1.52.1, image 0.25, base64 0.22, thiserror 2.0.18, tracing 0.1.44 are already in Cargo.toml from Phases 1-2. [VERIFIED: Phase 2 CONTEXT.md, Cargo.toml]

## Architecture Patterns

### System Architecture Diagram

```
[Template Files (.md)]
  │
  ▼  (include_str! at compile time)
[PromptRegistry] ─── Arc<RwLock<>> ─── [PromptManager]
  │                                          │
  │  category/name/version                    │ get_prompt()
  │  3-level lookup                           │ validate_output()
  │                                          │
  ▼                                          ▼
[TemplateRenderer] ──── regex ────► [Rendered Prompt String]
  │
  │   ${variable} | $variable | ${variable|filter}
  │   6 filters: upper, lower, title, strip, truncate, json
  │   Missing variable → PromptError::TemplateRender
  ▼
[Prompt text] ─────┐
                   │
                   ├──► [Visual Analyzer Orchestrator]
                   │        │
[Video File]       │        │
  │                │        │
  ▼                │        │
[FrameExtractor] ──┘        │
  │                          │
  │  Fast Path: fps filter   │
  │  Fallback: 4-level       │
  ▼                          │
[Frame JPEGs on disk]        │
  │                          │
  ▼                          │
[image_to_base64_data_url]   │
  │   (Phase 2 image_utils)  │
  │   thumbnail 1024px       │
  │   JPEG quality 85        │
  │   base64 data URL        │
  ▼                          │
[LlmProvider::analyze_images]│
  │   (Phase 2 provider)      │
  │   Semaphore concurrency  │
  │   batch_size groups      │
  ▼                          │
[Raw LLM Response (JSON)]  ◄─┘
  │
  ▼
[parse_and_retry()]
  │  Strip ```json code fences
  │  serde_json::from_str<FrameObservation>
  ▼
[BatchAnalysisResult]
  ├── observations: Vec<FrameObservation>
  │     ├── frame_number, timestamp
  │     ├── scene_description, objects, actions
  │     ├── on_screen_text, visual_salience
  ├── overall_activity_summary
  ├── total_frames, analyzed_batches
  └── errors: Vec<String>
```

### Recommended Project Structure
```
src/
├── prompt/                         # Prompt System (D-01~D-08)
│   ├── mod.rs                      # pub mod declarations
│   ├── types.rs                    # PromptMetadata, ModelType, OutputFormat, Prompt struct
│   ├── error.rs                    # PromptError enum (D-21/22)
│   ├── registry.rs                 # PromptRegistry: {category: {name: {version: Prompt}}}
│   ├── template.rs                 # TemplateRenderer: regex + 6 filters (D-05/06)
│   ├── manager.rs                  # PromptManager: get_prompt, register_prompt, search, validate_output
│   ├── validators.rs               # Output validators (JSON, narration script, plot analysis)
│   ├── registry/                   # Registration functions (D-17)
│   │   ├── mod.rs                  # pub mod + register_all_prompts()
│   │   ├── documentary.rs          # frame_analysis_v1.0, narration_generation_v2.0
│   │   ├── short_drama_editing.rs  # plot_extraction_v2.0
│   │   └── short_drama_narration.rs # script_generation_v1.0 (placeholder)
│   └── templates/                  # .md template files (D-01: include_str! targets)
│       ├── documentary/
│       │   ├── frame_analysis_v1.0.md
│       │   └── narration_generation_v2.0.md
│       ├── short_drama_editing/
│       │   └── plot_extraction_v2.0.md
│       └── short_drama_narration/
│           └── script_generation_v1.0.md
│
├── visual/                         # Visual Analyzer (D-09~D-16)
│   ├── mod.rs                      # pub mod declarations
│   ├── error.rs                    # VisualError enum (D-21/22)
│   ├── types.rs                    # FrameObservation, BatchAnalysisResult structs (D-15)
│   ├── frame_extractor.rs          # extract_frames: fast path + 4-level fallback (D-10/12)
│   └── analyzer.rs                 # Visual analysis orchestrator (D-13/14/16/19)
│
├── ffmpeg/                         # Existing (Phase 1) — frame_extractor calls via run_ffmpeg()
├── llm/                            # Existing (Phase 2) — analyzer calls LlmProvider + image_utils
├── config/                         # Existing (Phase 1) — FramesSection config types
├── error.rs                        # Existing — PromptError and VisualError variants live here (or in modules)
├── lib.rs                          # Add: pub mod prompt; pub mod visual;
└── Cargo.toml                      # Add: regex = "1.11"
```

### Pattern 1: Template Rendering with Pre-validation

**What:** Regex-based `${variable}` and `$variable` replacement with 6 filters. Because `regex::Regex::replace_all` closure cannot return `Result`, use a two-pass approach: first validate all variables via `find_iter()`, then perform replacement (which is guaranteed to succeed).

**When to use:** All template rendering operations.

**Example:**
```rust
// Source: AI-SPEC Section 3, adapted for the closure-can't-return-Result pitfall
// Two-pass approach: validate first, then replace
pub fn render(template: &str, vars: &HashMap<&str, &str>) -> Result<String, PromptError> {
    let re = Regex::new(r"\$\{(\w+)\}|\$(\w+)").map_err(|e| {
        PromptError::TemplateRender(format!("正则编译失败: {}", e))
    })?;

    // Pass 1: Extract all variable names, validate all exist
    let mut missing = Vec::new();
    for cap in re.find_iter(template) {
        let name = cap.as_str().trim_start_matches("${").trim_end_matches('}').trim_start_matches('$');
        if !vars.contains_key(name) {
            missing.push(name.to_string());
        }
    }
    if !missing.is_empty() {
        return Err(PromptError::TemplateRender(format!(
            "缺少必需参数: {}",
            missing.join(", ")
        )));
    }

    // Pass 2: Replace (all variables guaranteed to exist)
    let result = re.replace_all(template, |caps: &regex::Captures| {
        let name = caps.get(1).or_else(|| caps.get(2))
            .map(|m| m.as_str())
            .unwrap_or("");
        vars.get(name).copied().unwrap_or("") // safe: pre-validated
    });

    // Pass 3: Apply filters (${variable|filter_name})
    let filter_re = Regex::new(r"\$\{(\w+)\|(\w+)\}").unwrap();
    let result = filter_re.replace_all(&result, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let filter_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if let Some(filter_fn) = builtin_filters().get(filter_name) {
            if let Some(value) = vars.get(var_name) {
                return filter_fn(value);
            }
        }
        caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
    });

    Ok(result.to_string())
}
```

### Pattern 2: Frame Extraction with Fast Path + Fallback

**What:** Two-phase frame extraction using ffmpeg-sidecar. First attempts a single ffmpeg command with `fps=1/{interval}` filter for speed. If that fails (zero frames), falls back to per-frame extraction with 4 output format levels.

**When to use:** All frame extraction from video files.

**Example:**
```rust
// Source: AI-SPEC Section 4 Core Pattern, aligned with Python VideoProcessor
pub async fn extract_frames(
    video_path: &Path,
    output_dir: &Path,
    interval_seconds: f64,
    quality: Option<u32>,
    progress: Option<ProgressCallback>,
) -> Result<usize, VisualError> {
    // D-12: Fast path — single ffmpeg fps filter command
    // Must use spawn_blocking since ffmpeg-sidecar is synchronous
    let fast_result = tokio::task::spawn_blocking({
        let video = video_path.to_path_buf();
        let output = output_dir.to_path_buf();
        move || -> Result<usize, String> {
            let mut cmd = FfmpegCommand::new();
            cmd.hide_banner()
                .loglevel("error")
                .input(video.to_str().unwrap_or(""))
                .video_filter(format!("fps=1/{}", interval_seconds))
                .output(output.join("fastframe_%06d.jpg").to_str().unwrap_or(""))
                .overwrite();
            // ... spawn, iterate events, handle errors
            // On success, rename fastframe_XXXXXX.jpg -> keyframe_XXXXXX_HHMMSSmmm.jpg
            Ok(frame_count)
        }
    }).await.map_err(|e| VisualError::FrameExtraction(e.to_string()))?;

    match fast_result {
        Ok(count) if count > 0 => return Ok(count),
        _ => {
            // Clean up fast path artifacts
            // Fallback to per-frame extraction with 4 output format levels
            extract_frames_fallback(video_path, output_dir, interval_seconds, quality, progress).await
        }
    }
}
```

### Pattern 3: Batch Visual Analysis with Error Collection

**What:** Calls LlmProvider::analyze_images() (Phase 2) which handles base64 encoding, batching, and Semaphore concurrency. The visual analyzer wraps this with JSON schema validation and error collection per D-14.

**When to use:** After frames are extracted, send them to vision LLM for analysis.

**Example:**
```rust
// Source: AI-SPEC Section 3 Entry Point Pattern
pub async fn analyze_video_frames(
    video_path: &Path,
    llm_provider: &dyn LlmProvider,
    prompt_template: &str,
    context: &HashMap<&str, &str>,
    batch_size: usize,
    max_concurrency: usize,
    progress: Option<ProgressCallback>,
) -> Result<BatchAnalysisResult, VisualError> {
    // Step 1: Extract keyframes
    let frame_dir = ...;  // caller-specified (D-23)
    let frame_count = extract_frames(video_path, &frame_dir, 3.0, None, progress.clone())
        .await.map_err(VisualError::FrameExtraction)?;

    // Step 2: Collect frame paths (D-24 naming pattern)
    let frame_paths = collect_frame_paths(&frame_dir)?;

    // Step 3: Hardcoded JSON format constraint (D-19)
    let system_prompt = Some("你是一个专业的影视解说视觉分析助手。分析视频帧并返回结构化JSON。");
    let rendered_prompt = render_template(prompt_template, context)?;
    let full_prompt = format!(
        "{}\n\nIMPORTANT: Respond with valid JSON matching this schema: {{frame_observations: [{{frame_number, timestamp, scene_description, objects, actions, on_screen_text, visual_salience}}], overall_activity_summary: string}}",
        rendered_prompt
    );

    // Step 4: Batch send via Phase 2 LlmProvider
    let results = llm_provider.analyze_images(
        &frame_paths, &full_prompt, system_prompt.as_deref(),
        Some(batch_size), Some(max_concurrency),
        Some(LlmResponseFormat::Json),
        Some(0.1), Some(4096),
    ).await.map_err(|e| VisualError::Analysis(e.to_string()))?;

    // Step 5: Parse results, collecting errors (D-14)
    let mut observations = Vec::new();
    let mut errors = Vec::new();
    for (idx, text) in results.iter().enumerate() {
        match parse_and_retry(text, 2) {
            Ok(obs) => observations.push(obs),
            Err(e) => errors.push(format!("批次 {}: {}", idx, e)),
        }
    }

    if observations.is_empty() && !errors.is_empty() {
        return Err(VisualError::BatchPartial { analyzed_count: 0, total_count: results.len(), errors });
    }
    Ok(BatchAnalysisResult { observations, total_frames: frame_count, analyzed_batches: results.len(), errors })
}
```

### Anti-Patterns to Avoid
- **Using `regex::Regex::replace_all` closure for error handling:** The closure signature is `impl Fn(&Captures) -> String`, it cannot return `Result`. Always pre-validate with `find_iter()` before replacement. [CITED: AI-SPEC pitfall 2]
- **Calling `FfmpegCommand::spawn()` directly in async context:** ffmpeg-sidecar spawn is synchronous blocking. Always wrap in `tokio::task::spawn_blocking()` (Phase 1 has `run_ffmpeg()` helper). [CITED: AI-SPEC pitfall 3]
- **Assuming LLM always returns clean JSON:** Even with `response_format=JsonObject`, many models return markdown code block wrapped JSON. Always strip ```json / ``` before `serde_json::from_str()`. [CITED: AI-SPEC pitfall 5]

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| LLM API HTTP client | Custom reqwest-based client | async-openai (already Phase 2) | Handles SSE streaming, retry backoff, error mapping, authentication. Phase 2 already wraps as LlmProvider trait |
| Image to base64 data URL | Custom JPEG encode + base64 | Phase 2 image_utils::image_to_base64_data_url() | Already implements thumbnail 1024px + Lanczos3 + JPEG quality 85 + base64 STANDARD + data URL format. Has JPEG passthrough optimization |
| Template variable validation | Skip validation, let LLM get broken prompts | regex find_iter() pre-validation | Prevents silent prompt corruption; D-07 mandates this |
| Concurrent API throttling | Custom counting semaphore | tokio::sync::Semaphore | Standard Rust async primitive; Phase 2 already uses same pattern |

**Key insight:** Phase 4 leverages heavily on infrastructure built in Phases 1-2. The new code is primarily the Prompt System (template engine + registry) and the Visual Analyzer orchestration layer. The heavy lifting of FFmpeg execution, LLM API calls, and image preprocessing is already implemented. Do not duplicate any of these.

## Common Pitfalls

### Pitfall 1: Regex replace_all Closure Can't Return Result
**What goes wrong:** D-07 requires missing variables to return an error. But `Regex::replace_all` with closure has signature `Fn(&Captures) -> String`, not `Result<String, E>`. If the closure panics or returns a sentinel for missing variables, the error handling is fragile.
**Why it happens:** regex crate's API design: closures for replacement cannot fail.
**How to avoid:** Use two-pass approach. Pass 1: `find_iter()` to extract all variable names, validate against context HashMap, collect missing names. If any missing, return `PromptError::TemplateRender` before Pass 2. Pass 2: `replace_all()` is guaranteed all variables exist in context.
**Warning signs:** Panic in template rendering; empty strings in rendered output where variables should be.

### Pitfall 2: spawn_blocking + Semaphore Deadlock
**What goes wrong:** If `Semaphore::acquire_owned()` is called inside `tokio::spawn_blocking()` (or vice versa), and the blocking pool is exhausted waiting for permits while the async tasks that release permits are waiting for blocking pool threads, a deadlock occurs.
**Why it happens:** Mixing sync blocking and async waiting.
**How to avoid:** Acquire the Semaphore permit in async context BEFORE entering `spawn_blocking`. Or use `tokio::task::spawn()` (not `spawn_blocking`) for the image preprocessing tasks, since `image_to_base64_data_url()` is already wrapped in `spawn_blocking` inside Phase 2's `analyze_images()`.
**Warning signs:** Frame extraction or image processing tasks hang indefinitely.

### Pitfall 3: Markdown Code Block Wrapping in LLM Responses
**What goes wrong:** Even when `response_format=JsonObject` is set, some models (especially through proxy gateways like SiliconFlow) return JSON wrapped in ` ```json { ... } ``` `. `serde_json::from_str()` fails on this.
**How to avoid:** Always apply defensive stripping before JSON parsing:
```rust
fn parse_and_retry(text: &str, max_attempts: usize) -> Result<FrameObservation, VisualError> {
    for attempt in 1..=max_attempts {
        let cleaned = text.trim()
            .strip_prefix("```json").or_else(|| text.trim().strip_prefix("```"))
            .map(|s| s.trim_end_matches("```").trim())
            .unwrap_or(text.trim());
        match serde_json::from_str::<FrameObservation>(cleaned) {
            Ok(obs) => return Ok(obs),
            Err(e) => { /* log and retry once */ }
        }
    }
    Err(VisualError::Analysis("JSON parse failed".into()))
}
```
**Warning signs:** JSON parse errors in batch analysis results; `errors` list in `BatchAnalysisResult` is non-empty for batches.

### Pitfall 4: Async-openai builder API response_format null
**What goes wrong:** Calling `.response_format(value)` with a default/empty value sends `response_format: null` in the request body. Some API gateways (SiliconFlow) return 400 error for this.
**How to avoid:** Use conditional builder pattern: `if use_json { request_builder.response_format(ResponseFormat::JsonObject); }` — never call the method unconditionally. This is already handled in Phase 2's `openai_compatible.rs`.
**Warning signs:** 400 Bad Request from LLM API only when response_format is involved.

### Pitfall 5: Frame Output Format Compatibility
**What goes wrong:** FFmpeg with hardware acceleration (especially NVIDIA CUDA/NVENC on Windows) can produce broken JPEG output due to MJPEG encoder incompatibilities. The first frame may succeed but subsequent frames produce corrupted files.
**Why it happens:** The `-f image2` format with `-q:v` quality flag conflicts with some hwaccel pixel format pipelines.
**How to avoid:** Implement the full 4-level fallback exactly as Python's `_extract_single_frame_optimized()`:
1. Software decode (no hwaccel args, `-ss` before `-i`, `-pix_fmt yuv420p`)
2. Hardware acceleration (append hwaccel args from config)
3. Plain software (`-ss` after `-i`, `-avoid_negative_ts make_zero`, lower quality)
4. Ultra compatibility: PNG format first, then BMP/MJPEG as last resort
**Warning signs:** Frame count is 0 after extraction; extracted images have 0 bytes; progress stalls at frame extraction step.

## Code Examples

### TemplateRenderer — Two-Pass Validation + Filter Application
```rust
// Source: AI-SPEC Section 3 pitfall #2 workaround + Python template.py
use regex::Regex;
use std::collections::HashMap;

type FilterFn = fn(&str) -> String;

fn builtin_filters() -> HashMap<&'static str, FilterFn> {
    let mut m = HashMap::new();
    m.insert("upper", |s| s.to_uppercase());
    m.insert("lower", |s| s.to_lowercase());
    m.insert("title", |s| {
        let mut chars = s.chars().peekable();
        let mut out = String::with_capacity(s.len());
        let mut new_word = true;
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                out.push(c);
                new_word = true;
            } else if new_word {
                out.extend(c.to_uppercase());
                new_word = false;
            } else {
                out.push(c);
            }
        }
        out
    });
    m.insert("strip", |s| s.trim());
    m.insert("truncate", |s| {
        if s.len() <= 100 { s.to_string() }
        else { format!("{}...", &s[..97]) }
    });
    m.insert("json", |s| {
        // Simple JSON string escape + wrapping
        serde_json::to_string(s).unwrap_or_else(|_| format!("\"{}\"", s))
    });
    m
}

pub fn render(template: &str, vars: &HashMap<&str, &str>) -> Result<String, PromptError> {
    let var_re = Regex::new(r"\$\{(\w+)\}|\$(\w+)").map_err(|e| {
        PromptError::TemplateRender(format!("正则编译失败: {}", e))
    })?;

    // Pass 1: validate all variables exist
    let mut requested: Vec<String> = Vec::new();
    for cap in var_re.captures_iter(template) {
        let name = cap.get(1).or_else(|| cap.get(2))
            .map(|m| m.as_str())
            .unwrap_or("");
        if !vars.contains_key(name) && !requested.contains(&name.to_string()) {
            requested.push(name.to_string());
        }
    }
    if !requested.is_empty() {
        return Err(PromptError::TemplateRender(format!(
            "缺少必需参数: {}",
            requested.join(", ")
        )));
    }

    // Pass 2: variable replacement
    let result = var_re.replace_all(template, |caps: &regex::Captures| {
        let name = caps.get(1).or_else(|| caps.get(2))
            .map(|m| m.as_str()).unwrap_or("");
        vars.get(name).copied().unwrap_or("") // safe: validated in pass 1
    });

    // Pass 3: filter application (${variable|filter})
    let filter_re = Regex::new(r"\$\{(\w+)\|(\w+)\}").unwrap();
    let filters = builtin_filters();
    let result = filter_re.replace_all(&result, |caps: &regex::Captures| {
        let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let filter_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if let (Some(filter_fn), Some(value)) = (filters.get(filter_name), vars.get(var_name)) {
            return filter_fn(value);
        }
        // If filter not found or var not found, keep original text
        caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
    });

    Ok(result.to_string())
}
```

### FrameExtractor — Fast Path with spawn_blocking
```rust
// Source: AI-SPEC Section 4 Core Pattern + Python VideoProcessor._extract_frames_fast_path
pub async fn extract_frames_fast_path(
    video_path: &Path,
    output_dir: &Path,
    interval_seconds: f64,
    quality: u32,
) -> Result<usize, VisualError> {
    let video = video_path.to_path_buf();
    let output = output_dir.to_path_buf();
    let pid = std::process::id();

    tokio::task::spawn_blocking(move || -> Result<usize, VisualError> {
        let pattern = output.join("fastframe_%06d.jpg");
        let mut cmd = FfmpegCommand::new();
        cmd.hide_banner()
            .loglevel("error")
            .input(video.to_str().ok_or_else(|| {
                VisualError::FrameExtraction("视频路径无效".into())
            })?)
            .video_filter(format!("fps=1/{}", interval_seconds))
            .codec_video("mjpeg")
            .args(["-q:v", &quality.to_string()])
            .start_number(0)
            .overwrite()
            .output(pattern.to_str().unwrap());

        let mut child = cmd.spawn()
            .map_err(|e| VisualError::FrameExtraction(format!("FFmpeg启动失败: {}", e)))?;
        let iter = child.iter()
            .map_err(|e| VisualError::FrameExtraction(format!("FFmpeg迭代失败: {}", e)))?;

        // Consume events to process frames
        for event in iter {
            match event {
                FfmpegEvent::Error(e) => {
                    tracing::warn!("FFmpeg fast path error: {}", e);
                }
                _ => {}
            }
        }

        let _ = child.wait();
        // Rename fastframe_XXXXXX.jpg -> keyframe_XXXXXX_HHMMSSmmm.jpg
        rename_fast_path_frames(&output, interval_seconds)
    }).await.map_err(|e| VisualError::FrameExtraction(format!("任务崩溃: {}", e)))?;
}
```

### PromptRegistry with Arc<RwLock>
```rust
// Source: AI-SPEC Section 4 State Management + Python registry.py
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type SharedPromptRegistry = Arc<RwLock<PromptRegistry>>;

pub struct PromptRegistry {
    prompts: HashMap<String, HashMap<String, HashMap<String, Prompt>>>,
    default_versions: HashMap<String, HashMap<String, String>>,
}

impl PromptRegistry {
    pub fn register(&mut self, prompt: Prompt, is_default: bool) -> Result<(), PromptError> {
        let category = prompt.category.clone();
        let name = prompt.name.clone();
        let version = prompt.version.clone();

        if self.prompts.entry(category.clone())
            .or_default()
            .entry(name.clone())
            .or_default()
            .contains_key(&version)
        {
            return Err(PromptError::Registration(format!(
                "版本已存在: {}.{} v{}", category, name, version
            )));
        }

        self.prompts[&category][&name].insert(version.clone(), prompt);

        if is_default || !self.default_versions.get(&category)
            .and_then(|m| m.get(&name))
            .is_some()
        {
            self.default_versions.entry(category).or_default()
                .insert(name, version);
        }

        Ok(())
    }

    pub fn get(&self, category: &str, name: &str, version: Option<&str>) -> Result<&Prompt, PromptError> {
        let version = match version {
            Some(v) => v.to_string(),
            None => self.default_versions.get(category)
                .and_then(|m| m.get(name))
                .ok_or_else(|| PromptError::NotFound {
                    category: category.to_string(),
                    name: name.to_string(),
                    version: None,
                })?.clone(),
        };

        self.prompts.get(category)
            .and_then(|m| m.get(name))
            .and_then(|m| m.get(&version))
            .ok_or_else(|| PromptError::NotFound {
                category: category.to_string(),
                name: name.to_string(),
                version: Some(version),
            })
    }
}
```

### FrameObservation + BatchAnalysisResult Structs
```rust
// Source: AI-SPEC Section 4b Structured Outputs with serde
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FrameObservation {
    pub frame_number: u64,
    pub timestamp: String,
    pub scene_description: String,
    pub objects: Vec<String>,
    pub actions: Vec<String>,
    pub on_screen_text: Option<String>,
    pub visual_salience: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BatchAnalysisResult {
    pub observations: Vec<FrameObservation>,
    pub overall_activity_summary: Option<String>,
    pub total_frames: usize,
    pub analyzed_batches: usize,
    pub errors: Vec<String>,
}
```

### ProgressCallback Type (Reused from Phase 1)
```rust
// Source: src/ffmpeg/command.rs — Phase 1 D-15
pub type ProgressCallback = Box<dyn Fn(Option<f64>, &str) + Send + Sync>;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| string.Template (Python) | regex two-pass (Rust) | Phase 4 | Same `${variable}` syntax but stricter: Rust version validates all variables before rendering, Python version silently skips missing variables |
| asyncio.Semaphore (Python) | tokio::sync::Semaphore (Rust) | Phase 2 | Same pattern; Phase 4 reuses the already-established tokio semaphore |
| PIL.Image.thumbnail (Python) | image::imageops::resize (Rust) | Phase 2 | Same Lanczos3 filter, same 1024px target; Phase 4 reuses Phase 2 implementation |

**Deprecated/outdated:**
- Python `TemplateRenderer.render()` uses simple `str.replace()` for variables: Rust version uses `regex` for both pass 1 (validation) and pass 2 (replacement), because `str.replace()` in a HashMap loop would need extra work for filters
- Python `VideoProcessor._extract_single_frame_optimized()` calls FFmpeg per-frame sequentially: Rust version uses spawn_blocking but the per-frame fallback is still sequential (ffmpeg is the bottleneck, not Rust concurrency)

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The existing `image_to_base64_data_url()` in src/llm/image_utils.rs handles JPEG output from frame extraction via the JPEG passthrough path | Code Examples | Low — if JPEG passthrough fails, it falls back to full decode+re-encode, still correct, just slower |
| A2 | The existing `src/ffmpeg/command.rs::run_ffmpeg()` pattern can be reused for frame extraction | Architecture Patterns | Low — frame extraction needs different ffmpeg args but same spawn_blocking + CancellationToken pattern; may need a new helper rather than reuse run_ffmpeg directly |
| A3 | The existing `LlvmProvider::analyze_images()` in src/llm/openai_compatible.rs handles all batch size + concurrency internally | Standard Stack | Medium — if analyze_images returns errors at the batch level, the visual analyzer must wrap and continue per D-14; Phase 2 currently returns `Err` on ANY batch failure |
| A4 | Python template .md files can be directly used as include_str! targets | Code Examples | Medium — content needs no modification (same ${variable} syntax), but file path conventions may differ (+ Python template format uses `"` vs raw .md has trailing spaces) |

**Note:** A3 is the most significant risk. Phase 2's `analyze_images()` currently returns `Err` on any single batch failure (line 541-543 in openai_compatible.rs: `Ok(Err(e)) => return Err(e)`). D-14 requires collecting partial errors and continuing. The visual analyzer in Phase 4 must either:
  1. Call `analyze_images()` per single batch (bypassing Phase 2's internal batching) and collect errors per batch, OR
  2. Accept that a single batch failure fails the entire analysis

Recommendation: The visual analyzer should call `analyze_images()` with `batch_size=all_frames` and `max_concurrency=1` PER BATCH, managing its own batches and error collection. This gives fine-grained error handling aligned with D-14.

## Open Questions

1. **Where should PromptError and VisualError live?**
   - What we know: Phase 1 put ConfigError and FFmpegError in `src/error.rs`. Phase 2 put LLMError there too. D-21 says "two error enums" but doesn't specify location.
   - What's unclear: Whether to add them to `src/error.rs` (consistent with previous phases) or put them in `src/prompt/error.rs` and `src/visual/error.rs` (better modularity, consistent with AI-SPEC project structure).
   - Recommendation: Put them in their module files (`src/prompt/error.rs`, `src/visual/error.rs`) for modularity. Add re-exports in `src/error.rs` if needed for backward compat. This matches the AI-SPEC structure recommendation and keeps module boundaries clean.

2. **Can Phase 2's `analyze_images()` be used directly for D-14 partial error collection?**
   - What we know: Phase 2 `analyze_images()` returns `Result<Vec<String>, LLMError>` — ANY batch failure causes full error.
   - What's unclear: Whether to modify Phase 2 to support per-batch error collection, or have the visual analyzer manage batches manually.
   - Recommendation: Visual analyzer manages its own batches and calls `client.chat().create()` (not `analyze_images()`) per batch. This gives direct control over error collection per D-14. Alternatively, call `analyze_images()` with single-frame "batches" (batch_size=1) but that's wasteful.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cargo | Build | yes | 1.93.1 | -- |
| rustc | Build | yes | 1.93.1 | -- |
| ffmpeg | Frame extraction | no | -- | ffmpeg-sidecar manages its own binary (download_ffmpeg feature) |
| Python 3 | Test data generation (Python reference comparison) | yes | 3.12.10 | Optional — not required for build |

**Missing dependencies with no fallback:** None. ffmpeg-sidecar handles FFmpeg installation via its `download_ffmpeg` feature (already in Cargo.toml).

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) |
| Config file | none (Cargo.toml `[dev-dependencies]`) |
| Quick run command | `cargo test -p narratoai-core prompt::tests -- --nocapture` |
| Full suite command | `cargo test -p narratoai-core -- --nocapture` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| PRMP-01 | registry get/register/search | unit | `cargo test prompt::registry::tests` | Wave 0 |
| PRMP-02 | template render correct + missing var error | unit | `cargo test prompt::template::tests` | Wave 0 |
| PRMP-03 | 3 categories registered | unit | `cargo test prompt::registry::tests::test_categories` | Wave 0 |
| PRMP-04 | validation rejects missing params | unit | `cargo test prompt::template::tests::test_missing_variable_error` | Wave 0 |
| VISL-01 | frame extraction fast path + fallback | integration | `cargo test visual::frame_extractor::tests` | Wave 0 |
| VISL-02 | batch analysis JSON deserialization | unit | `cargo test visual::types::tests::test_json_schema_compliance` | Wave 0 |
| VISL-03 | FrameObservation/BatchAnalysisResult serde round-trip | unit | `cargo test visual::types::tests::test_serde_roundtrip` | Wave 0 |

### Reference Dataset (per AI-SPEC Section 5)
| Category | Count | Content |
|----------|-------|---------|
| Prompt registry tests | 6 | 2 doc + 2 SDE + 2 SDN templates; 3 output formats |
| Template rendering (normal) | 3 | Golden file comparison with Python v0.7.8 output |
| Template rendering (error) | 2 | Missing variable + invalid filter → PromptError |
| Mock LLM responses | 10 | 3 valid JSON + 2 code-block-wrapped + 1 missing field + 1 camelCase + 1 empty objects + 1 plain text + 1 oversized |
| Test videos | 2 | 10s simple scene + 30s complex scene (transitions, action, text overlay) |

### Sampling Rate
- **Per task commit:** `cargo test -p narratoai-core prompt::tests && cargo test -p narratoai-core visual::tests`
- **Per wave merge:** `cargo test -p narratoai-core -- --nocapture && cargo clippy -p narratoai-core -- -D warnings`
- **Phase gate:** Full suite green + JSON schema compliance test passing before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `src/prompt/template.rs` — `#[cfg(test)] mod tests` with golden file comparisons
- [ ] `src/prompt/registry.rs` — `#[cfg(test)] mod tests` with registry CRUD operations
- [ ] `src/visual/types.rs` — `#[cfg(test)] mod tests` with 10 mock JSON deserialization tests
- [ ] `src/visual/frame_extractor.rs` — `#[cfg(test)] mod tests` with integration test (requires FFmpeg binary or mock)
- [ ] `tests/` — integration tests for the full visual analysis pipeline (mock LLM)

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | partial | Template variable validation via regex find_iter() pre-check; all template variables provided by internal pipeline (not user input) |
| V8 Data Protection | partial | Frame images sent to third-party LLM API over HTTPS; no PII handling required by module; Phase 2 handles API key security |

### Known Threat Patterns for {stack}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Template injection (malicious variable values) | Tampering | Template variables are provided by Phase 6 pipeline, not direct user input. Validate serialized/deserialized values through serde type checking. |
| Insecure deserialization (LLM JSON) | Tampering | `serde_json::from_str<FrameObservation>()` with `#[serde(deny_unknown_fields)]` rejects unexpected fields. `parse_and_retry()` limits to 2 retries. |

**Security justification:** Phase 4 operates as internal infrastructure consumed by Phase 6. Template variables originate from the pipeline, not from end-user input. Frame data is transient (written to disk then sent to LLM API). No authentication, session management, or access control requirements apply.

## Sources

### Primary (HIGH confidence)
- [AI-SPEC] `.planning/phases/04-prompt-system-visual-analyzer/04-AI-SPEC.md` — Framework decisions, code patterns, pitfall documentation, evaluation strategy
- [CONTEXT.md] `.planning/phases/04-prompt-system-visual-analyzer/04-CONTEXT.md` — 24 locked implementation decisions D-01 through D-24
- [Python base] `app/services/prompts/base.py` — PromptMetadata, BasePrompt, ModelType/OutputFormat enums
- [Python registry] `app/services/prompts/registry.py` — PromptRegistry 3-level index, search, default versions
- [Python template] `app/services/prompts/template.py` — TemplateRenderer str.replace + regex filter logic
- [Python manager] `app/services/prompts/manager.py` — PromptManager get_prompt, validate_output, search_prompts
- [Python validators] `app/services/prompts/validators.py` — PromptOutputValidator JSON/narration/plot validation
- [Python exceptions] `app/services/prompts/exceptions.py` — 6 prompt exception classes
- [Python video processor] `app/utils/video_processor.py` — 4-level fallback + fast path frame extraction
- [Python frame analysis service] `app/services/documentary/frame_analysis_service.py` — Full visual analysis orchestration
- [Python frame analysis models] `app/services/documentary/frame_analysis_models.py` — FrameAnalysisConfig, FrameBatchResult
- [Python frame analysis prompt] `app/services/prompts/documentary/frame_analysis.py` — Documentary frame analysis template
- [Python narration generation prompt] `app/services/prompts/documentary/narration_generation.py` — Narration template
- [Python plot extraction prompt] `app/services/prompts/short_drama_editing/plot_extraction.py` — SDE plot extraction template

### Secondary (MEDIUM confidence)
- [Phase 1] `.planning/phases/01-foundation/01-CONTEXT.md` — Crate structure, spawn_blocking, thiserror patterns
- [Phase 2] `.planning/phases/02-llm-service-layer/02-CONTEXT.md` — LlmProvider trait, image preprocessing, Semaphore
- [Existing Rust code] `src/ffmpeg/command.rs` — run_ffmpeg(), ProgressCallback, spawn_blocking pattern
- [Existing Rust code] `src/llm/openai_compatible.rs` — analyze_images() implementation, JSON fallback
- [Existing Rust code] `src/llm/image_utils.rs` — image_to_base64_data_url() with JPEG passthrough
- [Existing Rust code] `src/error.rs` — thiserror pattern, LLMError, FFmpegError, ConfigError
- [Existing Rust code] `src/config/types.rs` — FramesSection with frame_interval_input, vision_batch_size, vision_max_concurrency
- [Existing Rust code] `src/llm/registry.rs` — HashMap<String, Arc<dyn LlmProvider>> pattern
- [Existing Rust code] `src/llm/register.rs` — register_all_providers() pattern
- [Existing Rust code] `src/llm/types.rs` — LlmResponseFormat enum
- [Existing Rust code] `src/ffmpeg/hwaccel.rs` — Hardware acceleration detection and profiles
- [Existing Rust code] `Cargo.toml` — Current dependency versions

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all libraries verified in existing Cargo.toml or AI-SPEC; regex 1.11 is the standard Rust regex crate
- Architecture patterns: HIGH — patterns derived from confirmed working Python reference implementation + existing Phase 1-2 patterns
- Pitfalls: HIGH — four of five pitfalls are documented in AI-SPEC; the fifth (frame compatibility) is from Python codebase

**Research date:** 2026-04-30
**Valid until:** 2026-05-30 (30 days — stable tooling)
