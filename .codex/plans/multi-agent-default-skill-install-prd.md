# Multi-Agent Default Skill Install PRD

## Purpose

Make Thought Castle easy to use after Homebrew installation by allowing:

```bash
thought-castle skill install
```

with no `--target` flag. The command should install the Thought Castle skill into default skill directories for Pi Agent, Claude Code, Codex, and the shared Agent Skills location.

## Source Evidence

Official and local evidence:

- Pi docs list global skill locations `~/.pi/agent/skills/` and `~/.agents/skills/`.
- Pi docs list global extension locations under `~/.pi/agent/extensions/`.
- Claude Code docs list the personal skills folder as `~/.claude/skills/<skill-name>/SKILL.md`.
- OpenAI's Codex skill-installer documentation installs into `$CODEX_HOME/skills`, defaulting to `~/.codex/skills`.
- Current local Codex skill discovery includes `~/.codex/skills` and `~/.agents/skills`.

Sources:

- https://pi.dev/docs/latest/skills
- https://pi.dev/docs/latest/extensions
- https://code.claude.com/docs/en/skills
- https://github.com/openai/skills/blob/main/skills/.system/skill-installer/SKILL.md

## Goals

- `thought-castle skill install` installs to all default skill directories:
  - `~/.pi/agent/skills/`
  - `~/.claude/skills/`
  - `${CODEX_HOME:-~/.codex}/skills/`
  - `~/.agents/skills/`
- `thought-castle skill install --target <path>` keeps working for a single custom target.
- `thought-castle skill install <path>` keeps working for existing positional target usage.
- README and skill docs explain the no-flag multi-agent install command.
- Homebrew formula caveats tell users to run `thought-castle skill install` after `brew install`.

## Non-Goals

- Do not install Pi TypeScript extensions. Thought Castle currently ships a skill, not an extension.
- Do not write into `~/.pi/agent/extensions/`.
- Do not make Homebrew automatically mutate the user's home directory during install.

## User Flow

```bash
brew install malleus35/tap/thought-castle
thought-castle skill install
thought-castle init ~/thought-castle-lab
```

After this, the skill is available to Pi Agent, Claude Code, Codex, and shared Agent Skills harnesses that scan the listed default directories.

## TDD Plan

1. Add a failing CLI contract test:
   - set `HOME` and `CODEX_HOME` to temp directories
   - run `thought-castle skill install`
   - expect `SKILL.md` in all four default directories
2. Add README/skill documentation contracts for the no-flag multi-agent install command.
3. Implement default target resolution from `HOME` and `CODEX_HOME`.
4. Update Homebrew formula caveats.
5. Run `cargo test`.
6. Ask a subagent to independently review the implementation against this PRD.

## Acceptance Checks

- `cargo test` passes.
- `thought-castle skill install` works with `HOME` and `CODEX_HOME` set.
- Explicit `--target` install still writes only the requested target.
- README explains all default install targets.
- Installed skill remains the same `thought-castle` skill package.
- Homebrew formula includes caveats with `thought-castle skill install`.
- Subagent verification finds no blocking mismatch with this PRD.
