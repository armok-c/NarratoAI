# Conventions

**Last mapped:** 2026-04-27
**Project:** NarratoAI v0.7.8

## Code Style

### Language
- **All code in English** — identifiers, comments, variable names
- **User-facing strings in Chinese** — UI labels, error messages, log messages
- **Docstrings in Chinese** — function/class documentation

### Imports
- Standard library first, then third-party, then local:
  ```python
  import os
  import traceback
  from loguru import logger
  from app.config import config
  from app.models import const
  ```
- Grouped with blank lines between groups

### Logging
- Uses `loguru` exclusively (NOT Python stdlib `logging`)
- Pattern: `logger.info()`, `logger.error()`, `logger.debug()`
- Log messages often use f-strings with Chinese text
- Log format configured in `webui.py:init_log()`

### Error Handling
- Custom exceptions in `app/models/exception.py` and `app/services/llm/exceptions.py`
- `try/except` blocks with `traceback` for detailed error info
- `loguru.logger.error()` for logging caught exceptions
- `raise ValueError(...)` for input validation failures
- Chinese error messages in user-facing exceptions

### Type Hints
- Pydantic models use typed fields with `Field()`
- Service methods use type hints in signatures
- `Optional[str]`, `List[str]`, `Union[...]` common patterns
- Not consistently applied to all functions (gradual adoption)

### Configuration
- Runtime config via TOML (`config.toml`, gitignored)
- Template config in `config.example.toml`
- Config accessed via `app.config.config` module-level singleton
- Config sections: `[app]`, `[ui]`, `[azure]`, `[tencent]`, `[soulvoice]`, `[tts_qwen]`, `[indextts2]`, `[doubaotts]`, `[proxy]`, `[frames]`

## Patterns

### Provider/Registry Pattern
Used for LLM services and prompts:
- Abstract base class defines interface
- Registry stores instances by category/name/version
- Factory creates and caches instances
- Single concrete implementation (OpenAI compatible)

### Strategy Pattern (TTS)
- `voice.py:tts()` acts as strategy dispatcher
- Engine name string routes to specific TTS implementation
- Each engine handles its own configuration

### ABC Pattern (State)
- `BaseState` abstract class
- `MemoryState` and `RedisState` implementations
- Global singleton selected at import time based on config

### Pipeline Pattern (Video Processing)
- 6-step sequential pipeline in `task.py`
- Each step is a separate function call
- State tracked via task ID and progress percentage
- OST type drives behavior at each step

### Facade Pattern (LLM)
- `UnifiedLLMService` provides simple API
- Hides complexity of provider selection, retry, validation
- `migration_adapter.py` bridges old API to new

## File Organization

- Business domains separated into subdirectories: `documentary/`, `SDE/`, `SDP/`
- Shared services at `app/services/` root level
- UI components in `webui/components/` — one file per settings panel
- Tests colocated with source in `app/services/llm/` (unit tests)
- Integration tests in root `tests/` directory

## Encoding

- All Python files use UTF-8 encoding
- Config files use UTF-8 (with UTF-8-sig fallback in loader)
- Script JSON files use UTF-8 (`encoding="utf-8"` in open calls)

## Git Conventions

- Branch naming: feature branches (e.g., `feature/doubao-tts`, `feature/export-jianying-draft`)
- Commit messages: conventional commits style (`fix:`, `add`, `Merge pull request`)
- PR template available at `.github/pull_request_template.md`
