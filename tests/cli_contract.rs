use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_creative-idea-lab"))
}

fn temp_path(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("creative-idea-lab-test-{name}-{suffix}"))
}

fn assert_exists(path: impl AsRef<Path>) {
    assert!(
        path.as_ref().exists(),
        "expected path to exist: {}",
        path.as_ref().display()
    );
}

#[test]
fn init_creates_core_vault_structure_and_templates() {
    let target = temp_path("init");

    let output = Command::new(cli())
        .arg("init")
        .arg(&target)
        .output()
        .expect("failed to run creative-idea-lab init");

    assert!(
        output.status.success(),
        "init should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for directory in [
        "00_raw-sessions",
        "01_sessions",
        "10_knowledge",
        "20_thoughts",
        "30_ideas",
        "40_posts/linkedin",
        "40_posts/x.com",
        "40_posts/published",
        "_templates",
        "_system",
    ] {
        assert_exists(target.join(directory));
    }

    for template in [
        "_templates/10_knowledge.md",
        "_templates/20_thought.md",
        "_templates/30_idea.md",
        "_templates/40_post.md",
    ] {
        assert_exists(target.join(template));
    }

    let thought_template = fs::read_to_string(target.join("_templates/20_thought.md"))
        .expect("thought template should be readable");
    assert!(thought_template.contains("source_refs: []"));
    assert!(thought_template.contains("user_confirmed: false"));

    let post_template = fs::read_to_string(target.join("_templates/40_post.md"))
        .expect("post template should be readable");
    assert!(post_template.contains("## Review Checklist"));

    fs::remove_dir_all(target).ok();
}

#[test]
fn validate_reports_initialized_lab_as_valid() {
    let target = temp_path("validate");
    let init = Command::new(cli())
        .arg("init")
        .arg(&target)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before validate");

    let output = Command::new(cli())
        .arg("validate")
        .arg(&target)
        .output()
        .expect("failed to run validate");

    assert!(
        output.status.success(),
        "validate should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("valid"));

    fs::remove_dir_all(target).ok();
}

#[test]
fn skill_print_outputs_installable_skill_markdown() {
    let output = Command::new(cli())
        .args(["skill", "print"])
        .output()
        .expect("failed to run skill print");

    assert!(
        output.status.success(),
        "skill print should succeed\nstderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("name: creative-idea-lab"));
    assert!(stdout.contains("# Creative Idea Lab"));
}

#[test]
fn skill_install_writes_skill_to_explicit_target() {
    let target = temp_path("skills");

    let output = Command::new(cli())
        .args(["skill", "install", "--target"])
        .arg(&target)
        .output()
        .expect("failed to run skill install");

    assert!(
        output.status.success(),
        "skill install should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let skill_path = target.join("creative-idea-lab").join("SKILL.md");
    assert_exists(&skill_path);

    let skill = fs::read_to_string(skill_path).expect("installed skill should be readable");
    assert!(skill.contains("name: creative-idea-lab"));
    assert!(skill.contains("creative-idea-lab validate"));

    fs::remove_dir_all(target).ok();
}

#[test]
fn packaged_skill_file_is_available_for_agent_installers() {
    let skill_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("skills")
        .join("creative-idea-lab")
        .join("SKILL.md");

    let skill = fs::read_to_string(&skill_path)
        .unwrap_or_else(|error| panic!("skill file should be readable at {skill_path:?}: {error}"));

    assert!(skill.contains("name: creative-idea-lab"));
    assert!(skill.contains("creative-idea-lab validate"));
}

#[test]
fn ingest_copies_raw_file_and_writes_metadata_sidecar() {
    let lab = temp_path("ingest-lab");
    let source_dir = temp_path("ingest-source");
    fs::create_dir_all(&source_dir).expect("source dir should be created");
    let source_file = source_dir.join("pi-session.jsonl");
    fs::write(&source_file, r#"{"id":"1","type":"user","text":"hello"}"#)
        .expect("source fixture should be written");

    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before ingest");

    let output = Command::new(cli())
        .arg("ingest")
        .arg(&lab)
        .arg(&source_file)
        .args(["--provider", "pi", "--source-type", "ai_conversation"])
        .output()
        .expect("failed to run ingest");

    assert!(
        output.status.success(),
        "ingest should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let raw_file = lab.join("00_raw-sessions").join("pi-session.jsonl");
    assert_exists(&raw_file);
    assert_eq!(
        fs::read_to_string(&raw_file).expect("raw file should be readable"),
        r#"{"id":"1","type":"user","text":"hello"}"#
    );

    let metadata = fs::read_to_string(
        lab.join("00_raw-sessions")
            .join("pi-session.jsonl.meta.json"),
    )
    .expect("metadata sidecar should be readable");
    assert!(metadata.contains(r#""provider": "pi""#));
    assert!(metadata.contains(r#""source_type": "ai_conversation""#));
    assert!(metadata.contains(r#""original_filename": "pi-session.jsonl""#));

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(source_dir).ok();
}

#[test]
fn note_new_creates_thought_draft_with_source_trace() {
    let lab = temp_path("thought-note");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before note new");

    let output = Command::new(cli())
        .args(["note", "new", "thought"])
        .arg(&lab)
        .args([
            "--title",
            "AI Content Fatigue",
            "--session",
            "[[01_sessions/example.md#^t0038]]",
            "--raw-file",
            "00_raw-sessions/example.txt",
        ])
        .output()
        .expect("failed to run note new thought");

    assert!(
        output.status.success(),
        "note new thought should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let note_path = lab.join("20_thoughts").join("ai-content-fatigue.md");
    assert_exists(&note_path);
    let note = fs::read_to_string(note_path).expect("thought note should be readable");
    assert!(note.contains("status: draft"));
    assert!(note.contains("user_confirmed: false"));
    assert!(note.contains("session: \"[[01_sessions/example.md#^t0038]]\""));
    assert!(note.contains("raw_file: \"00_raw-sessions/example.txt\""));
    assert!(note.contains("# AI Content Fatigue"));

    fs::remove_dir_all(lab).ok();
}

#[test]
fn note_new_creates_platform_post_draft() {
    let lab = temp_path("post-note");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before note new");

    let output = Command::new(cli())
        .args(["note", "new", "post"])
        .arg(&lab)
        .args([
            "--title",
            "Process Erasure",
            "--platform",
            "linkedin",
            "--session",
            "[[01_sessions/example.md#^t0040]]",
            "--raw-file",
            "00_raw-sessions/example.txt",
        ])
        .output()
        .expect("failed to run note new post");

    assert!(
        output.status.success(),
        "note new post should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let note_path = lab
        .join("40_posts")
        .join("linkedin")
        .join("process-erasure.md");
    assert_exists(&note_path);
    let note = fs::read_to_string(note_path).expect("post note should be readable");
    assert!(note.contains("platform: linkedin"));
    assert!(note.contains("session: \"[[01_sessions/example.md#^t0040]]\""));
    assert!(note.contains("## Review Checklist"));
    assert!(note.contains("# Process Erasure"));

    fs::remove_dir_all(lab).ok();
}
