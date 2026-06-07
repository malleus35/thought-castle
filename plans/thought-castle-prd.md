# Thought Castle PRD

## Purpose

AI와 사람의 대화에서 지식, 생각, 아이디어를 추출하고, 각 산출물이 원본 대화의 어느 지점에서 왔는지 추적 가능한 검증 지식 아카이브를 만든다.

## Product Definition

Thought Castle은 다음 네 계층의 산출물과 원본 보존 계층을 가진다.

```text
00_raw-sessions  원본 보존
01_sessions      참조 가능한 Markdown 정규화
10_knowledge     객관 지식 후보/검증 지식
20_thoughts      주관적 생각/감정/판단
30_ideas         조합형 아이디어/실험 후보
```

## Users

- Primary user: 개인 지식 저장소를 운영하는 사용자
- AI agent: raw session을 정규화하고 후보 산출물을 생성하는 자동화 실행자
- Future reader: 특정 생각, 지식, 아이디어가 어떤 대화에서 나왔는지 검토하는 사용자

## Non-Goals

- 모든 내용을 자동으로 사실로 확정하지 않는다.
- `20_thoughts`를 사용자 승인 없이 최종 생각으로 간주하지 않는다.
- 외부 게시나 배포 기능을 핵심 제품 범위에 포함하지 않는다.
- 원본 raw session을 덮어쓰거나 변형하지 않는다.

## Source Inputs

### 00 Raw Session Types

- Human conversation transcript: 녹음 대본, plain text, Markdown, DOCX/PDF 등 가능
- Pi session: 공식 문서 기준 JSONL, `id`/`parentId` tree 구조
- Codex / Claude Code / other agent sessions: 주로 JSON/JSONL일 가능성이 있으나 도구별 검증 필요
- Claude / ChatGPT / Gemini web sessions: 복사 붙여넣기, export zip, HTML, JSON 등 가능

## Core Requirements

### R1. Raw Preservation

`00_raw-sessions`는 원본 파일을 보존해야 한다. 자동화는 원본을 수정하지 않고 metadata sidecar만 추가할 수 있다.

### R2. Stable Session References

`01_sessions`는 모든 대화 turn에 안정적인 block id를 부여해야 한다.

Example:

```md
### t0038 user ^t0038
...
```

### R3. Source Traceability

`10_knowledge`, `20_thoughts`, `30_ideas`의 모든 파일은 `source_refs`를 가져야 한다.

### R4. Knowledge Verification Gate

`10_knowledge`는 자동 생성될 수 있지만 기본 상태는 `candidate`다. 검증 근거가 있어야 `verified`가 된다.

### R5. Thought Ownership Gate

`20_thoughts`는 자동 초안 생성이 가능하지만 기본 상태는 `draft`, `user_confirmed: false`다. 사용자 승인 후에만 `stable`이 된다.

### R6. Idea Generation

`30_ideas`는 자동 생성 중심이다. 단, 모든 idea는 `inputs`와 `source_refs`를 가져야 한다.

### R7. Archive-Only Scope

Thought Castle의 기본 제품 범위는 검증 가능한 지식 아카이빙이다. 외부 게시물 생성, 예약, 발행 추적은 제외한다.

## Folder Responsibilities

| Folder | Responsibility | Automation Level | Human Gate |
|---|---|---:|---|
| `00_raw-sessions` | 원본 보존 | low | 민감정보/저장 여부 |
| `01_sessions` | Markdown 정규화 | high | 제목/요약 검수 |
| `10_knowledge` | 객관 지식 추출 | high | verified 승격 |
| `20_thoughts` | 주관 생각 초안 | medium | stable 승인 |
| `30_ideas` | 창의 조합 생성 | high | 채택/폐기 |

## Status Model

### Knowledge

- `candidate`: 대화에서 추출됨
- `needs_verification`: 중요하지만 검증 필요
- `verified`: 근거 확인됨
- `disputed`: 반례 또는 불확실성 있음

### Thought

- `draft`: AI가 추출한 초안
- `reviewing`: 사용자가 검토 중
- `stable`: 사용자가 내 생각으로 승인
- `discarded`: 더 이상 유지하지 않음

### Idea

- `raw`: 조합 후보
- `reviewing`: 논의/검토 중
- `experimenting`: 작은 실험 진행 중
- `validated`: 유지할 가치 확인
- `discarded`: 폐기

## Acceptance Criteria

- `plans/thought-castle-prd.md`가 존재한다.
- `tasks/`에 주요 작업 단위가 분리되어 있다.
- `subtasks/`에 실행 가능한 체크리스트가 있다.
- `_templates/10_knowledge.md`, `_templates/20_thought.md`, `_templates/30_idea.md`가 존재한다.
- 각 템플릿은 `source_refs`를 포함한다.
- `20_thought`는 사용자 승인 필드를 포함한다.

## Open Questions

- Codex session과 Claude Code session의 실제 export/storage 포맷을 각 도구별로 확인해야 한다.
- Gemini export가 대화 전체를 어떤 구조로 제공하는지 공식 근거를 더 확인해야 한다.
- Plownote/Plaud 계열 transcript export 형식은 실제 샘플 파일로 검증해야 한다.
