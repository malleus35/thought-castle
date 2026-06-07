# Knowledge Archive Scope

## Goal

Reposition Thought Castle as a local-first verified knowledge archive for LLM study sessions, not a social posting pipeline.

## Scope

- Document installation and everyday usage in `README.md`.
- Explain how Thought Castle differs from Karpathy's LLM Wiki pattern and graphify.
- Remove first-class post drafting from the CLI contract:
  - no `40_posts` directories in new vaults
  - no post template in new vaults
  - no `note new post` support
  - no post candidates in normalized sessions
- Keep raw session automation for Codex, Claude Code, OpenCode, and Pi Agent.
- Keep manual capture for ChatGPT, Claude, Perplexity, and other web or desktop sessions.

## Acceptance Checks

- Tests fail before implementation for the new archive-only contract.
- `cargo test` passes after implementation.
- README includes Homebrew install, source install, quickstart, source sync, manual ingest, verification workflow, and comparison sections.
- README does not present LinkedIn, X, or platform posting as a supported product goal.

## Non-Goals

- Do not implement graph extraction.
- Do not implement ChatGPT or Claude private app scraping.
- Do not publish or schedule social posts.
