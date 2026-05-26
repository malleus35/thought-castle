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
