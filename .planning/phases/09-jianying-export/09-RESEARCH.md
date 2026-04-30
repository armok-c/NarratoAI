# Phase 9: JianYing Export - Research

**Researched:** 2026-04-29
**Domain:** pyJianYingDraft v0.2.6 源码逆向 + 剪映草稿 JSON 格式
**Confidence:** HIGH

## Summary

通过完整阅读 pyJianYingDraft v0.2.6 的全部核心源码（`draft_folder.py`、`script_file.py`、`segment.py`、`video_segment.py`、`audio_segment.py`、`track.py`、`time_util.py`、`local_materials.py`、`animation.py`、`keyframe.py`、`util.py`、`exceptions.py`）以及两个 JSON 模板文件（`draft_content_template.json`、`draft_meta_info.json`），完整逆向了剪映草稿 JSON 格式规范。

剪映草稿由一个文件夹组成，内含 `draft_content.json`（主内容）和 `draft_meta_info.json`（元信息）。主内容的核心结构是 `tracks`（轨道数组）和 `materials`（素材集合），轨道中的每个 segment 通过 `material_id` 引用 materials 中的素材。时间单位统一为**微秒**（1秒 = 1,000,000 微秒），`Timerange` 结构包含 `start` 和 `duration` 两个整数微秒值。

**Primary recommendation:** 在 Rust 中使用 serde 构建与剪映 JSON 格式一一映射的结构体体系，builder API 模式参照 pyJianYingDraft 的 `DraftFolder` -> `ScriptFile` -> `Track` -> `Segment` 层级，但不需要复刻全部功能（特效、滤镜、蒙版、动画、贴纸、文本等），仅实现 NarratoAI 实际使用的 video/audio 两种轨道和基础 segment。

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** 通过 pyJianYingDraft 源码逆向获取剪映草稿 JSON 格式规范。在 Rust 中直接生成相同格式的 JSON。
- **D-02:** 跟随 pyJianYingDraft 最新版本（v0.2.6），确保与当前剪映专业版兼容。
- **D-03:** 完整复制 pyJianYingDraft API——在 Rust 中实现对等的 `DraftFolder`、`VideoSegment`、`AudioSegment`、`TrackType`、`trange` 等 builder 类型。
- **D-04:** 导出函数仅接收处理后的数据。不做 TTS 生成和视频裁剪。
- **D-05:** 新建 `ExportRequest` 结构体作为导出函数的输入参数。
- **D-06:** 导出时严格校验 Option 字段。
- **D-07:** 视频轨道 + 音频轨道双轨布局。
- **D-08:** OST 类型映射与 Python 版 1:1 对齐。
- **D-09:** 视频片段来源智能回退——有处理后的视频文件则用它，没有则用原始视频路径 + source_time_range。
- **D-10:** 音频片段时长使用 ffprobe 精确获取。
- **D-11:** 分辨率配置化 + 默认 1080p。
- **D-12:** 草稿命名采用用户指定 + 自动回退 `NarratoAI_{timestamp}`。
- **D-13:** 帧率使用剪映默认值（30fps）。
- **D-14:** 草稿保存路径由调用方传入。

### Claude's Discretion
- ExportRequest 结构体的具体字段设计
- JianYingError 枚举的具体变体定义
- builder API 的方法命名和调用风格
- 模块内部文件拆分方式

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| JYNG-01 | 生成剪映草稿 JSON 格式（逆向工程 pyJianYingDraft 格式） | 完整逆向了 pyJianYingDraft v0.2.6 全部核心源码，JSON 格式规范已文档化 |
| JYNG-02 | 导出项目时间线——片段、字幕、音频轨道映射到剪映格式 | VideoSegment/AudioSegment 的 JSON 映射关系已完整记录，track/segment/materials 三层结构已理清 |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| JSON 草稿生成 | Rust 库层 (src/jianying/) | -- | 纯数据转换，无 UI 依赖 |
| 音频时长探测 | FFmpeg 层 (src/ffmpeg/probe.rs) | -- | 已实现 ffprobe 能力，复用即可 |
| 文件系统写入 | Rust 库层 (src/jianying/) | -- | 创建草稿文件夹 + 写入 JSON 文件 |
| 脚本数据输入 | Script 层 (src/script/types.rs) | -- | ScriptClip 作为导出数据源 |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| serde | 1.0.228 | JSON 序列化/反序列化 | 已在项目中使用，Rust 生态标准 |
| serde_json | 1.0.140 | JSON 生成 | 已在项目中使用 |
| uuid | -- | 生成素材/片段全局 ID | pyJianYingDraft 使用 uuid4 生成所有 ID [VERIFIED: 源码] |
| thiserror | 2.0.18 | 错误类型派生 | 已在项目中使用 |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| ffmpeg-sidecar | 2.5.1 | ffprobe 音频时长探测 | D-10 要求 ffprobe 获取音频时长 |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| uuid crate | 手动 hex 随机 | uuid crate 更标准，但剪映只要求 32 位 hex，自定义实现也足够 |

**Installation:**
```bash
cargo add uuid --features v4
```

**Version verification:** uuid crate 是 Rust 生态标准库，添加最新版本即可。其他依赖已在 Cargo.toml 中。

## Architecture Patterns

### System Architecture Diagram

```
ExportRequest
    |
    v
DraftFolder::create_draft(name, width, height)
    |
    +--> 创建草稿文件夹
    +--> 复制 draft_meta_info.json 模板
    +--> 创建 ScriptFile(width, height, fps)
    |
    v
ScriptFile
    |
    +--> add_track(TrackType::Video, "视频轨道")
    +--> add_track(TrackType::Audio, "音频轨道")
    |
    v
遍历 ScriptClip[]:
    |
    +--> [OST=0/2] 创建 VideoSegment + AudioSegment
    +--> [OST=1]   仅创建 VideoSegment
    |
    +--> VideoSegment 构造:
    |    |-- 有 clip.video --> VideoSegment(video_path, trange)
    |    |-- 无 clip.video --> VideoSegment(origin_path, trange, source_timerange=trange)
    |
    +--> AudioSegment 构造:
         |-- ffprobe 获取音频时长
         |-- safe_duration = min(audio_duration, video_duration)
         |-- AudioSegment(audio_path, trange)
    |
    v
ScriptFile.save()
    |
    +--> 收集所有 materials (videos, audios, speeds)
    +--> 排序 tracks by render_index
    +--> 序列化为 JSON
    +--> 写入 draft_content.json
```

### Recommended Project Structure
```
src/jianying/
├── mod.rs           # 模块入口，pub 导出 DraftFolder, ExportRequest 等
├── types.rs         # 剪映 JSON 结构体（serde 序列化用）
├── builder.rs       # DraftFolder + ScriptFile builder API
├── segment.rs       # VideoSegment, AudioSegment 构建
├── material.rs      # VideoMaterial, AudioMaterial, Speed, CropSettings
├── time.rs          # Timerange, trange(), tim(), SEC 常量
├── track.rs         # TrackType 枚举, Track 结构体
├── template.rs      # JSON 模板常量（draft_content_template, draft_meta_info_template）
└── error.rs         # JianYingError 枚举
```

### Pattern 1: Timerange 时间范围表示
**What:** 剪映 JSON 中所有时间值使用**微秒**（整数），`Timerange` 由 `start` + `duration` 组成
**When to use:** 所有 segment 的 target_timerange 和 source_timerange
**Example:**
```rust
// Source: pyJianYingDraft/time_util.py Timerange class
const SEC: i64 = 1_000_000; // 一秒 = 1,000,000 微秒

/// 时间范围——start 和 duration 均为微秒
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timerange {
    pub start: i64,
    pub duration: i64,
}

/// trange 便捷构造函数——接受 "1.5s" 格式的字符串
pub fn trange(start: &str, duration: &str) -> Timerange {
    Timerange {
        start: parse_time(start),
        duration: parse_time(duration),
    }
}

/// 解析时间字符串为微秒，支持 "1h52m3s", "0.15s" 格式
fn parse_time(input: &str) -> i64 {
    // 实现参考 pyJianYingDraft/time_util.py tim() 函数
    // 简化版只需支持 "Ns" 格式（N 为浮点数）
    let input = input.trim().to_lowercase();
    if input.ends_with('s') {
        let secs: f64 = input.trim_end_matches('s').parse().unwrap();
        (secs * SEC as f64).round() as i64
    } else {
        input.parse().unwrap_or(0)
    }
}
```

### Pattern 2: Material 素材注册
**What:** 每个 VideoSegment/AudioSegment 构造时创建对应的 VideoMaterial/AudioMaterial，添加到 ScriptFile.materials 中
**When to use:** segment 添加到轨道时自动注册素材
**Example:**
```rust
// Source: pyJianYingDraft/local_materials.py
// VideoMaterial.export_json() 生成的 JSON 结构
{
    "audio_fade": null,
    "category_id": "",
    "category_name": "local",
    "check_flag": 63487,
    "crop": {
        "upper_left_x": 0.0, "upper_left_y": 0.0,
        "upper_right_x": 1.0, "upper_right_y": 0.0,
        "lower_left_x": 0.0, "lower_left_y": 1.0,
        "lower_right_x": 1.0, "lower_right_y": 1.0
    },
    "crop_ratio": "free",
    "crop_scale": 1.0,
    "duration": 5000000,     // 微秒
    "height": 1080,
    "id": "abc123...",       // UUID hex
    "local_material_id": "",
    "material_id": "abc123...",
    "material_name": "video.mp4",
    "media_path": "",
    "path": "C:\\full\\path\\to\\video.mp4",
    "type": "video",         // 或 "photo"
    "width": 1920
}

// AudioMaterial.export_json() 生成的 JSON 结构
{
    "app_id": 0,
    "category_id": "",
    "category_name": "local",
    "check_flag": 3,
    "copyright_limit_type": "none",
    "duration": 3500000,     // 微秒
    "effect_id": "",
    "formula_id": "",
    "id": "def456...",
    "local_material_id": "def456...",
    "music_id": "def456...",
    "name": "audio.mp3",
    "path": "C:\\full\\path\\to\\audio.mp3",
    "source_platform": 0,
    "type": "extract_music",
    "wave_points": []
}
```

### Pattern 3: Track JSON 输出
**What:** 轨道导出为 JSON 时的完整结构
**When to use:** ScriptFile.dumps() 时对每个 Track 调用 export_json()
**Example:**
```rust
// Source: pyJianYingDraft/track.py Track.export_json()
{
    "attribute": 0,          // int(mute)，0=不静音
    "flag": 0,
    "id": "track_uuid_hex",
    "is_default_name": false, // len(name) == 0
    "name": "视频轨道",
    "segments": [ /* segment export_json() 数组 */ ],
    "type": "video"          // TrackType 枚举的 name
}
```

### Pattern 4: Segment 基础 JSON 结构
**What:** 所有 segment 共享的基础字段，由 BaseSegment.export_json() + MediaSegment.export_json() 组合
**When to use:** VideoSegment 和 AudioSegment 的 JSON 输出都基于此
**Example:**
```rust
// Source: pyJianYingDraft/segment.py
// BaseSegment.export_json() 生成:
{
    "enable_adjust": true,
    "enable_color_correct_adjust": false,
    "enable_color_curves": true,
    "enable_color_match_adjust": false,
    "enable_color_wheels": true,
    "enable_lut": true,
    "enable_smart_color_adjust": false,
    "last_nonzero_volume": 1.0,
    "reverse": false,
    "track_attribute": 0,
    "track_render_index": 0,
    "visible": true,
    "id": "segment_uuid_hex",
    "material_id": "material_uuid_hex",
    "target_timerange": {"start": 0, "duration": 5000000},
    "common_keyframes": [],
    "keyframe_refs": []
}

// MediaSegment.export_json() 在此基础上追加:
{
    // ... BaseSegment 字段 ...
    "source_timerange": {"start": 0, "duration": 5000000}, // 或 null
    "speed": 1.0,
    "volume": 1.0,
    "extra_material_refs": ["speed_uuid_hex"],
    "is_tone_modify": false
}
```

### Pattern 5: VideoSegment 完整 JSON 输出
**What:** VideoSegment = VisualSegment + 额外字段
**Example:**
```rust
// Source: pyJianYingDraft/video_segment.py VideoSegment.export_json()
// VisualSegment.export_json() 在 MediaSegment 基础上追加:
{
    // ... MediaSegment 字段 ...
    "clip": {
        "alpha": 1.0,
        "flip": {"horizontal": false, "vertical": false},
        "rotation": 0.0,
        "scale": {"x": 1.0, "y": 1.0},
        "transform": {"x": 0.0, "y": 0.0}
    },
    "uniform_scale": {"on": true, "value": 1.0},
}

// VideoSegment.export_json() 在 VisualSegment 基础上追加:
{
    // ... VisualSegment 字段 ...
    "hdr_settings": {"intensity": 1.0, "mode": 1, "nits": 1000},
}
```

### Pattern 6: AudioSegment 完整 JSON 输出
**What:** AudioSegment 在 MediaSegment 基础上仅追加两个 null 字段
**Example:**
```rust
// Source: pyJianYingDraft/audio_segment.py AudioSegment.export_json()
{
    // ... MediaSegment 字段 ...
    "clip": null,
    "hdr_settings": null
}
```

### Pattern 7: Speed 对象
**What:** 每个 segment 伴随一个 Speed 素材对象，通过 extra_material_refs 关联
**Example:**
```rust
// Source: pyJianYingDraft/segment.py Speed.export_json()
{
    "curve_speed": null,
    "id": "speed_uuid_hex",
    "mode": 0,
    "speed": 1.0,
    "type": "speed"
}
```

### Anti-Patterns to Avoid
- **直接手动拼接 JSON 字符串:** 必须使用 serde 结构体 + serde_json 序列化，避免格式错误
- **忽略 extra_material_refs:** Speed 对象的 ID 必须出现在 segment 的 extra_material_refs 数组中，否则剪映无法正确解析播放速度
- **使用浮点秒数而非微秒整数:** 剪映 JSON 中所有时间值都是整数微秒，不是浮点秒
- **遗漏 draft_content_template.json 中的固定字段:** 如 `version: 360000`、`color_space: 0`、`platform.app_id: 3704` 等，这些是剪映识别草稿的必要字段

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| UUID 生成 | 自定义 hex 随机 | uuid crate (v4) | pyJianYingDraft 使用 uuid4，Rust uuid crate 的 v4 feature 提供等价功能 |
| JSON 序列化 | 手动 JSON 拼接 | serde + serde_json | 避免转义错误、格式问题、嵌套结构出错 |
| 音频时长探测 | 手动解析文件头 | ffprobe（已有） | 已在 Phase 1 实现 probe_video，可扩展支持音频 |

**Key insight:** pyJianYingDraft 使用 pymediainfo 获取素材时长和分辨率。Rust 版不需要引入 mediainfo 依赖——视频素材的时长/分辨率可以从 ScriptClip 的已有字段（duration、source_time_range）推算，音频时长通过已有的 ffprobe 获取。仅在需要创建 VideoMaterial 时需要知道素材的 width/height，这可以从 ExportRequest 的分辨率参数获取（因为裁剪后的视频片段分辨率等于输出分辨率）。

## Common Pitfalls

### Pitfall 1: 时间单位混淆
**What goes wrong:** 剪映 JSON 使用微秒（整数），而 NarratoAI ScriptClip 的 duration 是浮点秒
**Why it happens:** pyJianYingDraft 定义 `SEC = 1_000_000`，所有内部计算都用微秒。Python 版通过 `trange("1.5s", "7.441s")` 隐式转换
**How to avoid:** Rust 实现中必须显式将 `f64` 秒转换为 `i64` 微秒：`(secs * 1_000_000.0).round() as i64`
**Warning signs:** 剪映打开后片段时长异常（过长或过短 1,000,000 倍）

### Pitfall 2: material_id 引用断裂
**What goes wrong:** segment 的 `material_id` 与 materials 列表中的素材 `id` 不匹配
**Why it happens:** 每个 segment 的 `material_id` 指向 VideoMaterial/AudioMaterial 的 `material_id` 字段，Speed 的 `id` 也要出现在 `extra_material_refs` 中
**How to avoid:** 使用统一的 UUID 生成策略，segment 构造时保存 material_id 引用
**Warning signs:** 剪映打开后显示空白片段或素材加载失败

### Pitfall 3: 主视频轨道必须从 0s 开始
**What goes wrong:** 剪映强制将主视频轨道（最底层视频轨道）的第一个片段对齐至 0s
**Why it happens:** 剪映的 `maintrack_adsorb` 行为
**How to avoid:** 确保 video track 的第一个 segment 的 target_timerange.start = 0。这是默认行为（current_time 从 0 开始），不需要特殊处理
**Warning signs:** 剪映打开后片段位置偏移

### Pitfall 4: 路径格式必须为绝对路径
**What goes wrong:** 剪映无法找到素材文件
**Why it happens:** pyJianYingDraft 在 VideoMaterial/AudioMaterial 构造函数中使用 `os.path.abspath(path)` 转换为绝对路径
**How to avoid:** Rust 实现中 path 字段必须使用绝对路径，可以用 `std::fs::canonicalize()` 或 `dunce::canonicalize()` (Windows)
**Warning signs:** 剪映打开后素材显示为离线/缺失

### Pitfall 5: render_index 决定轨道层叠顺序
**What goes wrong:** 轨道层叠顺序错误（音频显示在视频上方）
**Why it happens:** 剪映通过 `render_index` 排序轨道，video 默认为 0，audio 默认为 0，排序后按添加顺序层叠
**How to avoid:** tracks 按 render_index 排序后输出。pyJianYingDraft 中 video 的 render_index=0，audio 的 render_index=0，但由于 tracks 排序是稳定排序，先添加的 video 在底层
**Warning signs:** 剪映打开后轨道顺序不对

### Pitfall 6: Speed 素材遗漏
**What goes wrong:** 剪映无法正确播放变速片段
**Why it happens:** 每个 MediaSegment 自动创建一个 Speed 对象，其 ID 必须同时出现在 segment 的 `extra_material_refs` 和 materials.speeds 列表中
**How to avoid:** 即使 speed=1.0 也必须生成 Speed 素材对象，不能省略
**Warning signs:** 剪映播放异常或导出失败

### Pitfall 7: JSON 模板固定字段不能省略
**What goes wrong:** 剪映无法识别草稿文件
**Why it happens:** `draft_content_template.json` 包含大量剪映版本识别字段（如 `version: 360000`、`platform.app_version: "5.9.0"`）
**How to avoid:** Rust 实现必须完整复制模板中的所有固定字段
**Warning signs:** 剪映打开时报错"草稿格式不兼容"或直接闪退

## Code Examples

### 剪映草稿文件夹结构
```
JianYingDraftPath/
└── NarratoAI_1745900000/          # 草稿名称
    ├── draft_content.json          # 主内容（时间线、轨道、素材）
    └── draft_meta_info.json        # 元信息（草稿 ID、云端状态等）
```
[VERIFIED: pyJianYingDraft/draft_folder.py create_draft() 方法]

### draft_content.json 顶层结构
```json
{
    "canvas_config": { "height": 1080, "ratio": "original", "width": 1920 },
    "color_space": 0,
    "config": {
        "maintrack_adsorb": true,
        "video_mute": false,
        // ... 其他 config 字段保持模板默认值
    },
    "duration": 60000000,
    "fps": 30.0,
    "id": "91E08AC5-22FB-47e2-9AA0-7DC300FAEA2B",
    "materials": {
        "audios": [ /* AudioMaterial JSON 数组 */ ],
        "speeds": [ /* Speed JSON 数组 */ ],
        "videos": [ /* VideoMaterial JSON 数组 */ ],
        "audio_effects": [],
        "audio_fades": [],
        "canvases": [],
        "effects": [],
        "masks": [],
        "material_animations": [],
        "transitions": [],
        "video_effects": [],
        // ... 其他素材类型保持空数组
    },
    "tracks": [ /* Track JSON 数组，按 render_index 排序 */ ],
    "version": 360000
}
```
[VERIFIED: pyJianYingDraft/assets/draft_content_template.json + script_file.py dumps()]

### Rust serde 结构体设计示例
```rust
use serde::{Serialize, Deserialize};

/// 时间范围（微秒）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timerange {
    pub start: i64,
    pub duration: i64,
}

/// TrackType 枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrackType {
    Video,
    Audio,
}

impl TrackType {
    pub fn as_str(&self) -> &str {
        match self {
            TrackType::Video => "video",
            TrackType::Audio => "audio",
        }
    }

    pub fn render_index(&self) -> i32 {
        match self {
            TrackType::Video => 0,
            TrackType::Audio => 0,
        }
    }
}

/// Speed 素材
#[derive(Debug, Clone, Serialize)]
pub struct Speed {
    #[serde(rename = "curve_speed")]
    pub curve_speed: Option<()>,
    pub id: String,
    pub mode: u32,
    pub speed: f64,
    #[serde(rename = "type")]
    pub type_field: String,
}

/// Segment 基础 JSON（合并 BaseSegment + MediaSegment 字段）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SegmentJson {
    pub enable_adjust: bool,
    pub enable_color_correct_adjust: bool,
    pub enable_color_curves: bool,
    pub enable_color_match_adjust: bool,
    pub enable_color_wheels: bool,
    pub enable_lut: bool,
    pub enable_smart_color_adjust: bool,
    pub last_nonzero_volume: f64,
    pub reverse: bool,
    pub track_attribute: u32,
    pub track_render_index: u32,
    pub visible: bool,
    pub id: String,
    pub material_id: String,
    pub target_timerange: Timerange,
    pub common_keyframes: Vec<serde_json::Value>,
    pub keyframe_refs: Vec<serde_json::Value>,
    // MediaSegment 字段
    pub source_timerange: Option<Timerange>,
    pub speed: f64,
    pub volume: f64,
    pub extra_material_refs: Vec<String>,
    pub is_tone_modify: bool,
}
```

### 导出核心流程示例
```rust
// 对应 Python 版 jianying_task.py 的导出循环
pub fn export_draft(req: &ExportRequest) -> Result<PathBuf, JianYingError> {
    let mut draft = DraftFolder::create_draft(
        &req.draft_path,
        &req.draft_name,
        req.width,
        req.height,
    )?;

    draft.add_track(TrackType::Video, "视频轨道");
    draft.add_track(TrackType::Audio, "音频轨道");

    let mut current_time_us: i64 = 0;

    for clip in &req.script {
        let duration_secs = clip.duration.ok_or(/* ... */)?;
        let duration_us = (duration_secs * 1_000_000.0).round() as i64;
        let target = Timerange { start: current_time_us, duration: duration_us };

        // 视频片段
        if let Some(ref video_path) = clip.video {
            let video_seg = VideoSegment::new(video_path, target.clone())?;
            draft.add_segment(video_seg, "视频轨道")?;
        } else {
            let source_tr = parse_source_timerange(clip)?;
            let video_seg = VideoSegment::with_source_timerange(
                &req.video_origin_path, target.clone(), source_tr
            )?;
            draft.add_segment(video_seg, "视频轨道")?;
        }

        // 音频片段（OST=0 或 OST=2）
        if clip.ost == OstType::NarrationOnly || clip.ost == OstType::Mixed {
            if let Some(ref audio_path) = clip.audio {
                let audio_duration = probe_audio_duration(audio_path)?;
                let safe_duration_us = duration_us.min(
                    (audio_duration * 1_000_000.0).round() as i64
                );
                let audio_target = Timerange {
                    start: current_time_us,
                    duration: safe_duration_us,
                };
                let audio_seg = AudioSegment::new(audio_path, audio_target)?;
                draft.add_segment(audio_seg, "音频轨道")?;
            }
        }

        current_time_us += duration_us;
    }

    let draft_path = draft.save()?;
    Ok(draft_path)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| snake_case 类名 (Draft_folder) | PascalCase (DraftFolder) | pyJianYingDraft ~0.2.x | API 命名统一为 PascalCase |
| 单文件 script_file.py | 模块化拆分 | pyJianYingDraft ~0.2.x | 更清晰的代码组织 |

**Deprecated/outdated:**
- `Script_file`、`Draft_folder` 等 snake_case 别名：v0.2.6 仍保留但标记 deprecated，Rust 实现不需要兼容

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | 剪映专业版兼容 draft_content_template.json 中的 `version: 360000` 和 `platform.app_version: "5.9.0"` | Architecture Patterns | 剪映可能更新版本号要求 |
| A2 | VideoMaterial 不需要通过 pymediainfo 获取素材实际 width/height，可使用 ExportRequest 的分辨率 | Don't Hand-Roll | 如果剪映校验素材尺寸与实际不符会报错 |
| A3 | `draft_meta_info.json` 中的固定 `draft_id`（BC69C7CD-...）不需要每次生成新 UUID | Code Examples | 剪映可能要求每个草稿有唯一 ID |

**A2 详细说明:** pyJianYingDraft 使用 pymediainfo 读取视频文件的实际 width/height/duration。Rust 版有两个选择：(a) 使用 ffprobe 获取这些信息（已有能力），(b) 对于裁剪后的视频片段使用输出分辨率作为近似值。推荐 (a) 方案——复用 Phase 1 的 probe_video。

**A3 详细说明:** pyJianYingDraft 的模板中 `draft_id` 是固定值。但草稿应该是唯一的，建议 Rust 实现中为每个草稿生成新的 UUID。

## Open Questions

1. **VideoMaterial 的 width/height 是否必须精确**
   - What we know: pyJianYingDraft 使用 pymediainfo 读取实际尺寸
   - What's unclear: 剪映是否校验素材尺寸与实际文件一致
   - Recommendation: 使用 ffprobe 获取精确值，与 Python 版行为一致

2. **draft_id 是否需要唯一**
   - What we know: 模板中是固定 UUID
   - What's unclear: 同一剪映目录下多个草稿使用相同 draft_id 是否冲突
   - Recommendation: 每次创建草稿生成新 UUID，安全起见

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| ffprobe | 音频时长探测 (D-10) | 已实现 | Phase 1 probe_video | -- |
| serde + serde_json | JSON 序列化 | 已有 | 1.0.228 / 1.0.140 | -- |
| uuid (v4) | 素材 ID 生成 | 需添加 | -- | 手动 hex 随机 |
| std::fs | 文件创建/写入 | 标准库 | -- | -- |

**Missing dependencies with no fallback:**
- uuid crate: 需添加到 Cargo.toml

**Missing dependencies with fallback:**
- uuid: 可使用自定义 hex 随机生成器作为 fallback（pyJianYingDraft 只使用 uuid4 的 hex 输出）

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | pytest (Rust: cargo test) |
| Config file | Cargo.toml [dev-dependencies] |
| Quick run command | `cargo test -p narratoai-core jianying` |
| Full suite command | `cargo test -p narratoai-core` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| JYNG-01 | Timerange 序列化为 {"start":N,"duration":N} | unit | `cargo test trange_export_json` | Wave 0 |
| JYNG-01 | VideoSegment 完整 JSON 输出结构正确 | unit | `cargo test video_segment_export_json` | Wave 0 |
| JYNG-01 | AudioSegment 完整 JSON 输出结构正确 | unit | `cargo test audio_segment_export_json` | Wave 0 |
| JYNG-01 | ScriptFile 完整 draft_content.json 输出 | integration | `cargo test script_file_dumps` | Wave 0 |
| JYNG-01 | draft_meta_info.json 模板正确写入 | integration | `cargo test draft_meta_info` | Wave 0 |
| JYNG-02 | OST=0 正确添加视频+音频双轨 | unit | `cargo test ost_narration_only` | Wave 0 |
| JYNG-02 | OST=1 仅添加视频片段 | unit | `cargo test ost_original_sound` | Wave 0 |
| JYNG-02 | OST=2 正确添加视频+音频双轨 | unit | `cargo test ost_mixed` | Wave 0 |
| JYNG-02 | 视频片段回退到原始路径+source_timerange | unit | `cargo test video_fallback_source_range` | Wave 0 |
| JYNG-02 | 音频安全时长 min(音频时长, 视频时长) | unit | `cargo test safe_audio_duration` | Wave 0 |

### Sampling Rate
- **Per task commit:** `cargo test -p narratoai-core jianying`
- **Per wave merge:** `cargo test -p narratoai-core`
- **Phase gate:** Full suite green + 手动在剪映中验证生成的草稿

### Wave 0 Gaps
- [ ] `src/jianying/` 模块目录 — 需创建
- [ ] `src/jianying/error.rs` — JianYingError 枚举
- [ ] `src/jianying/time.rs` — Timerange, trange, parse_time
- [ ] `src/jianying/template.rs` — JSON 模板常量
- [ ] `src/jianying/material.rs` — VideoMaterial, AudioMaterial, Speed
- [ ] `src/jianying/segment.rs` — VideoSegment, AudioSegment
- [ ] `src/jianying/track.rs` — TrackType, Track
- [ ] `src/jianying/builder.rs` — DraftFolder, ScriptFile builder
- [ ] `tests/jianying_*.rs` — 集成测试文件
- [ ] uuid crate 添加到 Cargo.toml

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | -- |
| V3 Session Management | no | -- |
| V4 Access Control | no | -- |
| V5 Input Validation | yes | 文件路径校验（存在性、绝对路径）、数值范围校验（时长>0、分辨率>0） |
| V6 Cryptography | no | -- |

### Known Threat Patterns for JianYing Export

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal | Tampering | 校验路径在预期目录内 |
| 文件覆盖 | Tampering | allow_replace=false 默认行为 |

## Sources

### Primary (HIGH confidence)
- pyJianYingDraft v0.2.6 完整源码 — 直接读取安装的包文件 [VERIFIED: D:\App\python\Lib\site-packages\pyJianYingDraft\]
- pyJianYingDraft GitHub: https://github.com/GuanYixuan/pyJianYingDraft — 库主页
- `app/services/jianying_task.py` — Python 版剪映导出实现 [VERIFIED: 代码库文件]

### Secondary (MEDIUM confidence)
- pyJianYingDraft v0.2.6 PyPI: https://pypi.org/project/pyjianyingdraft/ — 版本信息

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — 所有数据结构从源码直接提取
- Architecture: HIGH — JSON 格式和文件结构从模板文件和源码双重确认
- Pitfalls: HIGH — 从 pyJianYingDraft 源码中的验证逻辑和注释提取

**Research date:** 2026-04-29
**Valid until:** 2026-05-29（剪映格式更新频率低，30 天内稳定）
