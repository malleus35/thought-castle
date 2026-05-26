# Task 02: Session Normalization

## Goal

원본 세션을 `01_sessions`의 canonical Markdown으로 변환한다.

## Requirements

- 모든 대화 turn에 안정적인 block id를 부여한다.
- raw file path를 frontmatter에 기록한다.
- Pi 같은 tree session은 branch/parent 정보를 보존한다.
- 사람이 읽을 수 있는 summary와 context를 생성한다.

## Subtasks

- [ ] canonical session template 확정
- [ ] turn id naming rule 정의
- [ ] branch 표시 방식 정의
- [ ] source_refs target 형식 검증
- [ ] raw-to-session 변환 테스트 샘플 작성

## Output

- `01_sessions/*.md`
- source reference anchor
