---
name: thought-castle
description: Manage a Thought Castle verified knowledge archive with raw session sync, source traceability, and evidence-gated notes.
---

# Thought Castle

Use this skill when working in a Thought Castle vault or when creating one.
Thought Castle is for preserving LLM study sessions and turning them into traceable, verified knowledge. It is not a publishing workflow.

## Commands

```bash
thought-castle init <path>
thought-castle validate <path>
thought-castle source list <lab> --provider <codex|claude-code|opencode|pi-agent> --root <path>
thought-castle sync <lab> --provider <codex|claude-code|opencode|pi-agent> --root <path>
thought-castle ingest <lab> <source-file> --provider <name> --source-type <type>
thought-castle ingest manual <lab> --provider <name> --title <title> --file <path>
thought-castle session normalize <lab> <raw-file> --title <title> --source-type <type>
thought-castle note new <knowledge|thought|idea> <lab> --title <title> --session <ref> --raw-file <path>
thought-castle skill print
thought-castle skill install --target <skills-dir>
```

## Operating Rules

- Keep raw sessions immutable in `00_raw-sessions`.
- Normalize sessions into `01_sessions` with stable block ids.
- Do not mark `10_knowledge` as `verified` without evidence.
- Do not mark `20_thoughts` as `stable` without user confirmation.
- Every derived note must include `source_refs`.
- Keep generated ideas separate from verified knowledge.

## Agent Workflow

1. Run `thought-castle validate .` before editing a lab.
2. Read `README.md` and the active plan in `.codex/plans/` when present.
3. Preserve source traceability when creating derived notes.
4. Use draft/candidate/raw statuses until the user approves promotion.

## Common Tasks

### List automatic source candidates

```bash
thought-castle source list . --provider codex --root ~/.codex/sessions
thought-castle source list . --provider claude-code --root ~/.claude/projects
thought-castle source list . --provider opencode --root ~/.local/share/opencode
thought-castle source list . --provider pi-agent --root ~/.pi/agent/sessions
```

### Sync automatic local sources

```bash
thought-castle sync . --provider codex --root ~/.codex/sessions
thought-castle sync . --provider claude-code --root ~/.claude/projects
thought-castle sync . --provider opencode --root ~/.local/share/opencode
thought-castle sync . --provider pi-agent --root ~/.pi/agent/sessions
```

### Ingest a raw session

```bash
thought-castle ingest . ~/Downloads/session.jsonl --provider pi --source-type ai_conversation
```

### Ingest a manual web or desktop capture

```bash
thought-castle ingest manual . \
  --provider chatgpt \
  --title "LLM Wiki Conversation" \
  --file ./thread.md
```

### Normalize a raw session

```bash
thought-castle session normalize . 00_raw-sessions/session.txt \
  --title "AI Content Fatigue Conversation" \
  --source-type ai_conversation
```

### Create a knowledge candidate

```bash
thought-castle note new knowledge . \
  --title "Central Limit Theorem" \
  --session "[[01_sessions/example.md#^t0038]]" \
  --raw-file "00_raw-sessions/example.txt"
```

### Create a thought draft

```bash
thought-castle note new thought . \
  --title "AI Content Fatigue" \
  --session "[[01_sessions/example.md#^t0040]]" \
  --raw-file "00_raw-sessions/example.txt"
```
