# Source Reference Schema

## Purpose

모든 파생 노트가 원본 대화의 어느 부분에서 추출되었는지 되돌아갈 수 있게 한다.

## YAML Shape

```yaml
source_refs:
  - session: "[[01_sessions/YYYY-MM-DD-session-slug.md#^t0001]]"
    raw_file: "00_raw-sessions/YYYY-MM-DD-source-id.ext"
    source_type: ai_conversation
    extraction_type: knowledge
    confidence: medium
```

## Rules

- `session`은 Obsidian block reference를 포함해야 한다.
- `raw_file`은 원본 파일 경로를 가리켜야 한다.
- `extraction_type`은 `knowledge`, `thought`, `idea` 중 하나여야 한다.
- 자동 생성물은 `confidence`를 기록해야 한다.
