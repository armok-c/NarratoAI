# Concerns

**Last mapped:** 2026-04-27
**Project:** NarratoAI v0.7.8

## Technical Debt

### High Severity

#### 1. Giant UI Component Files
- `webui/components/audio_settings.py` (45KB, ~1200 lines) and `basic_settings.py` (33KB, ~900 lines)
- These files mix UI rendering, business logic, and state management
- Difficult to maintain, test, or reuse
- **Impact:** Any UI change requires navigating massive files

#### 2. Legacy LLM Interface (`app/services/llm.py`)
- `llm.py` at service root coexists with the new `llm/` package
- `migration_adapter.py` bridges old/new APIs
- Two code paths increase maintenance burden
- **Impact:** Confusion about which interface to use, duplicated logic

#### 3. No Dependency Management
- Only `requirements.txt` with loose version pins (many `>=`)
- No lock file, no `pyproject.toml`
- **Impact:** Reproducible builds not guaranteed, dependency conflicts possible

### Medium Severity

#### 4. Test Coverage Extremely Low
- Only 5 test files, all focused on Documentary domain
- Zero tests for SDE, SDP, TTS, audio, video pipeline, state management
- No integration test suite (only excluded manual tests)
- No CI test automation
- **Impact:** Regressions likely when modifying core pipeline

#### 5. Inconsistent Error Handling
- Mix of `raise ValueError(...)`, custom exceptions, and `logger.error()` + return None
- Some functions silently fail, others raise
- No consistent error propagation strategy
- **Impact:** Hard to debug failures in production, errors may be swallowed

#### 6. Global Mutable State
- `app/services/state.py` uses module-level singleton
- Config loaded at import time, not lazy
- `app/config/config.py` modifies filesystem in `load_config()` (removes directory if config.toml is a dir)
- **Impact:** Testing is harder, order-dependent behavior

## Security Concerns

### Moderate

#### 7. API Keys in Config File
- `config.toml` contains API keys (gitignored but risky)
- No `.env` file support, no secret management
- `config.example.toml` shows the key structure publicly
- **Impact:** If `config.toml` is accidentally committed, keys are exposed

#### 8. No Input Validation on File Paths
- File paths from user input used directly in FFmpeg commands
- `subprocess.run()` calls with user-influenced arguments
- **Impact:** Potential command injection if not carefully validated

#### 9. Proxy Configuration
- Proxy settings in `[proxy]` section with no validation
- Could be used to redirect API traffic
- **Impact:** Man-in-the-middle risk if proxy is compromised

## Performance Concerns

### Moderate

#### 10. Synchronous Video Processing
- Video pipeline in `task.py` is synchronous
- Large video processing blocks the process
- No progress streaming to client
- **Impact:** User experience degrades with long videos

#### 11. No Caching for LLM Responses
- Same prompts sent multiple times for similar content
- No response caching layer
- **Impact:** Unnecessary API costs, slower processing

#### 12. Large File Upload Handling
- `streamlit run webui.py --server.maxUploadSize=2048`
- 2GB upload limit but no chunking or resumable uploads
- **Impact:** Timeout or failure with large files on slow connections

## Fragile Areas

### High Risk of Breaking

#### 13. OST-Type Driven Pipeline
- 3 OST values (0, 1, 2) drive completely different code paths in pipeline
- Adding a new OST type requires changes across multiple files
- No enum or centralized OST type definition
- **Impact:** Easy to introduce bugs when modifying pipeline behavior

#### 14. TTS Engine Dispatch
- `voice.py` uses string-based dispatch for 7 TTS engines
- Each engine has different configuration, error handling, and output format
- No common TTS interface/protocol
- **Impact:** Adding new TTS engine requires modifying dispatch function

#### 15. Streamlit Session State
- Heavy reliance on `st.session_state` for passing data between components
- No type safety on session state keys
- Key naming is inconsistent across components
- **Impact:** Session state key conflicts, hard-to-debug state issues

## Known Issues

#### 16. Docker Config Handling
- `config.py:load_config()` deletes directories named `config.toml`
- Defensive code against a specific past bug
- **Impact:** Unexpected behavior if directory name collides

#### 17. Optional Dependencies Not Guarded
- Some code imports optional packages at module level
- `moviepy` availability checked with try/except
- Other optional deps may fail ungracefully
- **Impact:** Import errors for missing optional dependencies
