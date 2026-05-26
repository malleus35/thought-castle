---
name: creative-idea-lab
description: Manage Creative Idea Lab vault structure, validate source traceability, and prepare agent workflows.
---

# Creative Idea Lab

Use this skill when working in a Creative Idea Lab vault or when creating one.

## Commands

```bash
creative-idea-lab init <path>
creative-idea-lab validate <path>
creative-idea-lab ingest <lab> <source-file> --provider <name> --source-type <type>
creative-idea-lab note new <knowledge|thought|idea|post> <lab> --title <title> --session <ref> --raw-file <path>
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

## Common Tasks

### Ingest a raw session

```bash
creative-idea-lab ingest . ~/Downloads/session.jsonl --provider pi --source-type ai_conversation
```

### Create a thought draft

```bash
creative-idea-lab note new thought . \
  --title "AI Content Fatigue" \
  --session "[[01_sessions/example.md#^t0038]]" \
  --raw-file "00_raw-sessions/example.txt"
```

### Create a LinkedIn post draft

```bash
creative-idea-lab note new post . \
  --title "Process Erasure" \
  --platform linkedin \
  --session "[[01_sessions/example.md#^t0040]]" \
  --raw-file "00_raw-sessions/example.txt"
```
