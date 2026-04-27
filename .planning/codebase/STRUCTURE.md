# Structure

**Last mapped:** 2026-04-27
**Project:** NarratoAI v0.7.8

## Directory Layout

```
NarratoAI/
├── app/                          # Core application package
│   ├── __init__.py
│   ├── config/                   # Configuration loading
│   │   ├── __init__.py
│   │   ├── config.py             # TOML config loader
│   │   ├── defaults.py           # Default values, model name normalization
│   │   ├── ffmpeg_config.py      # FFmpeg hardware acceleration detection
│   │   ├── audio_config.py       # Audio configuration constants
│   │   └── test_config_bootstrap_unittest.py
│   ├── models/                   # Data models
│   │   ├── __init__.py
│   │   ├── schema.py             # Pydantic models (VideoClipParams, etc.)
│   │   ├── const.py              # Constants (task states, etc.)
│   │   └── exception.py          # Custom exceptions
│   ├── services/                 # Business logic
│   │   ├── __init__.py
│   │   ├── task.py               # ★ Core pipeline orchestrator
│   │   ├── voice.py              # ★ TTS routing hub
│   │   ├── llm.py                # Legacy LLM interface
│   │   ├── llm/                  # LLM service abstraction
│   │   │   ├── __init__.py
│   │   │   ├── base.py           # Abstract base classes
│   │   │   ├── manager.py        # Provider factory + cache
│   │   │   ├── unified_service.py # Unified facade
│   │   │   ├── openai_compatible_provider.py # Single concrete provider
│   │   │   ├── migration_adapter.py # New/old interface bridge
│   │   │   ├── providers/__init__.py # register_all_providers()
│   │   │   ├── config_validator.py
│   │   │   ├── validators.py     # Output validation
│   │   │   ├── exceptions.py     # LLM-specific exceptions
│   │   │   └── test_*.py         # Unit + integration tests
│   │   ├── prompts/              # Prompt management
│   │   │   ├── __init__.py
│   │   │   ├── base.py           # BasePrompt, ModelType, OutputFormat
│   │   │   ├── registry.py       # PromptRegistry
│   │   │   ├── manager.py        # PromptManager
│   │   │   ├── template.py       # Template rendering
│   │   │   ├── validators.py     # Prompt validation
│   │   │   ├── exceptions.py     # Prompt exceptions
│   │   │   ├── documentary/      # Documentary prompts
│   │   │   │   ├── __init__.py
│   │   │   │   ├── frame_analysis.py
│   │   │   │   └── narration_generation.py
│   │   │   ├── short_drama_editing/ # SDE prompts
│   │   │   │   ├── __init__.py
│   │   │   │   ├── plot_extraction.py
│   │   │   │   └── subtitle_analysis.py
│   │   │   └── short_drama_narration/ # SDP prompts
│   │   │       ├── __init__.py
│   │   │       ├── plot_analysis.py
│   │   │       └── script_generation.py
│   │   ├── documentary/          # Documentary domain
│   │   │   ├── __init__.py
│   │   │   ├── frame_analysis_models.py
│   │   │   └── frame_analysis_service.py
│   │   ├── SDE/                  # Short Drama Explanation
│   │   │   └── short_drama_explanation.py
│   │   ├── SDP/                  # Short Drama Mix
│   │   │   ├── generate_script_short.py
│   │   │   └── utils/
│   │   │       ├── short_schema.py
│   │   │       ├── step1_subtitle_analyzer_openai.py
│   │   │       ├── step5_merge_script.py
│   │   │       └── utils.py
│   │   ├── audio_merger.py       # Audio mixing/merging
│   │   ├── audio_normalizer.py   # Audio normalization
│   │   ├── clip_video.py         # Video clipping
│   │   ├── merger_video.py       # Video concatenation
│   │   ├── generate_video.py     # Video generation
│   │   ├── generate_narration_script.py # Narration generation
│   │   ├── script_service.py     # Script management
│   │   ├── update_script.py      # Script updating
│   │   ├── subtitle.py           # Subtitle generation
│   │   ├── subtitle_merger.py    # Subtitle merging
│   │   ├── subtitle_text.py      # Subtitle text processing
│   │   ├── video.py              # Video utilities
│   │   ├── video_service.py      # Video service
│   │   ├── material.py           # Material management
│   │   ├── upload_validation.py  # Upload validation
│   │   ├── state.py              # State management (Memory/Redis)
│   │   ├── jianying_task.py      # JianYing draft export
│   │   └── youtube_service.py    # YouTube integration
│   └── utils/                    # Utilities
│       ├── utils.py              # General utilities
│       ├── ffmpeg_utils.py       # FFmpeg wrapper
│       ├── video_processor.py    # Video frame extraction
│       ├── check_script.py       # Script validation
│       ├── gemini_analyzer.py    # Legacy Gemini analyzer
│       ├── gemini_openai_analyzer.py # Gemini via OpenAI
│       ├── qwenvl_analyzer.py    # QwenVL analyzer
│       └── script_generator.py   # Script generation
├── webui/                        # Web UI
│   ├── __init__.py
│   ├── components/               # UI panels
│   │   ├── __init__.py
│   │   ├── basic_settings.py     # Main settings panel (33KB, largest)
│   │   ├── audio_settings.py     # Audio/TTS settings (45KB)
│   │   ├── script_settings.py    # Script editing (22KB)
│   │   ├── subtitle_settings.py  # Subtitle settings
│   │   ├── video_settings.py     # Video settings
│   │   ├── system_settings.py    # System config
│   │   └── ffmpeg_diagnostics.py # FFmpeg diagnostics
│   ├── config/
│   │   └── settings.py           # WebUI settings
│   ├── tools/                    # Business tool handlers
│   │   ├── base.py
│   │   ├── generate_script_docu.py
│   │   ├── generate_script_short.py
│   │   └── generate_short_summary.py
│   ├── utils/
│   │   ├── cache.py
│   │   ├── file_utils.py
│   │   └── vision_analyzer.py
│   └── i18n/
│       └── __init__.py
├── tests/                        # Test suite
│   ├── test_documentary_frame_analysis_service.py
│   ├── test_generate_narration_script_documentary_unittest.py
│   ├── test_generate_script_docu_unittest.py
│   ├── test_script_service_documentary_unittest.py
│   └── test_video_processor_documentary_unittest.py
├── docs/                         # Documentation images
├── resource/                     # Static resources
│   ├── fonts/
│   ├── public/
│   ├── scripts/
│   ├── songs/                    # Background music
│   ├── srt/
│   └── videos/
├── .github/                      # GitHub config
│   ├── pull_request_template.md
│   ├── release-drafter.yml
│   └── workflows/
│       ├── auto-release-generator.yml
│       ├── codeReview.yml
│       └── discord-release-notification.yml
├── webui.py                      # ★ Main entry point (Streamlit)
├── config.toml                   # Runtime config (gitignored)
├── config.example.toml           # Config template
├── requirements.txt              # Python dependencies
├── project_version               # Version string (0.7.8)
├── Dockerfile                    # Container build
├── docker-compose.yml            # Container orchestration
├── Makefile                      # Build shortcuts
├── run.bat                       # Windows launcher
└── conftest.py                   # Pytest config
```

## Key File Sizes

| File | Lines | Notes |
|------|-------|-------|
| `webui/components/audio_settings.py` | ~1200 | Largest UI component |
| `webui/components/basic_settings.py` | ~900 | Main settings |
| `webui.py` | ~500 | Main entry point |
| `app/services/task.py` | ~600+ | Core pipeline |
| `app/services/voice.py` | ~800+ | TTS routing |
| Total | ~26,272 | All Python files |

## Naming Conventions

- **Directories:** snake_case (app/services/, app/models/)
- **Modules:** snake_case (video_processor.py, frame_analysis_service.py)
- **Classes:** PascalCase (VideoClipParams, LLMServiceManager)
- **Constants:** UPPER_SNAKE_CASE (TASK_STATE_PROCESSING)
- **Config sections:** snake_case in TOML ([app], [ui], [azure])
- **Business domains:** Acronyms as directories (SDE/, SDP/)
