# Session Sync Implementation PRD

## Problem Statement

Thought Castle has a clear source strategy, but the CLI cannot yet discover local agent sessions or ingest manual web/desktop conversation captures as first-class raw sessions.

## Decision

Split ingestion into two lanes.

Automatic sync is limited to providers that already persist local session files or databases:

- Codex local JSONL
- Claude Code local JSONL
- OpenCode local SQLite database
- Pi Agent local JSONL

Manual/export ingestion handles web and desktop apps:

- ChatGPT export or copied conversation
- Claude export or copied conversation
- Perplexity export or copied conversation
- consumer Pi export/sample
- arbitrary Markdown/text/HTML/JSON raw files

## Goals

- Add read-only source discovery for local automatic providers.
- Add manual raw-session ingestion for web/desktop/export sources.
- Avoid reading or printing full conversation content during discovery.
- Keep raw sessions immutable after ingest.
- Preserve deterministic metadata sidecars.

## Non-Goals

- No full provider parser yet.
- No web or desktop app scraping.
- No background scheduler.
- No automatic knowledge extraction.
- No verification gate implementation beyond existing templates.

## CLI Requirements

### R1. Source List

`thought-castle source list <lab> --provider <provider> --root <path>` lists local source candidates without importing full conversation text.

Supported providers in this slice:

- `codex`
- `claude-code`
- `opencode`
- `pi-agent`

Expected output includes:

- provider name
- candidate count
- candidate file paths or database path

### R2. Source List Safety

`source list` must not print message bodies from JSONL files. It may inspect filenames, extensions, directory structure, and file metadata.

### R3. Manual Ingest

`thought-castle ingest manual <lab> --provider <provider> --title <title> --file <path>` copies the source file into `00_raw-sessions/manual/` and writes a `.meta.json` sidecar.

The sidecar must include:

- provider
- source_type: `manual_capture`
- original_filename
- title
- byte_len
- content_hash

### R4. Documentation

README and the installable skill must show the new commands.

## Test Requirements

- A Codex fixture root with nested `*.jsonl` sessions is counted.
- A Claude Code fixture root with project `*.jsonl` sessions is counted.
- A Pi Agent fixture root with path-shaped `*.jsonl` sessions is counted.
- An OpenCode fixture root with `opencode.db` is counted.
- Fixture message text is not printed by `source list`.
- Manual ingest copies a fixture file to `00_raw-sessions/manual/` and writes metadata.

## Risks

- Real provider formats may drift.
- OpenCode schema parsing is deferred, so this slice only verifies database file discovery.
- Provider roots outside the workspace must be supplied explicitly or approved by the user in future automation.

## Acceptance Checks

- `cargo test` passes.
- Existing CLI behavior remains unchanged.
- New source list and manual ingest commands are covered by contract tests.
