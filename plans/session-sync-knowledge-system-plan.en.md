# Thought Castle Session Sync and Verifiable Knowledge Plan

## Purpose

This plan captures the current research and implementation direction for turning Thought Castle into a local-first system that ingests LLM conversation sessions, normalizes them into stable session records, and compiles them into a verifiable personal knowledge base.

The product direction is now broader than an idea generator. Idea generation remains important, but the primary objective is to build a trusted knowledge substrate from the user's real learning conversations.

## Current Product State

Thought Castle already defines a five-layer pipeline:

```text
00_raw-sessions  -> immutable source capture
01_sessions      -> normalized Markdown sessions with stable block references
10_knowledge     -> objective knowledge candidates and verified facts
20_thoughts      -> subjective understanding, judgments, and personal context
30_ideas         -> creative combinations and experiment candidates
40_posts         -> platform-specific drafts and publication records
```

The Rust CLI currently supports:

- `init`: create the vault skeleton, templates, and system docs.
- `validate`: verify required folders/templates and basic invariants.
- `ingest`: copy a raw file into `00_raw-sessions` and write a metadata sidecar.
- `session normalize`: create a minimal canonical Markdown session with a stable block id.
- `note new`: create `knowledge`, `thought`, `idea`, or `post` drafts with `source_refs`.
- `skill print` and `skill install`: expose an installable Thought Castle agent skill.

The current tests pass, but provider-specific session sync, topic routing, claim extraction, evidence verification, and graph integration are not implemented yet.

## Research Summary

### ChatGPT

For consumer ChatGPT accounts, the official supported path is data export through ChatGPT Settings/Data Controls or the OpenAI Privacy Portal. OpenAI's help page says the downloaded zip includes chat history and other account data, and that exports may take time to arrive.

The ChatGPT macOS app follows the web product's data retention policy. OpenAI's macOS data-retention page says files uploaded through the macOS app are stored in the cloud, not locally, and tied to the OpenAI account.

In 2024, the ChatGPT macOS app was reported to store local conversations in plaintext at `~/Library/Application Support/com.openai.chat`; that was changed after the disclosure, and current local desktop storage should not be treated as a stable public interface.

Enterprise/Edu workspaces have a Compliance Platform/Compliance API that can provide logs and metadata from ChatGPT workspaces, but this is not available for normal consumer accounts.

Implication: for consumer ChatGPT, reliable automation should start with export zip ingestion. Live periodic sync is only possible through unofficial browser automation, browser extensions, or a future first-party API. Those options are brittle and must be treated as experimental.

### Claude Web and Claude Desktop

Claude supports official data export for individual accounts through Settings/Privacy in the web app or Claude Desktop. The export includes conversation data and account data.

Claude Desktop has additional local agent/Cowork storage. Anthropic documents `local-agent-mode-sessions/` as Cowork conversation history and `claude-code-sessions/` as Code tab conversation history. `IndexedDB`, `Local Storage`, and `Session Storage` are documented as renderer-side UI state, not the main source of conversation truth.

On this machine, `~/Library/Application Support/Claude` exists and includes `IndexedDB`, `Local Storage`, `local-agent-mode-sessions`, `claude-code`, and `claude-code-vm`.

Implication: Claude web history should use official export first. Claude Desktop Cowork/3P sessions can be considered for local read-only sync if the user explicitly approves reading those local session directories.

### Claude Code

Claude Code has a strong local sync path. Anthropic documents that each message, tool use, and result is written to plaintext JSONL under `~/.claude/projects/`.

On this machine, `~/.claude/projects` contains many JSONL session files.

Implication: Claude Code should be a first-class automatic sync adapter.

### Codex

On this machine, Codex session data exists under `~/.codex/sessions/2026/.../*.jsonl`, with additional index/history files such as `~/.codex/session_index.jsonl` and `~/.codex/history.jsonl`.

Implication: Codex should be a first-class automatic sync adapter. The implementation should read only file metadata during discovery and then ingest session files through explicit sync commands.

### OpenCode

On this machine, OpenCode local data exists under `~/.local/share/opencode/opencode.db`.

Implication: OpenCode should be handled as a read-only SQLite adapter. The first implementation should inspect schema only, then add a parser behind a failing test using fixture data.

### Pi Agent

Pi Agent has a strong local sync path. The Pi sessions documentation says sessions auto-save to `~/.pi/agent/sessions/`, organized by working directory. Each session is a JSONL file with a tree structure. The session format documentation also specifies the concrete path shape: `~/.pi/agent/sessions/--<path>--/<timestamp>_<uuid>.jsonl`.

Pi session files include a header, messages, model changes, thinking-level changes, compactions, branch summaries, custom entries, labels, and tree links via `id` and `parentId`.

Implication: Pi Agent should be a first-class automatic sync adapter.

### Consumer Pi Chatbot

No local consumer Pi macOS application or obvious local `heypi`/Inflection storage was found on this machine. Public search did not reveal a stable first-party local sync API. Export or sample capture remains the likely route.

Implication: consumer Pi should remain in the manual/sample-needed category until an export format or stable storage path is confirmed.

## Is Automatic Sync Impossible?

No. It is not impossible, but the sync strategy must vary by provider.

## Routing Decision

Thought Castle should split ingestion into two lanes.

### Automatic Lane

Only automate sources that already write durable local session files or databases:

- Codex local JSONL sessions.
- Claude Code local JSONL sessions.
- OpenCode local SQLite sessions.
- Pi Agent local JSONL sessions.

This lane can support scheduled sync, `thought-castle sync`, and later background watchers because the source of truth is already local and persistent.

### Manual or Export Lane

Web and desktop apps should enter Thought Castle through explicit user action:

- official export zip ingestion for ChatGPT and Claude
- downloaded or exported Perplexity threads when available
- copy/paste capture for single conversations
- manual raw session files such as Markdown, text, HTML, JSON, PDF, or DOCX

This lane should still create raw files, metadata sidecars, normalized sessions, and source refs. It should not pretend to be live automatic sync.

### Feasible and Preferred

- Claude Code: local JSONL sync from `~/.claude/projects`.
- Codex: local JSONL sync from `~/.codex/sessions`.
- OpenCode: read-only SQLite sync from `~/.local/share/opencode/opencode.db`.
- Pi Agent: local JSONL sync from `~/.pi/agent/sessions`.
- Claude Desktop Cowork/3P: local directory sync from documented `local-agent-mode-sessions`, only after explicit user approval.

### Feasible but Manual or Semi-Automatic

- ChatGPT consumer: official export zip ingestion.
- Claude web/standard account: official export zip ingestion.
- Perplexity: user-downloaded export or manual copy/paste ingestion.
- Consumer Pi chatbot: export/sample ingestion once format is available.

### Possible but Experimental

- Browser extension or local browser automation that exports active ChatGPT/Claude conversations while the user is logged in.
- Share-link based capture for selected ChatGPT conversations.
- UI scraping of the web sidebar/conversation pages.

These are not impossible, but they are fragile. They may break when the UI changes, may miss hidden/paginated messages, may conflict with platform terms, and require stricter privacy controls. They should not be the core ingestion path.

### Strategic Alternative

For future conversations, the most reliable capture method is forward capture: use a Thought Castle CLI/skill/MCP workflow as the entry point for learning sessions. The system records the conversation as it happens, then submits prompts to the target LLM/API or helps the user paste the output back. This avoids depending on private web-app storage.

## Comparison With Karpathy LLM Wiki

Karpathy's LLM Wiki pattern has three layers:

- immutable raw sources
- LLM-maintained Markdown wiki pages
- a schema/instructions file that governs ingest, query, and lint workflows

The key idea is compilation: the LLM does not re-derive knowledge from raw sources on every question; it maintains a persistent wiki that compounds over time.

Thought Castle should adopt this compounding write-back principle, but with stricter personal epistemology:

- `10_knowledge` is not "truth"; it starts as `candidate`.
- `verified` requires evidence.
- `20_thoughts` separates the user's interpretation from objective claims.
- `30_ideas` is derived from knowledge and thought, not the primary source of truth.
- every derived note must preserve source traceability.

In short: Karpathy LLM Wiki is a maintained knowledge wiki. Thought Castle should become a verified learning and idea system with stronger separation between source, claim, interpretation, idea, and publication.

## Comparison With graphify

graphify turns a corpus into a knowledge graph with clustered communities, graph JSON, visualization, and an audit trail. It is useful for finding non-obvious connections across documents and tagging relationships as extracted, inferred, or ambiguous.

Thought Castle should not be replaced by graphify. The two systems should interoperate:

- Thought Castle owns the source lifecycle, verification gates, and user-facing knowledge files.
- graphify analyzes the vault and exposes relationships, communities, and surprising connections.
- graphify outputs should be treated as analytical artifacts, not canonical truth.

Recommended integration:

```text
Thought Castle vault
  -> graphify run over 01_sessions, 10_knowledge, 20_thoughts, 30_ideas
  -> graphify-out/graph.json and GRAPH_REPORT.md
  -> selected insights become 30_ideas or 10_knowledge candidates with source_refs
```

## Proposed Architecture

### Source Adapters

Each adapter should implement the same high-level contract:

```text
discover -> list candidate source sessions without reading full content
sync     -> copy or snapshot selected raw sessions into 00_raw-sessions
parse    -> convert provider format into normalized session turns
index    -> record sync status, provider id, hash, title, timestamps, and topic hints
```

Initial adapters:

- `codex-local-jsonl`
- `claude-code-jsonl`
- `opencode-sqlite`
- `pi-agent-jsonl`
- `claude-desktop-cowork`
- `manual-raw-session`
- `chatgpt-export-zip`
- `claude-export-zip`
- `perplexity-export-or-manual`
- `pi-consumer-export-sample`

### Raw Session Index

Add a durable session index, probably under `_system/session-index.jsonl` or `_system/session-index.sqlite`.

Required fields:

- `provider`
- `provider_session_id`
- `source_path`
- `raw_path`
- `content_hash`
- `title`
- `created_at`
- `updated_at`
- `last_synced_at`
- `sync_status`
- `privacy_review`
- `topic_hints`
- `normalized_session_path`

### Sync Commands

Proposed CLI commands:

```bash
thought-castle source list <lab> --provider codex
thought-castle sync <lab> --provider codex --since 7d
thought-castle sync <lab> --provider claude-code --project /path/to/project
thought-castle sync <lab> --provider opencode
thought-castle sync <lab> --provider pi-agent
thought-castle sync <lab> --provider chatgpt-export --file ~/Downloads/chatgpt-export.zip
thought-castle sync <lab> --provider claude-export --file ~/Downloads/claude-export.zip
thought-castle ingest manual <lab> --provider perplexity --title "Thread Title" --file ./thread.md
thought-castle ingest paste <lab> --provider chatgpt --title "Conversation Title"
thought-castle normalize pending <lab>
thought-castle extract knowledge <lab> --session 01_sessions/foo.md
thought-castle verify <lab> --knowledge 10_knowledge/foo.md
```

### Topic Routing

Topic classification should not decide truth. It should only route material into reviewable folders/tags.

Suggested metadata:

```yaml
topics:
  - machine-learning/llm
  - software-engineering/agents
learning_intent:
  - understand
  - debug
  - compare
  - design
```

### Knowledge Extraction

Only extract a `10_knowledge` candidate when the statement is claim-like and independently checkable.

Candidate fields:

- claim
- source_refs
- confidence
- verification.status
- evidence
- caveats
- contradicted_by

Rules:

- LLM answers are not evidence by themselves.
- User hypotheses are not facts.
- A claim can be important but remain `needs_verification`.
- Verified claims require official documentation, primary sources, reproducible code, or other explicit evidence.

### Thought Extraction

Use `20_thoughts` for the user's understanding, confusion, preferences, goals, and interpretations.

Rules:

- Default status is `draft`.
- `user_confirmed` defaults to false.
- Agent-inferred emotion or intent must remain marked as inferred.

### Idea Generation

Ideas should be generated from reviewed material, not raw chat fragments alone.

Inputs:

- verified or high-value knowledge candidates
- confirmed or reviewing thoughts
- graphify surprise connections
- repeated learning questions

Outputs:

- `30_ideas/*.md`
- method
- source_refs
- input materials
- smallest next experiment
- risk review

## Implementation Phases

### Phase 1: Local Session Inventory

Goal: discover provider session sources safely without importing full content.

Tasks:

- Add provider inventory docs.
- Add fixture files for Codex, Claude Code, OpenCode, Pi Agent, ChatGPT export, Claude export, and manual raw sessions.
- Add failing tests for `source list`.
- Implement read-only discovery for local JSONL and SQLite locations.

Acceptance:

- The CLI can list available local source candidates without printing conversation text.

### Phase 2: Raw Sync

Goal: copy/snapshot selected sessions into `00_raw-sessions`.

Tasks:

- Add session index.
- Add sync command.
- Preserve original raw files.
- Add content hash and duplicate detection.

Acceptance:

- Re-running sync does not duplicate unchanged sessions.
- Raw files and sidecars are created deterministically.

### Phase 3: Provider Parsers

Goal: normalize raw provider formats into canonical `01_sessions`.

Tasks:

- Parse Codex JSONL.
- Parse Claude Code JSONL.
- Parse OpenCode SQLite fixture rows.
- Parse Pi Agent JSONL.
- Parse ChatGPT export zip.
- Parse Claude export zip.
- Normalize manual Markdown/text/HTML paste files.

Acceptance:

- Each provider fixture yields a Markdown session with stable turn ids and source refs.

### Phase 4: Knowledge Compilation

Goal: generate reviewable candidates from normalized sessions.

Tasks:

- Add claim extraction workflow.
- Add thought extraction workflow.
- Add topic routing.
- Add validation that every derived note has source refs.

Acceptance:

- Extracted knowledge remains `candidate` or `needs_verification`.
- Extracted thoughts remain `draft` with `user_confirmed: false`.

### Phase 5: Verification and Graph Loop

Goal: make the system trustworthy and useful for synthesis.

Tasks:

- Add verification checklist.
- Add source existence validator.
- Add graphify export/run/report integration.
- Feed selected graph insights into ideas or knowledge candidates.

Acceptance:

- `verified`, `stable`, and `published` states are blocked unless gates are satisfied.
- graphify reports can be produced without becoming canonical truth automatically.

## Privacy and Safety Rules

- Discovery must avoid reading or printing full conversation content.
- Sync must be opt-in per provider.
- Local encrypted or private app databases must not be reverse engineered silently.
- Browser automation and extensions are experimental and require explicit user approval.
- Raw sessions must remain immutable.
- Sensitive sessions need a privacy review status before extraction.
- Generated knowledge must never be marked verified without evidence.

## Immediate Next Step

The next implementation task should be `source list` for local-first providers:

1. Codex local JSONL discovery.
2. Claude Code local JSONL discovery.
3. OpenCode SQLite schema discovery.
4. Pi Agent local JSONL discovery.

This creates the foundation for automatic periodic sync without touching brittle web-app scraping first.

## Sources Used

- OpenAI ChatGPT data export: https://help.openai.com/en/articles/7260999-how-do-i-export-my-chatgpt-history-and-data
- OpenAI ChatGPT macOS retention: https://help.openai.com/en/articles/9268871-how-is-data-retained-in-the-macos-app
- OpenAI Compliance Platform: https://help.openai.com/en/articles/9261474-compliance-api-for-chatgpt-enterprise-edu-and-chatgpt-for-teachers
- Claude data export: https://support.claude.com/en/articles/9450526-how-can-i-export-my-claude-data
- Claude Desktop local data: https://claude.com/docs/cowork/3p/data-storage
- Claude Code sessions: https://code.claude.com/docs/en/how-claude-code-works
- Pi Agent sessions: https://pi.dev/docs/latest/sessions
- Pi Agent session format: https://pi.dev/docs/latest/session-format
- Karpathy LLM Wiki: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
