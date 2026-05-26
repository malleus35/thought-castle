# Creative Idea Lab

This folder is a workspace for designing a personal knowledge and creative output system.

The core pipeline is:

```text
00_raw-sessions
  -> 01_sessions
  -> 10_knowledge
  -> 20_thoughts
  -> 30_ideas
  -> 40_posts
```

`00_raw-sessions` stores original conversation/session artifacts.
`01_sessions` stores normalized Markdown sessions with stable references.
`10_knowledge` stores objective knowledge candidates and verified facts.
`20_thoughts` stores subjective interpretations, judgments, and emotional context.
`30_ideas` stores creative combinations and experiment candidates.
`40_posts` stores platform-specific drafts and published outputs.

Planning artifacts live in `plans/`, `tasks/`, and `subtasks/`.

## CLI

This repository includes a Rust CLI.

```bash
cargo run -- init /tmp/my-lab
cargo run -- validate /tmp/my-lab
cargo run -- ingest /tmp/my-lab ./session.jsonl --provider pi --source-type ai_conversation
cargo run -- note new thought /tmp/my-lab --title "AI Content Fatigue" --session "[[01_sessions/example.md#^t0038]]" --raw-file "00_raw-sessions/session.jsonl"
cargo run -- skill print
cargo run -- skill install --target ~/.agents/skills
```

## Agent Skill

The installable skill source lives at `skills/creative-idea-lab/SKILL.md`.
