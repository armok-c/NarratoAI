# Domain Pitfalls: NarratoAI Python-to-Rust Rewrite

**Domain:** Video processing pipeline + LLM/TTS integration (Rust rewrite of Python v0.7.8)
**Researched:** 2026-04-27
**Confidence:** HIGH (基于 smartEdit 项目的实际 Rust 实现经验和 Python 代码库分析)

## Critical Pitfalls

可能导致大规模重写或架构崩溃的错误。

---

### Pitfall 1: FFmpeg 绑定选型 -- 直接绑定 vs 命令行封装

**出了什么问题：** 选择 `ffmpeg-next`（FFmpeg C API 的 unsafe Rust 绑定）来替代 Python 的 `subprocess.run()` 调用 FFmpeg。`ffmpeg-next` 需要 FFmpeg 的 C 头文件和动态链接库，在 Windows 上构建极其痛苦（需要 MSVC + FFmpeg dev 包），且 API 不稳定、版本锁定困难。

**为什么会发生：** 直觉上认为 Rust 应该直接调用 C 库以获得最佳性能。Python 版也是调用外部 FFmpeg 二进制的，并非内嵌 C 库。

**后果：**
- Windows 开发环境搭建成为噩梦，每个开发者需要手动配置 FFmpeg dev 路径
- FFmpeg 版本更新导致编译失败
- CI/CD 流水线需要安装完整的 FFmpeg 开发包
- 调试困难：FFmpeg 的 C 错误码和 Rust panic 交织

**预防策略：** 使用 `ffmpeg-sidecar`（将 FFmpeg 二进制作为子进程调用），与 Python 版的 `subprocess.run()` 方式完全对齐。smartEdit 已经验证这条路可行（Cargo.toml 中使用 `ffmpeg-sidecar = "2.4"`）。性能瓶颈在视频编解码本身而不在进程调用开销。

**警告信号：**
- 构建脚本中需要 `pkg-config` 或 `FFMPEG_DIR` 环境变量
- `build.rs` 中有 `println!("cargo:rustc-link-lib=...")` 指向 FFmpeg
- Cargo.toml 中出现 `ffmpeg-sys-next` 或 `ffmpeg-next`

**应在哪个阶段处理：** Phase 1 -- 项目初始化和技术栈确认。这是架构级决策，选错则整个视频处理层需要重写。

---

### Pitfall 2: 异步运行时与 FFmpeg 子进程的阻塞冲突

**出了什么问题：** FFmpeg 子进程调用是阻塞 I/O 操作（等待视频编码完成），如果直接在 tokio 的异步上下文中运行，会阻塞 tokio 的工作线程，导致 TTS 请求和 LLM API 调用超时或饿死。

**为什么会发生：** Rust 的 async/await 不同于 Python 的 `asyncio.run()`。Python 中 `asyncio.run()` 在独立线程运行，而 Rust 的 tokio 共享线程池，一个阻塞操作会卡住整个线程。

**后果：**
- 视频编码期间，所有 LLM/TTS 请求挂起
- Tauri UI 命令无响应
- 看起来像是整个应用"冻住"

**预防策略：**
1. 使用 `tokio::task::spawn_blocking()` 将 FFmpeg 调用包装在阻塞线程池中
2. 或者使用 `std::process::Command` + `tokio::process::Command` 的异步版本配合 `ffmpeg-sidecar`
3. smartEdit 的做法：`ffmpeg-sidecar` 本身支持异步迭代输出，但底层仍是子进程

**警告信号：**
- FFmpeg 调用直接出现在 `async fn` 中而没有 `spawn_blocking`
- tokio 的工作线程数设置过小（默认等于 CPU 核心数）
- 并发 TTS 和视频处理时出现延迟

**应在哪个阶段处理：** Phase 1（异步架构设计）+ Phase 2（视频处理层实现时验证）。在实现第一个 FFmpeg 调用时就必须测试并发场景。

---

### Pitfall 3: OST 驱动的管线分支逻辑移植不完整

**出了什么问题：** Python 版的管线由 OST 值（0=仅解说, 1=仅原声, 2=混合）驱动三条完全不同的代码路径。移植时遗漏任何一条路径的特殊处理，都会导致特定类型的视频片段处理错误。

**为什么会发生：** Python 版的 OST 逻辑分散在 `task.py`、`clip_video.py`、`audio_merger.py`、`subtitle_merger.py` 多个文件中（见 CONCERNS.md #13）。没有集中的 OST 类型定义（没有枚举，用的是裸整数 0/1/2）。Rust 移植时容易遗漏某些文件中的 OST 特殊逻辑。

**后果：**
- 某种 OST 类型的片段无声、无字幕或时长错误
- 特定场景下的管线失败只在用户实际使用时才暴露
- 测试用例可能只覆盖最常见的 OST=0 场景

**预防策略：**
1. **第一步就定义 Rust 枚举：** `enum OstType { NarrationOnly, OriginalOnly, Mixed }`
2. **逐行对照 Python 管线代码** -- `task.py:start_subclip_unified()` 的每个 `if OST == X` 分支
3. 为每种 OST 类型编写独立的集成测试
4. 用 Python 版生成一个包含所有 3 种 OST 类型的测试脚本 JSON，在 Rust 版运行后逐帧对比

**警告信号：**
- Rust 管线代码中出现 `if ost == 0` 这样的裸数字比较而不是枚举匹配
- OST=1（仅原声）的处理路径缺少测试
- 管线中 TTS 时长裁剪和脚本时间戳裁剪的切换逻辑不清晰

**应在哪个阶段处理：** Phase 3（纪录片流水线实现）。这是核心管线的骨架逻辑，必须在实现管线时立即测试所有三条路径。

---

### Pitfall 4: Edge TTS 协议对齐的细微差异

**出了什么问题：** Edge TTS 不是公开 API，而是逆向工程的 WebSocket 协议。Python `edge-tts` 库和 Rust 实现之间任何微小的协议差异（时间戳格式、UUID 格式、Header 大小写、Sec-MS-GEC token 算法）都会导致 403 或连接失败。smartEdit 的 `edge_tts.rs` 有 1000+ 行，其中大量代码是在处理这些细节。

**为什么会发生：**
- WebSocket 帧的文本格式（`\r\n\r\n` 分隔头和正文）需要精确对齐
- `Sec-MS-GEC` token 算法依赖时钟偏差校正（`clock_skew.rs`）
- UUID 必须是 32 位无连字符格式（`uuid.uuid4().hex`），而不是标准 UUID 格式
- Chromium 版本号硬编码，需要随 Edge 更新而更新
- 日期字符串必须精确匹配 JavaScript 的 `Date.toString()` 格式

**后果：**
- 403 Forbidden 错误，难以诊断（是算法错误还是网络问题？）
- 每次 Chromium/Edge 更新可能导致认证失败
- 不同操作系统的 TLS 行为差异

**预防策略：**
1. 直接参考 smartEdit 的 `edge_tts.rs` 实现（已验证可用）
2. 将所有协议常量（CHROMIUM_VERSION、TRUSTED_CLIENT_TOKEN 等）集中在一个常量文件中
3. 编写与 Python `edge-tts` 输出的逐字节对比测试
4. 实现时钟偏差自动校正和 403 重试逻辑（smartEdit 已有）
5. 添加连接诊断功能（`diagnose_connection()`）

**警告信号：**
- Edge TTS 连接返回 403 且没有时钟校正逻辑
- UUID 使用 `to_string()` 而不是 `as_simple().to_string()`（包含连字符 vs 不包含）
- 日期格式不包含 `GMT+0000 (Coordinated Universal Time)` 后缀

**应在哪个阶段处理：** Phase 2（TTS 路由层实现）。Edge TTS 是默认引擎，必须第一个实现并确保可用。

---

### Pitfall 5: TTS 引擎 trait 设计不统一导致 7 个引擎各自为政

**出了什么问题：** Python 版的 7 个 TTS 引擎（edge_tts、azure_speech、tencent_tts、soulvoice、tts_qwen、indextts2、doubaotts）使用不同的协议（WebSocket、REST、SDK）、不同的认证方式、不同的音频输出格式。如果 Rust 的 `TtsEngine` trait 设计不好，每个引擎的实现会散落为互不兼容的孤岛。

**为什么会发生：** Python 版本身就是字符串分发的（`voice.py:tts()` 中的 if-elif 链），没有统一接口。Rust 重写时如果只是照搬这个结构而不设计好 trait，会继承同样的混乱。

**后果：**
- 添加新引擎需要修改核心分发逻辑
- 不同引擎的错误处理不一致
- 某些引擎缺少连接测试或语音列表功能
- 引擎之间无法共享通用逻辑（代理配置、重试、超时）

**预防策略：**
1. 定义统一的 `TtsEngine` trait（smartEdit 已有：`async fn synthesize()`, `test_connection()`, `list_voices()`）
2. 使用枚举 + `match` 分发而不是字符串比较
3. 引擎创建使用工厂模式，配置驱动
4. 共享基础设施：代理配置、HTTP 客户端、重试逻辑提取为公共模块
5. 优先实现 Edge TTS 和 Qwen3 TTS（smartEdit 已验证），其他引擎按使用频率排序

**警告信号：**
- TTS 引擎选择使用 `&str` 匹配而不是枚举
- 不同引擎的 `synthesize()` 返回不同的数据类型
- 代理配置在每个引擎中重复实现

**应在哪个阶段处理：** Phase 2（TTS 路由层设计）。trait 接口一旦定下就不应频繁变更，所有引擎实现都依赖它。

---

### Pitfall 6: Tauri 命令层的错误类型序列化丢失上下文

**出了什么问题：** Tauri 命令返回 Rust 的 `Result<T, E>` 给前端。如果错误类型 `E` 没有实现 `Serialize`，或者序列化后丢失了关键上下文（如 FFmpeg 错误的具体位置、LLM API 的状态码），前端无法显示有意义的错误信息。

**为什么会发生：** Rust 的错误处理是类型安全的，但跨越 FFI 边界（Rust -> Tauri -> JavaScript）时，错误被转换为 JSON。如果错误枚举没有手写 `Serialize` 实现，`thiserror` 的 `#[error(...)]` 只提供 `Display` 格式的字符串，丢失结构化信息。

**后果：**
- 前端只能显示 "Unknown error" 或模糊的错误消息
- 无法区分认证失败、速率限制、网络超时等需要不同用户操作的错误
- 调试时需要在 Rust 端查看日志才能理解错误原因

**预防策略：**
1. 为所有错误类型实现 `Serialize`（smartEdit 的 `SmartEditError` 已有 `to_json()` 和 `impl Serialize`）
2. 错误枚举包含 `error_code()` 方法，返回稳定的字符串标识（如 `"LLM_AUTH"`, `"TTS_TIMEOUT"`）
3. 错误枚举包含 `is_retryable()` 方法，前端据此决定是否显示重试按钮
4. 每个新增的错误变体都要有对应的前端处理逻辑

**警告信号：**
- Tauri 命令返回 `anyhow::Error` 或 `Box<dyn Error>`
- 错误类型只有 `Display` 实现没有 `Serialize`
- 前端收到错误后无法区分错误类型

**应在哪个阶段处理：** Phase 1（错误架构设计）。错误类型是贯穿整个系统的骨架，必须在第一行业务代码之前定义好。

---

## Moderate Pitfalls

可能导致延期或返工的错误。

---

### Pitfall 7: LLM 流式响应未实现（Python 版也没用但后续会需要）

**出了什么问题：** Python 版的 LLM 调用全部是非流式的（等待完整响应返回）。smartEdit 的 Rust 版也是非流式的。但如果后续要在前端实现"实时显示 AI 生成的文案"，就需要流式响应。如果初始架构不支持流式，后期改造工作量巨大。

**为什么会发生：** 先做最简单的非流式实现是合理的。但 reqwest 的流式处理和 Tauri 的事件系统结合方式需要在架构设计时就考虑好。

**预防策略：**
1. LLM 服务层预留流式接口（即使初始不实现）：`fn generate_text_stream() -> impl Stream<Item = Result<Chunk>>`
2. Tauri 事件系统（`app.emit()`）用于向前端推送流式数据
3. 初始阶段只实现非流式，但接口设计要兼容流式扩展

**警告信号：**
- LLM 客户端方法签名只能返回完整字符串
- Tauri 命令只能返回 `Result<String>` 而不能推送事件

**应在哪个阶段处理：** Phase 1（LLM 接口设计时预留扩展点）。不需要立即实现流式，但接口必须设计为可扩展。

---

### Pitfall 8: Python moviepy 功能的 Rust 替代方案选型

**出了什么问题：** Python 版使用 `moviepy` 做视频合成和字幕烧入（subtitle burn-in）。moviepy 是纯 Python 的视频处理库（底层也调用 FFmpeg），Rust 生态中没有直接等价物。如果用 `ffmpeg-sidecar` 重新实现所有 moviepy 功能，需要处理字幕渲染、字体嵌入、时间轴同步等复杂问题。

**为什么会发生：** moviepy 封装了大量视频编辑抽象（时间线、片段组合、特效），这些都是 FFmpeg 命令行不容易直接实现的。

**后果：**
- 字幕烧入质量不一致（字体渲染、位置、样式差异）
- 视频合成逻辑需要大量 FFmpeg filter graph 知识
- 某些 moviepy 特效在 FFmpeg 中没有等价命令

**预防策略：**
1. 分析 Python 版中 moviepy 的实际使用范围（主要是字幕烧入和最终合成）
2. 将 moviepy 操作转化为等效的 FFmpeg `drawtext` 或 `subtitles` filter 命令
3. 先用最简单的 FFmpeg 字幕烧入方式（SRT -> ASS -> burn-in），再逐步对齐样式
4. 编写对比测试：Python 版和 Rust 版对同一视频的输出应视觉一致

**警告信号：**
- 尝试在 Rust 中嵌入 Python 运行时（PyO3）来调用 moviepy
- 字幕烧入使用 FFmpeg `drawtext` 但没有处理中文字体问题

**应在哪个阶段处理：** Phase 3（视频处理管线实现时）。在实现字幕生成和最终合成步骤时解决。

---

### Pitfall 9: 并发文件操作导致状态损坏

**出了什么问题：** Rust 版使用文件存储（JSON/TOML）替代 Python 版的内存字典或 Redis。如果多个异步任务同时读写同一个状态文件（如任务进度、TTS 缓存、脚本结果），可能出现数据竞争导致文件内容损坏或丢失。

**为什么会发生：** 文件 I/O 不是原子操作。tokio 的并发任务可能在另一个任务写入文件时读取到部分写入的内容。Python 版的 Streamlit 是单线程的，天然不存在这个问题。

**后果：**
- 任务状态文件变成无效 JSON
- 脚本生成结果丢失或损坏
- 进度信息不准确

**预防策略：**
1. 使用 `tokio::sync::RwLock` 或 `tokio::sync::Mutex` 保护所有文件写入操作
2. 采用"写入临时文件 + 原子重命名"模式（`tempfile` + `std::fs::rename`）
3. 考虑使用 `sqlite` 作为状态存储（smartEdit 就用了 sqlx + SQLite）而不是纯文件
4. 文件损坏时要有自动恢复机制（备份 + 校验）

**警告信号：**
- 直接调用 `tokio::fs::write()` 写入状态文件而没有锁保护
- 状态文件写入没有使用临时文件+重命名模式
- 文件读取时没有处理 JSON 解析失败

**应在哪个阶段处理：** Phase 1（状态管理层设计）。文件状态管理是基础设施，必须在业务代码开始前设计好并发安全策略。

---

### Pitfall 10: 配置系统 TOML 解析与 Python 版的字段对齐

**出了什么问题：** Python 版的 `config.toml` 使用 `toml` 库加载为 Python 字典，字段名是 snake_case 字符串，值类型是动态的。Rust 版需要将每个字段映射到结构化的 Rust 结构体，字段名、类型、默认值、可选性都必须精确对齐。

**为什么会发生：** Python 的动态类型允许配置字段随意增减而不报错。Rust 的 serde 反序列化要求严格匹配，多余字段会被忽略或报错（取决于 `#[serde(deny_unknown_fields)]`），缺少字段也会报错（除非标注 `#[serde(default)]`）。

**后果：**
- 用户已有的 `config.toml` 在 Rust 版加载失败
- 每次新增配置字段都需要考虑向后兼容
- 不同版本的配置文件无法互通

**预防策略：**
1. 为配置结构体的每个字段添加 `#[serde(default)]` 提供默认值
2. 使用 `#[serde(rename = "camelCase")]` 或 `#[serde(rename_all = "snake_case")]` 保持与 Python 版一致
3. 实现 `Default` trait 提供完整默认配置
4. 编写测试：用 Python 版的 `config.example.toml` 在 Rust 版加载，确保不报错
5. 实现配置迁移逻辑：加载后检查版本号，自动补全新增字段

**警告信号：**
- 配置结构体中缺少 `#[serde(default)]` 标注
- 加载配置时直接 `toml::from_str()` 而没有处理字段缺失
- 与 Python 版的配置字段名大小写不一致

**应在哪个阶段处理：** Phase 1（配置系统实现）。配置是所有模块的基础，必须最先实现且与 Python 版对齐。

---

### Pitfall 11: Windows 路径和编码问题

**出了什么问题：** NarratoAI 是桌面应用，主要在 Windows 上运行。Windows 的路径分隔符（`\` vs `/`）、文件名编码（GBK vs UTF-8）、长路径限制（260 字符）都可能在 Rust 中引发问题。smartEdit 已经遇到了这些问题（`build_concat_file_contents()` 中需要手动将 `\` 替换为 `/`）。

**为什么会发生：** Rust 标准库的路径处理在 Windows 上使用 `\`，但 FFmpeg 的 concat demuxer 要求 `/`。视频文件名可能包含中文，需要确保所有路径操作使用一致的编码。

**后果：**
- FFmpeg 找不到输入文件（路径分隔符问题）
- 中文文件名的视频无法处理
- 临时文件路径过长导致创建失败

**预防策略：**
1. 所有传给 FFmpeg 的路径统一使用 `/`（`path.to_string_lossy().replace('\\', "/")`）
2. 使用 `uuid` 生成临时文件名，避免中文字符
3. 输出文件路径使用 `std::path::PathBuf` 处理，不要手动拼接字符串
4. 测试用例必须包含中文路径

**警告信号：**
- 文件路径使用 `format!("{}\\{}", dir, filename)` 拼接
- FFmpeg 命令中的路径包含 `\` 且没有转义
- 临时文件名使用用户输入的原始文本

**应在哪个阶段处理：** Phase 2（基础设施层）+ 持续关注。第一次实现 FFmpeg 调用时就要解决路径问题。

---

## Minor Pitfalls

可能导致小问题但不会影响项目进展的错误。

---

### Pitfall 12: 日志系统不统一

**出了什么问题：** Python 版使用 `loguru`，Rust 版应使用 `tracing`（smartEdit 已验证）。但如果 Rust 库的不同模块使用不同的日志库（`log` crate vs `tracing` crate），日志输出格式不一致，难以调试。

**预防策略：** 全局统一使用 `tracing` + `tracing-subscriber`。不要在同一个项目中混用 `log` 和 `tracing`。

**应在哪个阶段处理：** Phase 1（项目初始化）。

---

### Pitfall 13: Tauri 命令参数和返回值的类型导出

**出了什么问题：** Tauri 命令的参数和返回值需要序列化为 JSON。如果 Rust 端的类型定义与前端 TypeScript 不一致，会导致运行时错误。smartEdit 使用 `ts-rs` crate 自动从 Rust 类型生成 TypeScript 类型定义。

**预防策略：** 使用 `ts-rs` crate 为所有 Tauri 命令的参数和返回值自动生成 TypeScript 类型。前端直接引用生成的类型文件。

**应在哪个阶段处理：** Phase 1（Tauri 命令层设计时集成 ts-rs）。

---

### Pitfall 14: FFmpeg 进度解析

**出了什么问题：** Python 版的管线是同步的，没有实时进度反馈。Rust 版作为桌面应用，用户期望看到实时进度。FFmpeg 的 stderr 输出包含进度信息（`time=` 行），但需要解析且格式不统一。

**预防策略：** `ffmpeg-sidecar` 支持迭代读取 FFmpeg 的 stderr 输出。解析 `time=` 和 `speed=` 行计算进度百分比。通过 Tauri 事件系统向前端推送进度更新。

**应在哪个阶段处理：** Phase 3（视频处理管线实现时）。

---

### Pitfall 15: 测试策略 -- 对齐 Python 版的测试覆盖率基线

**出了什么问题：** Python 版只有 5 个测试文件，都聚焦纪录片流水线（见 CONCERNS.md #4）。Rust 重写时如果测试覆盖率同样低，就无法验证重写的正确性。但追求高覆盖率又可能拖慢进度。

**预防策略：**
1. 每个模块至少有单元测试（Rust 的测试文化天然支持）
2. 核心管线（OST 3 种路径）必须有集成测试
3. 使用 wiremock（smartEdit 已验证）模拟 LLM API
4. 对比测试：Python 版输出 vs Rust 版输出的二进制级或 JSON 级对比
5. 不追求 100% 覆盖率，但核心路径必须覆盖

**应在哪个阶段处理：** 持续。每个 Phase 都要伴随测试编写。

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| 项目初始化 / 技术栈 | FFmpeg 绑定选错 (Pitfall 1) | 直接用 ffmpeg-sidecar，禁止 ffmpeg-next |
| 异步架构 | 阻塞 FFmpeg 卡死 tokio (Pitfall 2) | spawn_blocking 包装所有 FFmpeg 调用 |
| 错误系统 | 错误类型不可序列化 (Pitfall 6) | 参照 smartEdit 的 SmartEditError 实现 |
| 配置系统 | TOML 字段不对齐 (Pitfall 10) | serde(default) + Python config.example.toml 测试 |
| 状态管理 | 并发文件写入损坏 (Pitfall 9) | RwLock + tempfile + atomic rename |
| TTS 路由层 | Edge TTS 协议对齐失败 (Pitfall 4) | 参考 smartEdit edge_tts.rs 逐行对齐 |
| TTS 路由层 | TtsEngine trait 设计不当 (Pitfall 5) | 先定义 trait 再实现任何引擎 |
| 视频处理管线 | OST 分支逻辑遗漏 (Pitfall 3) | 枚举 + 3 条路径独立测试 |
| 视频处理管线 | moviepy 功能无 Rust 替代 (Pitfall 8) | 全部转为 FFmpeg filter graph |
| 跨平台 | Windows 路径/编码问题 (Pitfall 11) | 统一 `/` 分隔符 + UUID 临时文件名 |
| LLM 服务层 | 流式响应预留不足 (Pitfall 7) | 接口设计时预留 Stream 返回类型 |
| 前端对接 | 类型定义不同步 (Pitfall 13) | ts-rs 自动生成 TypeScript 类型 |
| 管线进度 | 无实时进度反馈 (Pitfall 14) | 解析 FFmpeg stderr + Tauri 事件 |

## smartEdit 已踩过的坑（可直接避免）

smartEdit 项目（失败的 Rust 重写尝试）留下了宝贵的经验教训：

1. **PyO3 集成是死路：** smartEdit 尝试用 PyO3 嵌入 Python 运行时来调用 moviepy（`services/pyo3_spike/`），标记为 "DISPOSABLE"，最终放弃。NarratoAI 重写应完全避免引入 Python 运行时。

2. **Edge TTS 的 Chromium 版本号是硬编码的：** smartEdit 中 `CHROMIUM_MAJOR_VERSION` 和 `CHROMIUM_FULL_VERSION` 是硬编码常量（当前 "143.0.3650.75"），需要随 Edge 更新手动更新。必须有版本过期的检测和更新机制。

3. **SQLite 比纯文件存储更适合桌面应用：** smartEdit 使用了 `sqlx + SQLite` 来存储 TTS 配置、LLM 配置、脚本缓存等。虽然 PROJECT.md 说用文件存储，但实际经验表明 SQLite 在并发安全、查询效率、数据完整性方面都更优。

4. **WebSocket 代理支持必须从一开始就考虑：** smartEdit 的 Edge TTS 有完整的代理支持（HTTP CONNECT 隧道 + TLS），实现复杂度约 130 行（`connect_via_proxy()`）。如果不在初始版本就支持代理，中国用户可能无法使用 Edge TTS。

## Sources

- smartEdit 代码库 (`E:\GitLib\smartEdit\src-tauri\`) -- 已验证的 Rust 实现，包含 Edge TTS、LLM 服务、视频管线、TTS 路由、错误处理等
- NarratoAI Python 代码库 (`E:\GitLib\NarratoAI\`) -- 被重写的原始系统
- `.planning/codebase/CONCERNS.md` -- Python 版已知问题清单
- `.planning/codebase/ARCHITECTURE.md` -- Python 版架构文档
- `.planning/codebase/INTEGRATIONS.md` -- 外部集成清单（7 个 TTS 引擎、LLM 服务等）
- `.planning/PROJECT.md` -- 重写项目范围和约束
