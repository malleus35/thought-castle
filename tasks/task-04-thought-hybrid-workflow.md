# Task 04: Thought Hybrid Workflow

## Goal

대화에서 주관적 생각, 감정, 판단을 추출하되 사용자 승인 전에는 초안으로 유지한다.

## Requirements

- 기본 상태는 `draft`다.
- `user_confirmed: false`로 시작한다.
- 감정은 metadata로 얕게 분리하고 본문에 맥락을 남긴다.
- 사용자 승인 후에만 `stable`로 승격한다.

## Subtasks

- [ ] thought extraction criteria 정의
- [ ] emotion metadata 값 목록 정의
- [ ] user confirmation workflow 정의
- [ ] AI inferred vs self reported 구분 규칙 정의
- [ ] discarded thought 처리 방식 정의

## Output

- `20_thoughts/*.md`
- user confirmation fields
