use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

type CliResult<T> = Result<T, CliError>;

const CORE_DIRS: &[&str] = &[
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
    "plans",
    "tasks",
    "subtasks",
];

const TEMPLATE_FILES: &[(&str, &str)] = &[
    ("_templates/10_knowledge.md", KNOWLEDGE_TEMPLATE),
    ("_templates/20_thought.md", THOUGHT_TEMPLATE),
    ("_templates/30_idea.md", IDEA_TEMPLATE),
    ("_templates/40_post.md", POST_TEMPLATE),
];

const SYSTEM_FILES: &[(&str, &str)] = &[
    ("_system/source-reference-schema.md", SOURCE_REFERENCE_SCHEMA),
    ("_system/status-transition-rules.md", STATUS_TRANSITION_RULES),
];

fn main() {
    if let Err(error) = run(env::args().skip(1)) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run(args: impl IntoIterator<Item = String>) -> CliResult<()> {
    let args: Vec<String> = args.into_iter().collect();
    match args.as_slice() {
        [] => {
            print_help();
            Ok(())
        }
        [command, path] if command == "init" => init_lab(Path::new(path)),
        [command, path] if command == "validate" => validate_lab(Path::new(path)).map(|report| {
            println!("valid: {}", report.summary());
        }),
        [command, subcommand] if command == "skill" && subcommand == "print" => {
            print!("{SKILL_MD}");
            Ok(())
        }
        [command, subcommand, flag, target]
            if command == "skill" && subcommand == "install" && flag == "--target" =>
        {
            install_skill(Path::new(target))
        }
        [command, subcommand, target]
            if command == "skill" && subcommand == "install" && !target.starts_with('-') =>
        {
            install_skill(Path::new(target))
        }
        _ => Err(CliError::Usage(
            "expected: init <path> | validate <path> | skill print | skill install --target <path>"
                .to_string(),
        )),
    }
}

fn print_help() {
    println!(
        "creative-idea-lab\n\nCommands:\n  init <path>\n  validate <path>\n  skill print\n  skill install --target <path>"
    );
}

fn init_lab(root: &Path) -> CliResult<()> {
    for directory in CORE_DIRS {
        fs::create_dir_all(root.join(directory)).map_err(CliError::Io)?;
    }

    for (relative_path, contents) in TEMPLATE_FILES.iter().chain(SYSTEM_FILES.iter()) {
        write_if_missing(&root.join(relative_path), contents)?;
    }

    write_if_missing(&root.join("README.md"), INIT_README)?;

    println!("initialized: {}", root.display());
    Ok(())
}

fn validate_lab(root: &Path) -> CliResult<ValidationReport> {
    let mut missing = Vec::new();
    let mut invalid = Vec::new();

    for directory in CORE_DIRS {
        let path = root.join(directory);
        if !path.is_dir() {
            missing.push(path);
        }
    }

    for (relative_path, _) in TEMPLATE_FILES {
        let path = root.join(relative_path);
        if !path.is_file() {
            missing.push(path);
            continue;
        }

        let contents = fs::read_to_string(&path).map_err(CliError::Io)?;
        if !contents.contains("source_refs: []") {
            invalid.push(format!("{} must contain source_refs: []", path.display()));
        }

        if *relative_path == "_templates/20_thought.md"
            && !contents.contains("user_confirmed: false")
        {
            invalid.push(format!(
                "{} must contain user_confirmed: false",
                path.display()
            ));
        }

        if *relative_path == "_templates/40_post.md" && !contents.contains("## Review Checklist") {
            invalid.push(format!(
                "{} must contain ## Review Checklist",
                path.display()
            ));
        }
    }

    if !missing.is_empty() || !invalid.is_empty() {
        return Err(CliError::Validation { missing, invalid });
    }

    Ok(ValidationReport {
        directories: CORE_DIRS.len(),
        templates: TEMPLATE_FILES.len(),
    })
}

fn install_skill(target: &Path) -> CliResult<()> {
    let skill_dir = target.join("creative-idea-lab");
    fs::create_dir_all(&skill_dir).map_err(CliError::Io)?;
    fs::write(skill_dir.join("SKILL.md"), SKILL_MD).map_err(CliError::Io)?;
    println!("installed: {}", skill_dir.join("SKILL.md").display());
    Ok(())
}

fn write_if_missing(path: &Path, contents: &str) -> CliResult<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(CliError::Io)?;
    }
    fs::write(path, contents).map_err(CliError::Io)
}

struct ValidationReport {
    directories: usize,
    templates: usize,
}

impl ValidationReport {
    fn summary(&self) -> String {
        format!(
            "{} directories, {} templates",
            self.directories, self.templates
        )
    }
}

#[derive(Debug)]
enum CliError {
    Io(io::Error),
    Usage(String),
    Validation {
        missing: Vec<PathBuf>,
        invalid: Vec<String>,
    },
}

impl fmt::Display for CliError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Usage(message) => write!(formatter, "{message}"),
            Self::Validation { missing, invalid } => {
                writeln!(formatter, "lab validation failed")?;
                for path in missing {
                    writeln!(formatter, "missing: {}", path.display())?;
                }
                for message in invalid {
                    writeln!(formatter, "invalid: {message}")?;
                }
                Ok(())
            }
        }
    }
}

impl Error for CliError {}

const INIT_README: &str = r#"# Creative Idea Lab

Generated by `creative-idea-lab init`.

Use `creative-idea-lab validate .` to check the structure.
"#;

const KNOWLEDGE_TEMPLATE: &str = r#"---
type: knowledge
status: candidate
domain:
topics: []
source_refs: []
verification:
  status: unverified
  method:
  evidence: []
  verified_at:
tags:
  - type/knowledge
---

# Title

## Claim

## Explanation

## Evidence

## Caveats

## Related Notes

## Source Trace
"#;

const THOUGHT_TEMPLATE: &str = r#"---
type: thought
status: draft
authorship: agent_extracted
user_confirmed: false
mood:
emotion_tags: []
affect_valence:
affect_arousal:
emotion_source: agent_inferred
emotion_confidence:
confidence:
source_refs: []
related_knowledge: []
tags:
  - type/thought
---

# Title

## Trigger

## My Interpretation

## Why It Matters

## Tension

## Related Knowledge

## Next Question

## Source Trace
"#;

const IDEA_TEMPLATE: &str = r#"---
type: idea
status: raw
method:
inputs:
  knowledge: []
  thoughts: []
source_refs: []
tags:
  - type/idea
---

# Title

## Input Materials

## Combination

## New Possibility

## Why It Is Non-Obvious

## Risks

## Next Experiment

## Review Notes

## Source Trace
"#;

const POST_TEMPLATE: &str = r#"---
type: post
platform:
status: draft
source_refs: []
linked_notes: []
published_url:
published_at:
tags:
  - type/post
---

# Title

## Core Message

## Audience

## Draft

## Platform Version

## Review Checklist

- [ ] Clear claim
- [ ] Concrete example
- [ ] No unsupported fact
- [ ] Source trace exists
- [ ] Platform length checked

## Published Result
"#;

const SOURCE_REFERENCE_SCHEMA: &str = r#"# Source Reference Schema

## Purpose

Every derived note must be traceable to a normalized session block and raw source file.

## YAML Shape

```yaml
source_refs:
  - session: "[[01_sessions/YYYY-MM-DD-session-slug.md#^t0001]]"
    raw_file: "00_raw-sessions/YYYY-MM-DD-source-id.ext"
    source_type: ai_conversation
    extraction_type: knowledge
    confidence: medium
```
"#;

const STATUS_TRANSITION_RULES: &str = r#"# Status Transition Rules

## Knowledge

```text
candidate -> needs_verification -> verified
candidate -> disputed
candidate -> discarded
```

## Thought

```text
draft -> reviewing -> stable
draft -> discarded
```

## Idea

```text
raw -> reviewing -> experimenting -> validated
raw -> discarded
```

## Post

```text
draft -> reviewing -> scheduled -> published
draft -> archived
```
"#;

const SKILL_MD: &str = r#"---
name: creative-idea-lab
description: Manage Creative Idea Lab vault structure, validate source traceability, and prepare agent workflows.
---

# Creative Idea Lab

Use this skill when working in a Creative Idea Lab vault or when creating one.

## Commands

```bash
creative-idea-lab init <path>
creative-idea-lab validate <path>
creative-idea-lab skill print
creative-idea-lab skill install --target <skills-dir>
```

## Operating Rules

- Keep raw sessions immutable in `00_raw-sessions`.
- Normalize sessions into `01_sessions` with stable block ids.
- Do not mark `10_knowledge` as `verified` without evidence.
- Do not mark `20_thoughts` as `stable` without user confirmation.
- Do not mark `40_posts` as `published` without a URL and date.
- Every derived note must include `source_refs`.

## Agent Workflow

1. Run `creative-idea-lab validate .` before editing a lab.
2. Read `plans/creative-idea-lab-prd.md`.
3. Preserve source traceability when creating derived notes.
4. Use draft/candidate/raw statuses until the user approves promotion.
"#;
