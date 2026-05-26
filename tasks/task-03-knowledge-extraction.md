# Task 03: Knowledge Extraction

## Goal

대화에서 객관적 지식 후보를 추출해 `10_knowledge`에 저장한다.

## Requirements

- 기본 상태는 `candidate`다.
- 검증 전에는 사실처럼 취급하지 않는다.
- claim, evidence, caveats를 분리한다.
- 원본 대화 turn을 `source_refs`로 연결한다.

## Subtasks

- [ ] knowledge candidate 추출 기준 정의
- [ ] verification status 기준 정의
- [ ] evidence source 종류 정의
- [ ] `verified` 승격 규칙 정의
- [ ] 반례/불확실성 표기 방식 정의

## Output

- `10_knowledge/*.md`
- verification metadata
