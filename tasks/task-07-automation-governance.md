# Task 07: Automation Governance

## Goal

AI Agent가 자동으로 생성할 수 있는 범위와 사용자 승인 게이트를 명확히 한다.

## Requirements

- 자동 생성물은 기본적으로 `candidate`, `draft`, `raw` 상태다.
- `verified`, `stable`은 사용자 승인 또는 명시적 검증이 필요하다.
- source trace 없는 파생 노트는 유효하지 않다.

## Subtasks

- [ ] folder별 automation level 정의
- [ ] human gate 정의
- [ ] source trace validator 설계
- [ ] status transition rule 정의
- [ ] audit report 형식 정의

## Output

- automation responsibility matrix
- status transition rules
