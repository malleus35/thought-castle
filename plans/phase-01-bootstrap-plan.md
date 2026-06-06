# Phase 01 Bootstrap Plan

## Goal

Thought Castle의 수동/자동 경계를 먼저 고정하고, 이후 parser와 agent automation을 붙일 수 있는 문서 기반 구조를 만든다.

## Phase Scope

- Folder skeleton
- PRD
- Task documents
- Subtask checklists
- 10/20/30/40 templates
- Source reference schema
- Status transition rules

## Excluded

- 실제 parser 구현
- raw session sample import
- Obsidian plugin automation
- graphify/RAG pipeline

## Execution Order

1. Folder skeleton 생성
2. PRD 작성
3. Task 문서 분리
4. Subtask 체크리스트 분리
5. Template 생성
6. Source trace schema 작성
7. Status transition rule 작성
8. 실제 sample session으로 검증

## Completion Criteria

- 각 core folder의 책임이 문서화되어 있다.
- 자동 생성물과 사용자 승인물이 상태값으로 구분된다.
- 각 template이 `source_refs`를 포함한다.
- `20_thought`와 `40_post`가 hybrid workflow를 반영한다.
