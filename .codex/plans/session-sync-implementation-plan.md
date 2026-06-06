# Session Sync Implementation Plan

## Goal

Implement the first Thought Castle session sync slice after finalizing the bilingual session-sync plans and adding an implementation PRD.

## Scope

- Update `plans/session-sync-knowledge-system-plan.en.md` and `.ko.md` so the final strategy is explicit:
  - automatic sync only for Codex, Claude Code, OpenCode, and Pi Agent
  - manual/export ingestion for ChatGPT, Claude web/desktop, Perplexity, consumer Pi, and arbitrary pasted/raw files
- Add a PRD for the first implementation slice.
- Implement TDD-first CLI support for:
  - `source list <lab> --provider <provider> --root <path>`
  - local JSONL discovery for Codex, Claude Code, and Pi Agent
  - local SQLite file discovery for OpenCode without reading conversation rows
  - manual raw-file ingestion command for web/desktop/export sources

## Non-Goals

- No provider-specific full session parsing in this slice.
- No ChatGPT/Claude/Perplexity web scraping.
- No background daemon or scheduler.
- No knowledge extraction or verification implementation yet.

## TDD Plan

1. Add failing contract tests for `source list` and `ingest manual`.
2. Run tests and confirm failure is due to missing commands.
3. Implement only enough CLI behavior to pass:
   - parse flags
   - discover files without printing conversation contents
   - create manual raw files and metadata sidecars
4. Run full test suite.
5. Commit RED and GREEN work separately when feasible.

## Acceptance Checks

- `cargo test` passes.
- `source list` reports candidate counts for Codex, Claude Code, OpenCode, and Pi Agent fixture roots.
- `source list` output does not include fixture conversation text.
- `ingest manual` writes a raw file and metadata sidecar under `00_raw-sessions/manual/`.
- README and skill docs expose the new commands.
