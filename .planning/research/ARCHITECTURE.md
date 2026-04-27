# Architecture Patterns

**Domain:** AI video narration + automated editing (desktop application)
**Researched:** 2026-04-27
**Context:** NarratoAI Rust rewrite -- architecture design from Python v0.7.8

## Recommended Architecture

**Layered library architecture** with Tauri command facade. The Rust codebase is a standalone library (`narratoai` crate) that can be used independently or exposed via Tauri commands.

```
narratoai (library crate)
  |
  +-- config/          TOML loading, validation, defaults
  +-- models/          Data types (script, task, TTS, pipeline)
  +-- llm/             LLM service layer (OpenAI-compatible)
  +-- tts/             TTS provider abstraction + 7 engines
  +-- ffmpeg/          FFmpeg pipeline operations
  +-- prompts/         Versioned prompt registry
  +-- vision/          Frame extraction + analysis
  +-- pipeline/        3 business mode orchestrators
  +-- state/           Task state management (file-based)
  +-- export/          JianYing draft export
  |
tauri-app (binary crate, optional)
  |
  +-- commands/        #[tauri::command] functions
  +-- state/           Tauri State<> wrappers
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `config` | Load/validate TOML config, provide typed access | All components read config |
| `models` | Data structures for scripts, tasks, pipeline params | All components use models |
| `llm` | OpenAI-compatible API client, vision + text models | `prompts` (for prompt text), `vision` (for image payloads), `pipeline` (called by orchestrator) |
| `tts` | Text-to-speech provider routing, audio file output | `config` (engine credentials), `pipeline` (called by orchestrator) |
| `ffmpeg` | Video clip, merge, mix, subtitle burn-in, probe | `pipeline` (called by orchestrator), `state` (progress updates) |
| `prompts` | Versioned prompt registry, template rendering | `llm` (provides rendered prompts to LLM) |
| `vision` | Keyframe extraction, batch vision analysis | `ffmpeg` (frame extraction), `llm` (vision API calls) |
| `pipeline` | Orchestrate the 6-step video processing flow | `config`, `llm`, `tts`, `ffmpeg`, `prompts`, `state` |
| `state` | Task state persistence, progress tracking | `pipeline` (state updates), Tauri commands (progress reading) |
| `export` | JianYing draft JSON format generation | `models` (script data), `pipeline` (called after completion) |

### Data Flow

```
User Input (video file + config)
  |
  v
Pipeline Orchestrator
  |
  +-- Documentary Mode:
  |     Video -> Vision (frame extraction via FFmpeg)
  |           -> LLM Vision API (batch analysis)
  |           -> LLM Text API (narration script)
  |           -> Script JSON
  |
  +-- SDE Mode:
  |     Subtitle Text -> LLM Analysis -> Script with OST types
  |
  +-- SDP Mode:
  |     Subtitle Text -> LLM Plot Analysis -> Multi-clip Script
  |
  v
Script JSON (user can edit)
  |
  v
6-Step Pipeline:
  1. Load Script (JSON -> typed structs)
  2. TTS Generation (per-segment, parallel where possible)
  3. Video Clipping (FFmpeg, strategy driven by OST type)
  4. Audio/Subtitle Merge (FFmpeg amix + subtitles filter)
  5. Clip Concatenation (FFmpeg concat)
  6. Final Composition (BGM + subtitle burn-in)
  |
  v
Output: Final video file + JianYing draft (optional)
```

## Patterns to Follow

### Pattern 1: Trait-Based Provider Abstraction

**What:** Define traits for extensible provider systems (TTS engines, LLM providers).
**When:** Any subsystem with multiple interchangeable backends.
**Why:** Enables adding new engines/providers without modifying dispatch logic. Replaces Python's string-based dispatch.

```rust
// TTS provider trait
#[async_trait]
pub trait TtsProvider: Send + Sync {
    async fn synthesize(&self, text: &str, voice: &str, output_path: &Path) -> Result<TtsResult>;
    fn name(&self) -> &str;
    fn supported_voices(&self) -> Vec<VoiceInfo>;
}

// LLM provider trait
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn generate_text(&self, messages: Vec<Message>) -> Result<String>;
    async fn analyze_images(&self, images: Vec<ImageInput>, prompt: &str) -> Result<String>;
}
```

### Pattern 2: Builder for FFmpeg Commands

**What:** Use ffmpeg-sidecar's Builder API to construct FFmpeg commands step by step.
**When:** All FFmpeg operations (clip, merge, mix, subtitle burn-in).
**Why:** Readable, composable, avoids string concatenation errors.

```rust
use ffmpeg_sidecar::command::FfmpegCommand;

let output = FfmpegCommand::new()
    .input("input.mp4")
    .start_at(Duration::from_secs(10))
    .duration(Duration::from_secs(30))
    .args(["-c:v", "libx264", "-c:a", "aac"])
    .output("clip.mp4")
    .spawn()?
    .iter()?;
```

### Pattern 3: State Machine for Task Progress

**What:** Task state modeled as an enum with explicit transitions.
**When:** Task lifecycle management.
**Why:** Replaces Python's loose dict-based state with type-safe state transitions. Impossible to reach invalid states.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskState {
    Pending,
    GeneratingScript,
    GeneratingTts { segment: usize, total: usize },
    ClippingVideo { segment: usize, total: usize },
    MergingAudio,
    Concatenating,
    FinalComposition,
    Completed { output_path: PathBuf },
    Failed { error: String },
}
```

### Pattern 4: Pipeline Step Trait

**What:** Each pipeline step implements a common trait for uniform orchestration.
**When:** 6-step pipeline and sub-pipelines.
**Why:** Steps can be tested independently, reordered, or skipped.

```rust
#[async_trait]
pub trait PipelineStep {
    type Input;
    type Output;
    async fn execute(&self, input: Self::Input, ctx: &PipelineContext) -> Result<Self::Output>;
    fn name(&self) -> &str;
}
```

### Pattern 5: Typed OST Handling

**What:** OST type as a Rust enum with associated behavior, not an integer constant.
**When:** All pipeline steps that branch on OST type.
**Why:** Eliminates Python's fragile integer-based branching (OST=0/1/2). Compiler enforces exhaustive matching.

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OstType {
    NarrationOnly,   // OST=0: narration only, remove original audio
    OriginalOnly,    // OST=1: original audio only, no TTS
    Mixed,           // OST=2: mix narration + original audio
}

impl OstType {
    pub fn needs_tts(&self) -> bool {
        matches!(self, OstType::NarrationOnly | OstType::Mixed)
    }

    pub fn needs_original_audio(&self) -> bool {
        matches!(self, OstType::OriginalOnly | OstType::Mixed)
    }
}
```

### Pattern 6: Concurrency with Semaphore

**What:** Use tokio::sync::Semaphore to limit concurrent LLM vision API calls.
**When:** Vision frame analysis (batch processing of keyframes).
**Why:** Prevents API rate limiting. Python has `vision_max_concurrency` config for this purpose.

```rust
let semaphore = Arc::new(Semaphore::new(config.vision_max_concurrency));
let handles: Vec<_> = frames.into_iter().map(|frame| {
    let sem = semaphore.clone();
    tokio::spawn(async move {
        let _permit = sem.acquire().await.unwrap();
        llm.analyze_images(vec![frame], prompt).await
    })
}).collect();
```

## Anti-Patterns to Avoid

### Anti-Pattern 1: String-Based Dispatch

**What:** Dispatching to TTS engines or LLM providers by string name (like Python's `voice.py`).
**Why bad:** No compile-time checking. Typos cause runtime failures. Adding new engines requires modifying the dispatch function.
**Instead:** Use trait objects in a HashMap. Registration at startup, dispatch via trait method call.

### Anti-Pattern 2: God Object Pipeline

**What:** One massive function that handles all 6 pipeline steps (like Python's `start_subclip_unified`).
**Why bad:** 600+ lines of entangled logic. Hard to test, hard to modify, hard to understand.
**Instead:** Each step is a separate struct implementing `PipelineStep`. Orchestrator calls steps in sequence, passing typed data between them.

### Anti-Pattern 3: Untyped Configuration Access

**What:** Accessing config values as `config["app"]["vision_model"]` with string keys.
**Why bad:** No validation, no autocomplete, typos are runtime errors.
**Instead:** Deserialize TOML into a strongly-typed struct with serde. Validate on load.

```rust
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub app: AppSection,
    pub ui: UiSection,
    #[serde(default)]
    pub azure: Option<AzureSection>,
    // ... other sections
}
```

### Anti-Pattern 4: Blocking in Async Context

**What:** Calling blocking FFmpeg subprocess operations on the tokio runtime.
**Why bad:** Blocks the async runtime, prevents progress updates, freezes UI.
**Instead:** Use `tokio::task::spawn_blocking` for CPU-bound or blocking IO, or use ffmpeg-sidecar's async-compatible spawn with iterator interface.

### Anti-Pattern 5: Global Mutable State

**What:** Module-level mutable statics or lazy_static with Mutex for state.
**Why bad:** Testing becomes difficult, order-dependent behavior, hidden coupling.
**Instead:** Pass state explicitly through function parameters or use Tauri's `State` management. For task state, use a `TaskManager` struct owned by the application.

## Module Organization

```
src/
  lib.rs              -- Re-exports, crate root
  config/
    mod.rs            -- Config loading, validation
    types.rs          -- Typed config structs (derive Deserialize)
    defaults.rs       -- Default values
  models/
    mod.rs            -- Re-exports
    script.rs         -- Script JSON structures
    task.rs           -- Task state enum, task info
    tts.rs            -- TTS request/response types
    video.rs          -- Video params, aspect ratio, OST type
  llm/
    mod.rs            -- LlmProvider trait, public API
    openai.rs         -- OpenAI-compatible implementation
    types.rs          -- Message, Content, ImageInput types
  tts/
    mod.rs            -- TtsProvider trait, TtsRouter
    edge_tts.rs       -- Edge-TTS WebSocket implementation
    azure.rs          -- Azure Speech REST
    tencent.rs        -- Tencent Cloud TTS
    soulvoice.rs      -- SoulVoice API
    qwen.rs           -- DashScope Qwen TTS
    indextts2.rs      -- IndexTTS2 voice clone
    doubao.rs         -- Doubao TTS
  ffmpeg/
    mod.rs            -- FFmpeg operations facade
    clip.rs           -- Video clipping
    merge.rs          -- Audio/subtitle merge
    concat.rs         -- Clip concatenation
    mix.rs            -- Audio mixing
    subtitle.rs       -- Subtitle generation + burn-in
    probe.rs          -- Media file metadata via ffprobe
  prompts/
    mod.rs            -- PromptRegistry, PromptManager
    types.rs          -- BasePrompt, PromptVersion
    documentary.rs    -- Documentary mode prompts
    sde.rs            -- SDE mode prompts
    sdp.rs            -- SDP mode prompts
  vision/
    mod.rs            -- Frame analysis service
    extractor.rs      -- Keyframe extraction via FFmpeg
  pipeline/
    mod.rs            -- Pipeline orchestrator, PipelineStep trait
    documentary.rs    -- Documentary mode orchestrator
    sde.rs            -- SDE mode orchestrator
    sdp.rs            -- SDP mode orchestrator
    steps.rs          -- Individual pipeline step implementations
  state/
    mod.rs            -- TaskManager, file-based state
  export/
    mod.rs            -- Export trait
    jianying.rs       -- JianYing draft format
```

## Scalability Considerations

| Concern | At 1 user (desktop) | At concurrent tasks | At large videos |
|---------|---------------------|---------------------|-----------------|
| Memory | FFmpeg subprocess is primary memory consumer. ffmpeg-sidecar streams output, doesn't buffer entire video. | Single-user desktop = 1 task at a time. If future multi-task needed, add task queue. | Use streaming FFmpeg operations. Never load entire video into memory. |
| CPU | FFmpeg uses its own threads. Rust async runtime is lightweight. LLM/TTS are network-bound. | tokio handles concurrent I/O naturally. CPU-bound work goes to spawn_blocking. | FFmpeg handles its own threading. No special Rust-side optimization needed. |
| Disk | Temporary files for intermediate clips/audio. tempfile crate auto-cleans. | Sequential tasks reuse temp directory. Clean between tasks. | Large temp files possible. Clean up aggressively after each pipeline step completes. |
| Network | LLM and TTS API calls are rate-limited by providers. | Semaphore-based concurrency control. Configurable max concurrent requests. | Batch vision analysis with semaphore. Stream large audio downloads. |

## Tauri Command Layer

The Tauri command layer is a thin facade over the library. It does NOT contain business logic.

```rust
// tauri-app/src/commands/task.rs
#[tauri::command]
async fn start_documentary(
    state: State<'_, AppState>,
    video_path: String,
    config: DocumentaryConfig,
) -> Result<String, String> {
    let task_id = state.pipeline_manager
        .start_documentary(video_path, config)
        .await
        .map_err(|e| e.to_string())?;
    Ok(task_id)
}

#[tauri::command]
async fn get_task_progress(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<TaskProgress, String> {
    state.task_manager
        .get_progress(&task_id)
        .map_err(|e| e.to_string())
}
```

**State management in Tauri:**

```rust
pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub task_manager: TaskManager,
    pub pipeline_manager: PipelineManager,
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::new())
        .invoke_handler(generate_handler![
            commands::start_documentary,
            commands::start_sde,
            commands::start_sdp,
            commands::get_task_progress,
            commands::load_config,
            commands::save_config,
            commands::load_script,
            commands::save_script,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Progress streaming via Tauri Channel:**

```rust
#[tauri::command]
async fn start_pipeline(
    app: AppHandle,
    channel: Channel<PipelineProgress>,
    task_id: String,
) -> Result<(), String> {
    // Pipeline sends progress events through the channel
    // Frontend receives them in real-time
}
```

## Sources

- `.planning/codebase/ARCHITECTURE.md` -- Python architecture mapping
- `.planning/codebase/STRUCTURE.md` -- Directory layout and file inventory
- `.planning/codebase/CONCERNS.md` -- Technical debt and pain points
- `.planning/PROJECT.md` -- Architecture constraints (library + Tauri command)
- docs.rs: tauri (command patterns, State, Channel API)
- docs.rs: ffmpeg-sidecar (Builder pattern, Iterator interface)
- docs.rs: tokio (Semaphore, spawn_blocking)
- Python source: `app/services/task.py` -- Pipeline orchestrator analysis
- Python source: `app/services/voice.py` -- TTS routing pattern
- Python source: `app/services/llm/` -- LLM provider architecture
