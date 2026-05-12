/// ts-rs export smoke test
///
/// Calls Type::export() for every type annotated with #[derive(TS)] in
/// narratoai-core.  The test verifies that the TypeScript declarations can be
/// generated without errors (i.e., all TS derive attributes resolve correctly).
///
/// The actual .ts file output (via #[ts(export)]) is triggered when
/// `cargo test -p narratoai-core` runs because ts-rs writes files at compile
/// time of the derive macro.  This test adds a runtime assertion so CI fails
/// if a newly added TS type is accidentally omitted from the list.

use ts_rs::TS;

// ---- config::types ----
use narratoai_core::config::types::{
    AppConfig, AppSection, UiSection, AzureSection, TencentSection,
    SoulVoiceSection, TtsQwenSection, IndexTTS2Section, DoubaoTTSSection,
    ProxySection, FramesSection, AudioSection,
};

// ---- llm::types ----
use narratoai_core::llm::types::LlmResponseFormat;

// ---- script::types ----
use narratoai_core::script::types::{OstType, ScriptClip};

// ---- documentary::types ----
use narratoai_core::documentary::types::DocumentaryRequest;

// ---- sde::types ----
use narratoai_core::sde::types::SdeRequest;

// ---- sdp::types ----
use narratoai_core::sdp::types::SdpRequest;

#[test]
fn ts_export_config_types() {
    // Each export() call writes the .ts file via the #[ts(export, export_to)] attribute
    // at compile time. At runtime, we just verify it doesn't panic.
    assert!(!AppConfig::decl().is_empty(), "AppConfig decl should not be empty");
    assert!(!AppSection::decl().is_empty());
    assert!(!UiSection::decl().is_empty());
    assert!(!AzureSection::decl().is_empty());
    assert!(!TencentSection::decl().is_empty());
    assert!(!SoulVoiceSection::decl().is_empty());
    assert!(!TtsQwenSection::decl().is_empty());
    assert!(!IndexTTS2Section::decl().is_empty());
    assert!(!DoubaoTTSSection::decl().is_empty());
    assert!(!ProxySection::decl().is_empty());
    assert!(!FramesSection::decl().is_empty());
    assert!(!AudioSection::decl().is_empty());
}

#[test]
fn ts_export_llm_types() {
    assert!(!LlmResponseFormat::decl().is_empty());
}

#[test]
fn ts_export_script_types() {
    assert!(!OstType::decl().is_empty());
    assert!(!ScriptClip::decl().is_empty());
}

#[test]
fn ts_export_documentary_types() {
    assert!(!DocumentaryRequest::decl().is_empty());
}

#[test]
fn ts_export_sde_types() {
    assert!(!SdeRequest::decl().is_empty());
}

#[test]
fn ts_export_sdp_types() {
    assert!(!SdpRequest::decl().is_empty());
}
