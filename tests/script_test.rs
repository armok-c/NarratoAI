use narratoai_core::script::{
    edit,
    load_script, save_script,
    types::{OstType, Script},
};
use tempfile::TempDir;

/// 手工构建的测试 JSON 数据（覆盖三种 OST 类型）
fn test_json_data() -> String {
    r#"[
        {
            "_id": 1,
            "timestamp": "00:00:00,000-00:00:10,000",
            "picture": "测试画面描述一",
            "narration": "这是第一段测试解说文案",
            "OST": 0
        },
        {
            "_id": 2,
            "timestamp": "00:00:10,000-00:00:20,000",
            "picture": "测试画面描述二",
            "narration": "播放原片2",
            "OST": 1
        },
        {
            "_id": 3,
            "timestamp": "00:00:20,000-00:00:30,000",
            "picture": "测试画面描述三",
            "narration": "这是第三段混合音频的解说",
            "OST": 2
        }
    ]"#.to_string()
}

/// Python 版实际脚本样例数据（来自 resource/scripts/2026-0416-112716.json）
///
/// 注意: 原始 JSON 中的中文引号 “ ” 在 Rust raw string 中可能导致 serde_json
/// 解析错误（它们看起来像 JSON 字符串界定符），因此使用普通 ASCII 引号替代。
fn python_sample_json() -> String {
    r#"[
        {
            "_id": 1,
            "timestamp": "00:00:51,410-00:01:19,425",
            "picture": "情侣二人在酒店房间内亲密调情，女人自信满满，男人一脸坏笑。",
            "narration": "为了拿下眼前的男人，女人竟苦练房中秘术三十八式，扬言包他满意。眼看气氛烘托到位，男人正要享受这“第十九式”，没想到女人下一秒的举动，却让他彻底傻眼！",
            "OST": 0
        },
        {
            "_id": 2,
            "timestamp": "00:01:19,425-00:01:56,390",
            "picture": "男人正要继续亲热，被女人猛地推开，表情瞬间变得严肃。",
            "narration": "就在男人情难自已时，女人却一把将他推开，前一秒还风情万种的脸，此刻却写满了严肃，直接抛出了一个让男人猝不及防的问题。",
            "OST": 0
        },
        {
            "_id": 3,
            "timestamp": "00:01:56,390-00:02:00,310",
            "picture": "女人表情严肃，提醒男人要去民政局领证。",
            "narration": "播放原片3",
            "OST": 1
        },
        {
            "_id": 4,
            "timestamp": "00:02:00,310-00:02:09,180",
            "picture": "女人眼神犀利地质问男人，男人一脸错愕。",
            "narration": "原来，这根本不是什么调情，而是一场蓄谋已久的逼婚！女人看着男人错愕的表情，直接从电视剧里学来了渣男理论，质问他是不是想不负责任！但更炸裂的还在后面！",
            "OST": 0
        },
        {
            "_id": 5,
            "timestamp": "00:02:09,180-00:02:14,955",
            "picture": "女人情绪激动，说出自己已经给了全部积蓄一万八作为彩礼。",
            "narration": "播放原片5",
            "OST": 1
        },
        {
            "_id": 6,
            "timestamp": "00:02:14,955-00:02:19,955",
            "picture": "男人愣在原地，表情复杂，不知所措。",
            "narration": "好家伙！女方不仅倒贴一万八当彩礼，还搭上了全部身家！面对这堪称“绝杀”的质问，这个男人究竟是会乖乖领证，还是会当场翻脸？",
            "OST": 0
        }
    ]"#.to_string()
}

/// 从 JSON 字符串加载脚本到临时文件
fn load_from_json(json: &str) -> (TempDir, Script) {
    let dir = TempDir::new().expect("创建临时目录失败");
    let path = dir.path().join("script.json");
    std::fs::write(&path, json).expect("写入临时文件失败");
    let script = load_script(&path).expect("加载脚本应成功");
    (dir, script)
}

/// Test 1: 端到端 round-trip —— 创建 JSON 文件 -> load -> save -> load -> 验证数据一致
#[test]
fn test_load_save_roundtrip() {
    let (dir, original) = load_from_json(&test_json_data());
    let save_path = dir.path().join("saved.json");

    save_script(&original, &save_path).expect("保存应成功");
    let reloaded = load_script(&save_path).expect("重新加载应成功");

    assert_eq!(reloaded.len(), original.len(), "片段数量应一致");
    for (i, (orig, reload)) in original.iter().zip(reloaded.iter()).enumerate() {
        assert_eq!(reload._id, orig._id, "clip[{}]._id 不一致", i);
        assert_eq!(reload.timestamp, orig.timestamp, "clip[{}].timestamp 不一致", i);
        assert_eq!(reload.picture, orig.picture, "clip[{}].picture 不一致", i);
        assert_eq!(reload.narration, orig.narration, "clip[{}].narration 不一致", i);
        assert_eq!(reload.ost, orig.ost, "clip[{}].ost 不一致", i);
    }
}

/// Test 2: 编辑 round-trip —— 加载 -> update_narration -> save -> reload -> 验证新 narration
#[test]
fn test_edit_save_reload() {
    let (dir, script) = load_from_json(&test_json_data());
    let save_path = dir.path().join("edited.json");

    let edited = edit::update_narration(&script, 0, "新的中文文案").expect("编辑应成功");
    save_script(&edited, &save_path).expect("保存应成功");
    let reloaded = load_script(&save_path).expect("重新加载应成功");

    assert_eq!(reloaded[0].narration, "新的中文文案", "narration 应已更新");
    // 其他字段不变
    assert_eq!(reloaded[0]._id, script[0]._id, "_id 不应变");
    assert_eq!(reloaded[0].timestamp, script[0].timestamp, "timestamp 不应变");
    assert_eq!(reloaded[0].picture, script[0].picture, "picture 不应变");
    assert_eq!(reloaded[0].ost, script[0].ost, "ost 不应变");
}

/// Test 3: 多次编辑 round-trip —— 加载 -> update_narration -> set_ost -> update_timestamp -> save -> reload
#[test]
fn test_multiple_edits() {
    let (dir, script) = load_from_json(&test_json_data());
    let save_path = dir.path().join("multi_edited.json");

    let step1 = edit::update_narration(&script, 1, "修改后的解说文案").expect("第一步应成功");
    let step2 = edit::set_ost(&step1, 1, OstType::OriginalSound).expect("第二步应成功");
    let step3 = edit::update_timestamp(&step2, 1, "00:00:15,000-00:00:25,000").expect("第三步应成功");

    save_script(&step3, &save_path).expect("保存应成功");
    let reloaded = load_script(&save_path).expect("重新加载应成功");

    assert_eq!(reloaded[1].narration, "修改后的解说文案", "narration 应已更新");
    assert_eq!(reloaded[1].ost, OstType::OriginalSound, "ost 应已更新");
    assert_eq!(reloaded[1].timestamp, "00:00:15,000-00:00:25,000", "timestamp 应已更新");
    // 其他字段不变
    assert_eq!(reloaded[1]._id, script[1]._id, "_id 不应变");
    assert_eq!(reloaded[1].picture, script[1].picture, "picture 不应变");
}

/// Test 4: 加载 Python 版实际脚本样例数据，验证所有片段正确反序列化
#[test]
fn test_load_python_sample() {
    let (_dir, script) = load_from_json(&python_sample_json());

    assert!(script.len() >= 6, "应有至少 6 个片段，实际: {}", script.len());
    assert_eq!(script[0]._id, 1, "第一个片段 _id 应为 1");
    assert_eq!(
        script[2].ost, OstType::OriginalSound,
        "第三个片段 OST 应为 OriginalSound (对应 JSON 中 OST=1)"
    );
    // 验证中文字符正确反序列化
    assert!(
        script[0].narration.contains("秘术三十八式"),
        "中文 narration 应正确反序列化"
    );
    assert!(
        script[0].picture.contains("酒店房间"),
        "中文 picture 应正确反序列化"
    );
}

/// Test 5: 保存后的 JSON 可被 serde_json 重新解析，不丢失核心字段
#[test]
fn test_save_produces_valid_json() {
    let (dir, script) = load_from_json(&test_json_data());
    let save_path = dir.path().join("valid_output.json");

    let edited = edit::update_narration(&script, 0, "验证 JSON 输出").expect("编辑应成功");
    save_script(&edited, &save_path).expect("保存应成功");

    let content = std::fs::read_to_string(&save_path).expect("读取输出文件失败");

    // 验证是合法 JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("保存的文件应是合法 JSON");

    // 验证是数组
    assert!(parsed.is_array(), "顶层应为 JSON 数组");

    // 验证核心字段存在
    let first = &parsed[0];
    assert!(first.get("_id").is_some(), "应包含 _id 字段");
    assert!(first.get("timestamp").is_some(), "应包含 timestamp 字段");
    assert!(first.get("picture").is_some(), "应包含 picture 字段");
    assert!(first.get("narration").is_some(), "应包含 narration 字段");
    assert!(first.get("OST").is_some(), "应包含 OST 字段");

    // 验证不存在 null 字段（Option::None 被 skip_serializing_if 省略）
    let content_str = content.as_str();
    assert!(
        !content_str.contains("\"audio\": null"),
        "不应包含 audio: null"
    );
    assert!(
        !content_str.contains("\"video\": null"),
        "不应包含 video: null"
    );
    assert!(
        !content_str.contains("\"subtitle\": null"),
        "不应包含 subtitle: null"
    );
    assert!(
        !content_str.contains("\"duration\": null"),
        "不应包含 duration: null"
    );

    // 验证中文字符原样输出
    assert!(
        content_str.contains("验证 JSON 输出"),
        "中文字符应原样输出"
    );
}
