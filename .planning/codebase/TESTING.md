# Testing

**Last mapped:** 2026-04-27
**Project:** NarratoAI v0.7.8

## Framework

| Component | Technology |
|-----------|-----------|
| Test Runner | pytest |
| Assertion Style | unittest.TestCase (mixed with pytest) |
| Mocking | unittest.mock (patch, MagicMock) |

## Test Structure

### Root Test Directory: `tests/`
```
tests/
├── test_documentary_frame_analysis_service.py
├── test_generate_narration_script_documentary_unittest.py
├── test_generate_script_docu_unittest.py
├── test_script_service_documentary_unittest.py
└── test_video_processor_documentary_unittest.py
```

**All tests focus on the Documentary domain.** No tests for SDE, SDP, TTS, or video processing pipeline.

### Colocated Tests: `app/services/llm/`
```
app/services/llm/
├── test_llm_service.py          # Excluded from pytest (needs live API keys)
├── test_openai_compat_unittest.py
└── test_openai_compatible_integration.py  # Excluded from pytest (needs live API keys)
```

## Test Configuration

`conftest.py` excludes integration tests:
```python
collect_ignore = [
    "app/services/llm/test_llm_service.py",
    "app/services/llm/test_openai_compatible_integration.py",
]
```

These tests require live API credentials and are excluded from automated runs.

## Test Patterns

### Unit Test Pattern
- Uses `unittest.TestCase` base class
- Tests individual methods in isolation
- Mocks external dependencies (LLM calls, file I/O)
- Asserts on return values and method calls

Example from `test_documentary_frame_analysis_service.py`:
```python
class DocumentaryFrameAnalysisServiceTests(unittest.TestCase):
    def test_build_analysis_prompt_formats_real_frame_count(self):
        service = DocumentaryFrameAnalysisService()
        prompt = service._build_analysis_prompt(frame_count=3)
        self.assertIn("我提供了 3 张视频帧", prompt)
```

### Test Naming
- Files: `test_{feature}_{domain}_unittest.py`
- Classes: `{Feature}Tests(unittest.TestCase)`
- Methods: `test_{what}_{condition}`

## Coverage Gaps

### Untested Areas
- **SDE (Short Drama Explanation):** No tests
- **SDP (Short Drama Mix):** No tests
- **TTS routing:** No tests
- **Video pipeline (task.py):** No tests
- **Audio processing:** No tests
- **Subtitle generation:** No tests
- **State management:** No tests
- **Config loading:** No tests
- **JianYing export:** No tests
- **WebUI components:** No tests

### Tested Areas
- Documentary frame analysis (prompt building, result parsing)
- Documentary narration script generation
- Documentary script service
- Video processor (frame extraction)
- LLM OpenAI compatible provider (unit only)

## Running Tests

```bash
pytest                          # Run all non-excluded tests
pytest tests/ -v                # Verbose output
pytest app/services/llm/        # Run LLM tests (excludes integration)
```

No test configuration file (no pytest.ini or pyproject.toml).

## CI Testing

GitHub Actions workflows exist for:
- Auto-release generation
- Code review
- Discord notifications

No automated test execution in CI is evident from the workflow files.
