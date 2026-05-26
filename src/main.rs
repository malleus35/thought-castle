use std::collections::hash_map::DefaultHasher;
use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
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
    (
        "_system/source-reference-schema.md",
        SOURCE_REFERENCE_SCHEMA,
    ),
    (
        "_system/status-transition-rules.md",
        STATUS_TRANSITION_RULES,
    ),
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
        [command, lab, source, rest @ ..] if command == "ingest" => {
            let flags = Flags::parse(rest)?;
            let provider = flags.required("--provider")?;
            let source_type = flags.required("--source-type")?;
            ingest_raw(Path::new(lab), Path::new(source), provider, source_type)
        }
        [command, subcommand, kind, lab, rest @ ..] if command == "note" && subcommand == "new" => {
            let flags = Flags::parse(rest)?;
            let title = flags.required("--title")?;
            let session = flags.required("--session")?;
            let raw_file = flags.required("--raw-file")?;
            let platform = flags.optional("--platform");
            create_note(Path::new(lab), kind, title, session, raw_file, platform)
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
            "expected: init <path> | validate <path> | ingest <lab> <source> --provider <name> --source-type <type> | note new <kind> <lab> --title <title> --session <ref> --raw-file <path> | skill print | skill install --target <path>"
                .to_string(),
        )),
    }
}

fn print_help() {
    println!(
        "creative-idea-lab\n\nCommands:\n  init <path>\n  validate <path>\n  ingest <lab> <source> --provider <name> --source-type <type>\n  note new <kind> <lab> --title <title> --session <ref> --raw-file <path>\n  skill print\n  skill install --target <path>"
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

fn ingest_raw(lab: &Path, source: &Path, provider: &str, source_type: &str) -> CliResult<()> {
    let raw_dir = lab.join("00_raw-sessions");
    fs::create_dir_all(&raw_dir).map_err(CliError::Io)?;

    let filename = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| CliError::Usage("source must have a valid file name".to_string()))?;
    let destination = raw_dir.join(filename);
    if destination.exists() {
        return Err(CliError::Usage(format!(
            "raw file already exists: {}",
            destination.display()
        )));
    }

    let bytes = fs::read(source).map_err(CliError::Io)?;
    fs::write(&destination, &bytes).map_err(CliError::Io)?;

    let metadata = format!(
        concat!(
            "{{\n",
            "  \"provider\": \"{}\",\n",
            "  \"source_type\": \"{}\",\n",
            "  \"original_filename\": \"{}\",\n",
            "  \"byte_len\": {},\n",
            "  \"content_hash\": \"{}\"\n",
            "}}\n"
        ),
        json_escape(provider),
        json_escape(source_type),
        json_escape(filename),
        bytes.len(),
        content_hash(&bytes)
    );
    fs::write(raw_dir.join(format!("{filename}.meta.json")), metadata).map_err(CliError::Io)?;

    println!("ingested: {}", destination.display());
    Ok(())
}

fn create_note(
    lab: &Path,
    kind: &str,
    title: &str,
    session: &str,
    raw_file: &str,
    platform: Option<&str>,
) -> CliResult<()> {
    let slug = slugify(title);
    let extraction_type = kind;
    let source_refs = format!(
        concat!(
            "source_refs:\n",
            "  - session: \"{}\"\n",
            "    raw_file: \"{}\"\n",
            "    extraction_type: {}\n",
            "    confidence: medium"
        ),
        yaml_escape(session),
        yaml_escape(raw_file),
        extraction_type
    );

    let (relative_dir, template) = match kind {
        "knowledge" => ("10_knowledge".to_string(), KNOWLEDGE_TEMPLATE),
        "thought" => ("20_thoughts".to_string(), THOUGHT_TEMPLATE),
        "idea" => ("30_ideas".to_string(), IDEA_TEMPLATE),
        "post" => {
            let platform = platform.unwrap_or("linkedin");
            (format!("40_posts/{platform}"), POST_TEMPLATE)
        }
        _ => {
            return Err(CliError::Usage(
                "note kind must be one of: knowledge, thought, idea, post".to_string(),
            ));
        }
    };

    let mut note = template.replace("source_refs: []", &source_refs);
    note = note.replace("# Title", &format!("# {title}"));
    if kind == "post" {
        let platform = platform.unwrap_or("linkedin");
        note = note.replace("platform:\n", &format!("platform: {platform}\n"));
    }

    let directory = lab.join(relative_dir);
    fs::create_dir_all(&directory).map_err(CliError::Io)?;
    let destination = directory.join(format!("{slug}.md"));
    write_new(&destination, &note)?;

    println!("created: {}", destination.display());
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

fn write_new(path: &Path, contents: &str) -> CliResult<()> {
    if path.exists() {
        return Err(CliError::Usage(format!(
            "refusing to overwrite existing file: {}",
            path.display()
        )));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(CliError::Io)?;
    }
    fs::write(path, contents).map_err(CliError::Io)
}

fn content_hash(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for character in input.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    let slug = slug.trim_matches('-').to_string();
    if slug.is_empty() {
        "note".to_string()
    } else {
        slug
    }
}

fn json_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

fn yaml_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

struct Flags<'a> {
    values: Vec<(&'a str, &'a str)>,
}

impl<'a> Flags<'a> {
    fn parse(args: &'a [String]) -> CliResult<Self> {
        let mut values = Vec::new();
        let mut chunks = args.chunks_exact(2);
        for chunk in &mut chunks {
            let flag = chunk[0].as_str();
            let value = chunk[1].as_str();
            if !flag.starts_with("--") {
                return Err(CliError::Usage(format!("expected flag, got: {flag}")));
            }
            values.push((flag, value));
        }
        if !chunks.remainder().is_empty() {
            return Err(CliError::Usage(
                "flags must be passed as --name value pairs".to_string(),
            ));
        }
        Ok(Self { values })
    }

    fn required(&self, name: &str) -> CliResult<&'a str> {
        self.optional(name)
            .ok_or_else(|| CliError::Usage(format!("missing required flag: {name}")))
    }

    fn optional(&self, name: &str) -> Option<&'a str> {
        self.values
            .iter()
            .find_map(|(flag, value)| (*flag == name).then_some(*value))
    }
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

const SKILL_MD: &str = include_str!("../skills/creative-idea-lab/SKILL.md");
