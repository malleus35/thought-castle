# Task 01: Raw Session Ingestion

## Goal

`00_raw-sessions`에 들어오는 다양한 원본 대화 포맷을 보존하고 식별한다.

## Requirements

- 원본 파일은 수정하지 않는다.
- 파일별 provider, source type, captured date를 기록한다.
- 동일 파일 재수집을 감지할 수 있도록 hash를 기록한다.

## Subtasks

- [ ] raw filename convention 정의
- [ ] provider별 format inventory 작성
- [ ] metadata sidecar schema 정의
- [ ] 민감정보 검토 규칙 정의
- [ ] 원본 파일 hash 기록 방식 결정

## Output

- `00_raw-sessions/` 원본 저장 규칙
- raw metadata sidecar schema
