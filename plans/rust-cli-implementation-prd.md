# Rust CLI Implementation PRD

## Problem Statement

Creative Idea Lab은 문서 구조와 템플릿은 있지만, 이를 반복적으로 생성/검증/agent workflow에 연결하는 실행 도구가 없다.

## Goals

- Rust 기반 CLI를 제공한다.
- PRD의 folder skeleton과 template을 CLI로 생성할 수 있다.
- `source_refs`, `user_confirmed`, post checklist 같은 핵심 invariant를 검증할 수 있다.
- agent가 사용할 수 있는 installable skill을 제공한다.
- 단계별 TDD 커밋으로 구현한다.

## Non-Goals

- 이번 단계에서 모든 provider raw session parser를 완성하지 않는다.
- 이번 단계에서 LLM 호출이나 자동 추출을 구현하지 않는다.
- 이번 단계에서 Obsidian plugin이나 graphify/RAG pipeline을 구현하지 않는다.

## CLI Requirements

### R1. Init

`creative-idea-lab init <path>`는 target path에 core folders, templates, system docs를 생성한다.

### R2. Validate

`creative-idea-lab validate <path>`는 다음을 검증한다.

- core folders 존재
- required templates 존재
- templates contain `source_refs`
- thought template contains `user_confirmed`
- post template contains `Review Checklist`

### R3. Skill Install

`creative-idea-lab skill install --target <path>`는 agent skill directory에 Creative Idea Lab skill을 설치한다.

### R4. Skill Print

`creative-idea-lab skill print`는 설치 가능한 `SKILL.md` 내용을 stdout으로 출력한다.

## Constraints

- Production code는 failing test 이후에만 작성한다.
- Rust crate는 가능하면 표준 라이브러리 중심으로 시작한다.
- CLI는 agent automation에서 쓰기 쉬운 deterministic output을 낸다.
- File mutation command는 overwrite를 최소화한다.

## Tasks

1. Baseline docs commit
   - Existing PRD/task/template artifacts commit
   - Acceptance: git repo has initial documentation baseline

2. RED tests for CLI behavior
   - Add integration tests for init, validate, skill print/install
   - Acceptance: tests fail because CLI is not implemented

3. GREEN CLI implementation
   - Add Rust package
   - Implement init/validate
   - Acceptance: init/validate tests pass

4. Skill support implementation
   - Add embedded skill content
   - Implement skill print/install
   - Acceptance: skill tests pass

5. Documentation and audit
   - Add README usage
   - Run cargo test and CLI smoke checks
   - Acceptance: clean final status except intentional committed files

## Acceptance Checks

- `cargo test` passes.
- `cargo run -- init <tmpdir>` creates core structure.
- `cargo run -- validate <tmpdir>` succeeds after init.
- `cargo run -- skill print` prints `# Creative Idea Lab`.
- `cargo run -- skill install --target <tmpdir>` writes `creative-idea-lab/SKILL.md`.

## Risks and Open Questions

- Final package naming may change before distribution.
- Provider-specific session normalization remains future work.
- Skill install locations differ across hosts; CLI starts with explicit `--target`.
