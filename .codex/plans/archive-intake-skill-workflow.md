# Archive Intake Skill Workflow

## Goal

Make the installable `thought-castle` skill capable of running the document-management workflow after the user has installed the CLI and created a vault.

The user owns:

- installing `thought-castle`
- creating a vault with `thought-castle init`

The AI skill owns:

- syncing automatic local agent sessions
- accepting pasted web or desktop conversation text
- saving manual captures into raw sessions
- normalizing raw sessions
- creating traceable `knowledge`, `thought`, and `idea` drafts
- reporting what remains manual

## Scope

- Extend `skills/thought-castle/SKILL.md` with an "Archive Intake Run" workflow.
- Document a paste-first manual capture path where the user copies conversation text into chat and invokes the skill.
- Update README to tell users to invoke the skill for session management after vault creation.
- Add tests that pin the workflow language and safety rules.

## Acceptance Checks

- Skill says the user handles install and vault creation.
- Skill describes automatic sync for `codex`, `claude-code`, `opencode`, and `pi-agent`.
- Skill describes manual paste capture and storing pasted text before `ingest manual`.
- Skill instructs the agent to normalize and create note drafts with `source_refs`.
- Skill says not to mark knowledge `verified` or thoughts `stable` automatically.
- `cargo test` passes.
