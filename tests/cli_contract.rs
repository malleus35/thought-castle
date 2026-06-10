use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn cli() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_thought-castle"))
}

fn temp_path(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("thought-castle-test-{name}-{suffix}"))
}

fn assert_exists(path: impl AsRef<Path>) {
    assert!(
        path.as_ref().exists(),
        "expected path to exist: {}",
        path.as_ref().display()
    );
}

fn assert_not_exists(path: impl AsRef<Path>) {
    assert!(
        !path.as_ref().exists(),
        "expected path not to exist: {}",
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
        .expect("failed to run thought-castle init");

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
        "_templates",
        "_system",
    ] {
        assert_exists(target.join(directory));
    }
    assert_not_exists(target.join("40_posts"));

    for template in [
        "_templates/10_knowledge.md",
        "_templates/20_thought.md",
        "_templates/30_idea.md",
    ] {
        assert_exists(target.join(template));
    }
    assert_not_exists(target.join("_templates/40_post.md"));

    let thought_template = fs::read_to_string(target.join("_templates/20_thought.md"))
        .expect("thought template should be readable");
    assert!(thought_template.contains("source_refs: []"));
    assert!(thought_template.contains("user_confirmed: false"));

    let source_schema = fs::read_to_string(target.join("_system/source-reference-schema.md"))
        .expect("source schema should be readable");
    assert!(source_schema.contains("knowledge`, `thought`, or `idea"));
    assert!(!source_schema.contains("post"));

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
fn help_and_version_flags_are_supported() {
    for flag in ["--help", "-h"] {
        let output = Command::new(cli())
            .arg(flag)
            .output()
            .unwrap_or_else(|error| panic!("failed to run thought-castle {flag}: {error}"));

        assert!(
            output.status.success(),
            "{flag} should succeed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("thought-castle"));
        assert!(stdout.contains("Commands:"));
    }

    let version = Command::new(cli())
        .arg("--version")
        .output()
        .expect("failed to run thought-castle --version");

    assert!(
        version.status.success(),
        "--version should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&version.stdout),
        String::from_utf8_lossy(&version.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&version.stdout),
        "thought-castle 0.1.3\n"
    );
}

#[test]
fn flags_support_equals_syntax_and_reject_unknown_or_duplicate_names() {
    let lab = temp_path("flags-lab");
    let root = temp_path("flags-root");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(
        init.status.success(),
        "init should succeed before flag tests"
    );
    fs::create_dir_all(&root).expect("source root should be created");

    let equals_output = Command::new(cli())
        .args(["source", "list"])
        .arg(&lab)
        .arg("--provider=codex")
        .arg(format!("--root={}", root.display()))
        .output()
        .expect("failed to run source list with equals flags");

    assert!(
        equals_output.status.success(),
        "--flag=value syntax should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&equals_output.stdout),
        String::from_utf8_lossy(&equals_output.stderr)
    );

    let unknown_output = Command::new(cli())
        .args(["note", "new", "thought"])
        .arg(&lab)
        .args([
            "--title",
            "Unknown Flag",
            "--session",
            "[[01_sessions/example.md#^t0001]]",
            "--raw-file",
            "00_raw-sessions/example.txt",
            "--unknown",
            "value",
        ])
        .output()
        .expect("failed to run note new with unknown flag");

    assert!(
        !unknown_output.status.success(),
        "unknown flag should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&unknown_output.stdout),
        String::from_utf8_lossy(&unknown_output.stderr)
    );
    assert!(String::from_utf8_lossy(&unknown_output.stderr).contains("unknown flag: --unknown"));

    let duplicate_output = Command::new(cli())
        .args(["note", "new", "thought"])
        .arg(&lab)
        .args([
            "--title",
            "First",
            "--title",
            "Second",
            "--session",
            "[[01_sessions/example.md#^t0001]]",
            "--raw-file",
            "00_raw-sessions/example.txt",
        ])
        .output()
        .expect("failed to run note new with duplicate flag");

    assert!(
        !duplicate_output.status.success(),
        "duplicate flag should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&duplicate_output.stdout),
        String::from_utf8_lossy(&duplicate_output.stderr)
    );
    assert!(String::from_utf8_lossy(&duplicate_output.stderr).contains("duplicate flag: --title"));

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(root).ok();
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
    assert!(stdout.contains("name: thought-castle"));
    assert!(stdout.contains("# Thought Castle"));
}

#[test]
fn skill_install_writes_skill_to_explicit_target() {
    let target = temp_path("skills");
    let home = temp_path("explicit-home");
    let codex_home = temp_path("explicit-codex-home");
    fs::create_dir_all(&home).expect("home fixture should be created");
    fs::create_dir_all(&codex_home).expect("codex home fixture should be created");

    let output = Command::new(cli())
        .args(["skill", "install", "--target"])
        .arg(&target)
        .env("HOME", &home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("failed to run skill install");

    assert!(
        output.status.success(),
        "skill install should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let skill_path = target.join("thought-castle").join("SKILL.md");
    assert_exists(&skill_path);

    let skill = fs::read_to_string(skill_path).expect("installed skill should be readable");
    assert!(skill.contains("name: thought-castle"));
    assert!(skill.contains("thought-castle validate"));
    assert_not_exists(home.join(".pi").join("agent").join("skills"));
    assert_not_exists(home.join(".claude").join("skills"));
    assert_not_exists(codex_home.join("skills"));
    assert_not_exists(home.join(".agents").join("skills"));

    fs::remove_dir_all(target).ok();
    fs::remove_dir_all(home).ok();
    fs::remove_dir_all(codex_home).ok();
}

#[test]
fn skill_install_without_target_installs_to_default_agent_skill_dirs() {
    let home = temp_path("home");
    let codex_home = temp_path("codex-home");
    fs::create_dir_all(&home).expect("home fixture should be created");
    fs::create_dir_all(&codex_home).expect("codex home fixture should be created");

    let output = Command::new(cli())
        .args(["skill", "install"])
        .env("HOME", &home)
        .env("CODEX_HOME", &codex_home)
        .output()
        .expect("failed to run skill install without target");

    assert!(
        output.status.success(),
        "skill install should succeed with default target\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let expected_targets = [
        home.join(".pi").join("agent").join("skills"),
        home.join(".claude").join("skills"),
        codex_home.join("skills"),
        home.join(".agents").join("skills"),
    ];

    for target in expected_targets {
        let skill_path = target.join("thought-castle").join("SKILL.md");
        assert_exists(&skill_path);
        let skill = fs::read_to_string(skill_path).expect("installed skill should be readable");
        assert!(skill.contains("name: thought-castle"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(".pi/agent/skills"));
    assert!(stdout.contains(".claude/skills"));
    assert!(stdout.contains("codex-home"));
    assert!(stdout.contains(".agents/skills"));

    fs::remove_dir_all(home).ok();
    fs::remove_dir_all(codex_home).ok();
}

#[test]
fn packaged_skill_file_is_available_for_agent_installers() {
    let skill_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("skills")
        .join("thought-castle")
        .join("SKILL.md");

    let skill = fs::read_to_string(&skill_path)
        .unwrap_or_else(|error| panic!("skill file should be readable at {skill_path:?}: {error}"));

    assert!(skill.contains("name: thought-castle"));
    assert!(skill.contains("thought-castle validate"));
    assert!(skill.contains("thought-castle ingest"));
    assert!(skill.contains("thought-castle source list"));
    assert!(skill.contains("thought-castle sync"));
    assert!(skill.contains("thought-castle ingest manual"));
    assert!(skill.contains("thought-castle note new"));
    assert!(!skill.contains("note new post"));
    assert!(!skill.contains("40_posts"));
}

#[test]
fn readme_documents_installation_archive_scope_and_usage() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(&readme_path)
        .unwrap_or_else(|error| panic!("README should be readable at {readme_path:?}: {error}"));

    assert!(readme.contains("brew install malleus35/tap/thought-castle"));
    assert!(readme.contains("verified knowledge archive"));
    assert!(readme.contains("thought-castle source list"));
    assert!(readme.contains("thought-castle sync"));
    assert!(readme.contains("thought-castle ingest manual"));
    assert!(readme.contains("thought-castle skill install"));
    assert!(readme.contains("~/.pi/agent/skills/"));
    assert!(readme.contains("~/.claude/skills/"));
    assert!(readme.contains("${CODEX_HOME:-~/.codex}/skills/"));
    assert!(readme.contains("~/.agents/skills/"));
    assert!(readme.contains("Karpathy LLM Wiki"));
    assert!(readme.contains("graphify"));
    assert!(!readme.contains("40_posts"));
    assert!(!readme.contains("LinkedIn"));
    assert!(!readme.contains("x.com"));
}

#[test]
fn readme_delegates_archive_intake_to_the_agent_skill_after_vault_creation() {
    let readme_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let readme = fs::read_to_string(&readme_path)
        .unwrap_or_else(|error| panic!("README should be readable at {readme_path:?}: {error}"));

    assert!(readme.contains("After Install And Vault Creation"));
    assert!(readme.contains("ask an agent to run the Thought Castle archive intake workflow"));
    assert!(readme.contains("paste a copied transcript"));
    assert!(readme.contains("sync automatic local sessions"));
    assert!(readme.contains("normalize new raw sessions"));
}

#[test]
fn repository_defines_ci_for_rust_quality_gates() {
    let workflow_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows")
        .join("ci.yml");
    let workflow = fs::read_to_string(&workflow_path).unwrap_or_else(|error| {
        panic!("CI workflow should be readable at {workflow_path:?}: {error}")
    });

    assert!(workflow.contains("cargo fmt --check"));
    assert!(workflow.contains("cargo clippy --all-targets -- -D warnings"));
    assert!(workflow.contains("cargo test"));
    assert!(workflow.contains("ubuntu-latest"));
    assert!(workflow.contains("macos-latest"));
}

#[test]
fn packaged_skill_defines_archive_intake_workflow_for_agents() {
    let skill_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("skills")
        .join("thought-castle")
        .join("SKILL.md");

    let skill = fs::read_to_string(&skill_path)
        .unwrap_or_else(|error| panic!("skill file should be readable at {skill_path:?}: {error}"));

    assert!(skill.contains("Archive Intake Run"));
    assert!(skill.contains("The user owns installation and vault creation"));
    assert!(skill.contains("sync automatic local sessions"));
    assert!(skill.contains("Manual Paste Capture"));
    assert!(skill.contains("Save the pasted transcript"));
    assert!(skill.contains("thought-castle ingest manual"));
    assert!(skill.contains("thought-castle session normalize"));
    assert!(skill.contains("thought-castle note new knowledge"));
    assert!(skill.contains("thought-castle note new thought"));
    assert!(skill.contains("thought-castle note new idea"));
    assert!(skill.contains("Do not mark knowledge as `verified`"));
    assert!(skill.contains("Do not mark thoughts as `stable`"));
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
fn source_list_discovers_local_agent_sessions_without_printing_message_text() {
    let lab = temp_path("source-list-lab");
    let root = temp_path("source-list-root");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(
        init.status.success(),
        "init should succeed before source list"
    );

    let codex_dir = root.join("codex").join("2026").join("06").join("07");
    fs::create_dir_all(&codex_dir).expect("codex fixture dir should be created");
    fs::write(
        codex_dir.join("rollout-2026-06-07T00-00-00-session.jsonl"),
        r#"{"type":"message","text":"SECRET_MESSAGE_DO_NOT_PRINT"}"#,
    )
    .expect("codex fixture should be written");

    let claude_dir = root.join("claude-code").join("-Users-macbook-workspace");
    fs::create_dir_all(&claude_dir).expect("claude-code fixture dir should be created");
    fs::write(
        claude_dir.join("11111111-1111-4111-8111-111111111111.jsonl"),
        r#"{"type":"assistant","text":"SECRET_MESSAGE_DO_NOT_PRINT"}"#,
    )
    .expect("claude-code fixture should be written");

    let pi_dir = root.join("pi-agent").join("--Users--macbook--workspace--");
    fs::create_dir_all(&pi_dir).expect("pi-agent fixture dir should be created");
    fs::write(
        pi_dir.join("2026-06-07T00-00-00_22222222-2222-4222-8222-222222222222.jsonl"),
        r#"{"type":"message","content":"SECRET_MESSAGE_DO_NOT_PRINT"}"#,
    )
    .expect("pi-agent fixture should be written");

    for (provider, provider_root, expected_name) in [
        (
            "codex",
            root.join("codex"),
            "rollout-2026-06-07T00-00-00-session.jsonl",
        ),
        (
            "claude-code",
            root.join("claude-code"),
            "11111111-1111-4111-8111-111111111111.jsonl",
        ),
        (
            "pi-agent",
            root.join("pi-agent"),
            "2026-06-07T00-00-00_22222222-2222-4222-8222-222222222222.jsonl",
        ),
    ] {
        let output = Command::new(cli())
            .args(["source", "list"])
            .arg(&lab)
            .args(["--provider", provider, "--root"])
            .arg(&provider_root)
            .output()
            .unwrap_or_else(|error| panic!("failed to run source list for {provider}: {error}"));

        assert!(
            output.status.success(),
            "source list should succeed for {provider}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains(&format!("provider: {provider}")));
        assert!(stdout.contains("candidates: 1"));
        assert!(stdout.contains(expected_name));
        assert!(!stdout.contains("SECRET_MESSAGE_DO_NOT_PRINT"));
    }

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(root).ok();
}

#[test]
fn source_list_discovers_opencode_database_without_reading_rows() {
    let lab = temp_path("opencode-list-lab");
    let root = temp_path("opencode-list-root");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(
        init.status.success(),
        "init should succeed before source list"
    );

    fs::create_dir_all(&root).expect("opencode fixture root should be created");
    fs::write(root.join("opencode.db"), b"SECRET_MESSAGE_DO_NOT_PRINT")
        .expect("opencode db fixture should be written");

    let output = Command::new(cli())
        .args(["source", "list"])
        .arg(&lab)
        .args(["--provider", "opencode", "--root"])
        .arg(&root)
        .output()
        .expect("failed to run opencode source list");

    assert!(
        output.status.success(),
        "source list should succeed for opencode\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("provider: opencode"));
    assert!(stdout.contains("candidates: 1"));
    assert!(stdout.contains("opencode.db"));
    assert!(!stdout.contains("SECRET_MESSAGE_DO_NOT_PRINT"));

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(root).ok();
}

#[test]
fn ingest_manual_copies_web_or_desktop_capture_with_metadata() {
    let lab = temp_path("manual-ingest-lab");
    let source_dir = temp_path("manual-ingest-source");
    fs::create_dir_all(&source_dir).expect("source dir should be created");
    let source_file = source_dir.join("chatgpt-thread.md");
    fs::write(&source_file, "# Thread\n\nmanual capture")
        .expect("manual fixture should be written");

    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(
        init.status.success(),
        "init should succeed before manual ingest"
    );

    let output = Command::new(cli())
        .args(["ingest", "manual"])
        .arg(&lab)
        .args([
            "--provider",
            "chatgpt",
            "--title",
            "Manual ChatGPT Thread",
            "--file",
        ])
        .arg(&source_file)
        .output()
        .expect("failed to run ingest manual");

    assert!(
        output.status.success(),
        "ingest manual should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let raw_file = lab
        .join("00_raw-sessions")
        .join("manual")
        .join("chatgpt-thread.md");
    assert_exists(&raw_file);
    assert_eq!(
        fs::read_to_string(&raw_file).expect("manual raw file should be readable"),
        "# Thread\n\nmanual capture"
    );

    let metadata = fs::read_to_string(raw_file.with_file_name("chatgpt-thread.md.meta.json"))
        .expect("manual metadata should be readable");
    assert!(metadata.contains(r#""provider": "chatgpt""#));
    assert!(metadata.contains(r#""source_type": "manual_capture""#));
    assert!(metadata.contains(r#""title": "Manual ChatGPT Thread""#));

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(source_dir).ok();
}

#[test]
fn ingest_manual_escapes_control_characters_in_metadata_json() {
    let lab = temp_path("manual-ingest-escape-lab");
    let source_dir = temp_path("manual-ingest-escape-source");
    fs::create_dir_all(&source_dir).expect("source dir should be created");
    let source_file = source_dir.join("chatgpt-thread.md");
    fs::write(&source_file, "# Thread\n\nmanual capture")
        .expect("manual fixture should be written");

    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(
        init.status.success(),
        "init should succeed before manual ingest"
    );

    let output = Command::new(cli())
        .args(["ingest", "manual"])
        .arg(&lab)
        .args(["--provider", "chatgpt", "--title"])
        .arg("line1\nline2\t\"quote\"\\slash\u{0001}")
        .args(["--file"])
        .arg(&source_file)
        .output()
        .expect("failed to run ingest manual");

    assert!(
        output.status.success(),
        "ingest manual should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata = fs::read_to_string(
        lab.join("00_raw-sessions")
            .join("manual")
            .join("chatgpt-thread.md.meta.json"),
    )
    .expect("manual metadata should be readable");
    assert!(metadata.contains(r#""title": "line1\nline2\t\"quote\"\\slash\u0001""#));
    assert!(!metadata.contains("line1\nline2"));

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(source_dir).ok();
}

#[test]
fn sync_copies_automatic_agent_sessions_and_is_idempotent() {
    let lab = temp_path("sync-agent-lab");
    let root = temp_path("sync-agent-root");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before sync");

    let codex_dir = root.join("codex").join("2026").join("06").join("07");
    fs::create_dir_all(&codex_dir).expect("codex fixture dir should be created");
    fs::write(
        codex_dir.join("rollout-2026-06-07T00-00-00-session.jsonl"),
        r#"{"type":"message","text":"sync me"}"#,
    )
    .expect("codex fixture should be written");

    let output = Command::new(cli())
        .arg("sync")
        .arg(&lab)
        .args(["--provider", "codex", "--root"])
        .arg(root.join("codex"))
        .output()
        .expect("failed to run codex sync");

    assert!(
        output.status.success(),
        "sync should succeed for codex\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("synced: 1"));

    let raw_file = lab
        .join("00_raw-sessions")
        .join("codex")
        .join("rollout-2026-06-07T00-00-00-session.jsonl");
    assert_exists(&raw_file);
    let metadata = fs::read_to_string(
        lab.join("00_raw-sessions")
            .join("codex")
            .join("rollout-2026-06-07T00-00-00-session.jsonl.meta.json"),
    )
    .expect("sync metadata should be readable");
    assert!(metadata.contains(r#""provider": "codex""#));
    assert!(metadata.contains(r#""source_type": "automatic_session""#));

    let rerun = Command::new(cli())
        .arg("sync")
        .arg(&lab)
        .args(["--provider", "codex", "--root"])
        .arg(root.join("codex"))
        .output()
        .expect("failed to rerun codex sync");

    assert!(
        rerun.status.success(),
        "sync rerun should succeed for codex\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&rerun.stdout),
        String::from_utf8_lossy(&rerun.stderr)
    );
    assert!(String::from_utf8_lossy(&rerun.stdout).contains("synced: 0"));

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(root).ok();
}

#[test]
fn sync_copies_opencode_database_snapshot() {
    let lab = temp_path("sync-opencode-lab");
    let root = temp_path("sync-opencode-root");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before sync");

    fs::create_dir_all(&root).expect("opencode fixture root should be created");
    fs::write(root.join("opencode.db"), b"sqlite snapshot")
        .expect("opencode fixture should be written");

    let output = Command::new(cli())
        .arg("sync")
        .arg(&lab)
        .args(["--provider", "opencode", "--root"])
        .arg(&root)
        .output()
        .expect("failed to run opencode sync");

    assert!(
        output.status.success(),
        "sync should succeed for opencode\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let raw_file = lab
        .join("00_raw-sessions")
        .join("opencode")
        .join("opencode.db");
    assert_exists(&raw_file);
    assert_eq!(
        fs::read(&raw_file).expect("opencode raw db should be readable"),
        b"sqlite snapshot"
    );

    fs::remove_dir_all(lab).ok();
    fs::remove_dir_all(root).ok();
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
fn note_new_preserves_korean_title_in_slug_and_adds_suffix_for_duplicates() {
    let lab = temp_path("korean-note");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before note new");

    for expected_path in [
        lab.join("10_knowledge").join("중심극한정리.md"),
        lab.join("10_knowledge").join("중심극한정리-2.md"),
    ] {
        let output = Command::new(cli())
            .args(["note", "new", "knowledge"])
            .arg(&lab)
            .args([
                "--title",
                "중심극한정리",
                "--session",
                "[[01_sessions/example.md#^t0001]]",
                "--raw-file",
                "00_raw-sessions/example.txt",
            ])
            .output()
            .expect("failed to run note new knowledge");

        assert!(
            output.status.success(),
            "note new knowledge should succeed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_exists(expected_path);
    }

    assert_not_exists(lab.join("10_knowledge").join("note.md"));

    fs::remove_dir_all(lab).ok();
}

#[test]
fn note_new_escapes_control_characters_in_source_refs_yaml() {
    let lab = temp_path("note-yaml-escape");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(init.status.success(), "init should succeed before note new");

    let output = Command::new(cli())
        .args(["note", "new", "knowledge"])
        .arg(&lab)
        .args(["--title", "Escaped Trace", "--session"])
        .arg("[[01_sessions/example.md#^t0001]]\nline2\t\"quote\"\\slash\u{0001}")
        .args(["--raw-file"])
        .arg("00_raw-sessions/example.txt\nline2\t\"quote\"\\slash\u{0001}")
        .output()
        .expect("failed to run note new knowledge");

    assert!(
        output.status.success(),
        "note new knowledge should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let note_path = lab.join("10_knowledge").join("escaped-trace.md");
    assert_exists(&note_path);
    let note = fs::read_to_string(note_path).expect("knowledge note should be readable");
    assert!(note.contains(
        r#"session: "[[01_sessions/example.md#^t0001]]\nline2\t\"quote\"\\slash\u0001""#
    ));
    assert!(
        note.contains(r#"raw_file: "00_raw-sessions/example.txt\nline2\t\"quote\"\\slash\u0001""#)
    );
    assert!(!note.contains("[[01_sessions/example.md#^t0001]]\nline2"));

    fs::remove_dir_all(lab).ok();
}

#[test]
fn note_new_rejects_post_kind() {
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
            "--session",
            "[[01_sessions/example.md#^t0040]]",
            "--raw-file",
            "00_raw-sessions/example.txt",
        ])
        .output()
        .expect("failed to run note new post");

    assert!(
        !output.status.success(),
        "note new post should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("note kind must be one of: knowledge, thought, idea")
    );
    assert_not_exists(lab.join("40_posts"));

    fs::remove_dir_all(lab).ok();
}

#[test]
fn session_normalize_preserves_korean_title_in_slug() {
    let lab = temp_path("korean-session-normalize");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(
        init.status.success(),
        "init should succeed before normalize"
    );

    let raw_file = lab.join("00_raw-sessions").join("korean-session.txt");
    fs::write(&raw_file, "베이즈 정리를 설명해줘.").expect("raw fixture should be written");

    let output = Command::new(cli())
        .args(["session", "normalize"])
        .arg(&lab)
        .arg(&raw_file)
        .args(["--title", "베이즈 정리", "--source-type", "ai_conversation"])
        .output()
        .expect("failed to run session normalize");

    assert!(
        output.status.success(),
        "session normalize should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let session_path = lab.join("01_sessions").join("베이즈-정리.md");
    assert_exists(&session_path);
    let session = fs::read_to_string(session_path).expect("session should be readable");
    assert!(session.contains("# 베이즈 정리"));
    assert!(session.contains("### t0001 source ^t0001"));

    fs::remove_dir_all(lab).ok();
}

#[test]
fn session_normalize_creates_canonical_markdown_with_block_id() {
    let lab = temp_path("session-normalize");
    let init = Command::new(cli())
        .arg("init")
        .arg(&lab)
        .output()
        .expect("failed to run init");
    assert!(
        init.status.success(),
        "init should succeed before normalize"
    );

    let raw_file = lab.join("00_raw-sessions").join("manual-session.txt");
    fs::write(
        &raw_file,
        "중심극한정리가 뭐지?\n\n표본 평균의 분포 이야기다.",
    )
    .expect("raw fixture should be written");

    let output = Command::new(cli())
        .args(["session", "normalize"])
        .arg(&lab)
        .arg(&raw_file)
        .args([
            "--title",
            "CLT Conversation",
            "--source-type",
            "ai_conversation",
        ])
        .output()
        .expect("failed to run session normalize");

    assert!(
        output.status.success(),
        "session normalize should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let session_path = lab.join("01_sessions").join("clt-conversation.md");
    assert_exists(&session_path);
    let session = fs::read_to_string(session_path).expect("session should be readable");
    assert!(session.contains("type: session"));
    assert!(session.contains("source_type: ai_conversation"));
    assert!(session.contains("raw_file: 00_raw-sessions/manual-session.txt"));
    assert!(session.contains("### t0001 source ^t0001"));
    assert!(session.contains("중심극한정리가 뭐지?"));
    assert!(!session.contains("Post Candidates"));

    fs::remove_dir_all(lab).ok();
}
