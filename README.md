# Thought Castle

Thought Castle is a local-first verified knowledge archive for conversations with LLMs.

It captures raw study sessions, normalizes them into stable Markdown references, and helps turn useful fragments into evidence-gated knowledge, personal thoughts, and idea candidates. The goal is not content publishing. The goal is a durable archive where every claim can point back to the session and raw source that produced it.

## Why Use This

Karpathy LLM Wiki is an excellent pattern for letting an LLM maintain a persistent wiki from immutable raw sources. graphify is excellent when you want a folder converted into a navigable knowledge graph with HTML, JSON, and audit reports.

Thought Castle should exist only if it adds a different layer:

| Tool | Best at | Thought Castle difference |
| --- | --- | --- |
| Karpathy LLM Wiki | A general pattern for raw sources that compound into an LLM-maintained wiki | Thought Castle makes the workflow concrete for LLM study sessions: automatic local agent sync, manual captures, normalized session block ids, and evidence-gated note states. |
| graphify | Turning mixed files into a graph with communities, inferred edges, and reports | Thought Castle owns the source lifecycle before graph analysis: raw provenance, session normalization, candidate extraction, verification state, and user-confirmed understanding. graphify can still be used as an analysis layer over a Thought Castle vault. |

The practical distinction is this: graphify helps you see relationships; Thought Castle helps you decide whether a claim is actually part of your verified knowledge archive.

## Install

With Homebrew:

```bash
brew install malleus35/tap/thought-castle
```

Homebrew installs the latest tagged release from the tap. If you need the newest archive-only workflow from `main`, install from source until the next release is cut.

From source:

```bash
git clone https://github.com/malleus35/thought-castle.git
cd thought-castle
cargo install --path .
```

For local development:

```bash
cargo test
cargo run -- validate .
```

## Quickstart

Create and validate a vault:

```bash
thought-castle init ~/thought-castle-lab
thought-castle validate ~/thought-castle-lab
cd ~/thought-castle-lab
```

The core archive structure is:

```text
00_raw-sessions  -> original local sessions, exports, and manual captures
01_sessions      -> normalized Markdown sessions with stable block ids
10_knowledge     -> objective knowledge candidates and verified facts
20_thoughts      -> your interpretations, judgments, and confirmed understanding
30_ideas         -> generated hypotheses, combinations, and experiment candidates
```

Planning and task files live in `plans/`, `tasks/`, and `subtasks/`.

## Capture Sessions

List automatic local session candidates without printing message text:

```bash
thought-castle source list . --provider codex --root ~/.codex/sessions
thought-castle source list . --provider claude-code --root ~/.claude/projects
thought-castle source list . --provider opencode --root ~/.local/share/opencode
thought-castle source list . --provider pi-agent --root ~/.pi/agent/sessions
```

Sync automatic local sessions into `00_raw-sessions/<provider>/`:

```bash
thought-castle sync . --provider codex --root ~/.codex/sessions
thought-castle sync . --provider claude-code --root ~/.claude/projects
thought-castle sync . --provider opencode --root ~/.local/share/opencode
thought-castle sync . --provider pi-agent --root ~/.pi/agent/sessions
```

Ingest a manual capture from web or desktop apps:

```bash
thought-castle ingest manual . \
  --provider chatgpt \
  --title "LLM Wiki Conversation" \
  --file ./thread.md
```

Manual capture is the supported fallback for ChatGPT, Claude, Perplexity, exports, copied transcripts, and saved Markdown files.

## Normalize And Extract

Normalize a raw session into a canonical Markdown file:

```bash
thought-castle session normalize . 00_raw-sessions/manual/thread.md \
  --title "LLM Wiki Conversation" \
  --source-type ai_conversation
```

Create traceable notes from a normalized session:

```bash
thought-castle note new knowledge . \
  --title "Persistent Wiki Pattern" \
  --session "[[01_sessions/llm-wiki-conversation.md#^t0001]]" \
  --raw-file "00_raw-sessions/manual/thread.md"

thought-castle note new thought . \
  --title "My Verification Bias" \
  --session "[[01_sessions/llm-wiki-conversation.md#^t0001]]" \
  --raw-file "00_raw-sessions/manual/thread.md"

thought-castle note new idea . \
  --title "Session Evidence Review Loop" \
  --session "[[01_sessions/llm-wiki-conversation.md#^t0001]]" \
  --raw-file "00_raw-sessions/manual/thread.md"
```

## Verification Model

Thought Castle separates three things that often get mixed together in chat logs:

- `10_knowledge`: objective claims. These start as `candidate` and should only become `verified` when evidence is recorded.
- `20_thoughts`: your understanding, interpretation, or judgment. These should not become stable unless you confirm them.
- `30_ideas`: generated possibilities. These are useful, but they are not facts.

Every derived note carries `source_refs` so you can trace it back to a normalized session block and raw file. This is the main reason to use Thought Castle instead of treating chat history as loose memory.

## Agent Skill

The installable agent skill source lives at `skills/thought-castle/SKILL.md`.

```bash
thought-castle skill print
thought-castle skill install --target ~/.agents/skills
```

## References

- [Karpathy LLM Wiki](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)
- [graphify](https://graphify.net/zh/)
