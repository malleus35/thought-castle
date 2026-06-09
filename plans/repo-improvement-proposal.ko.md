# Thought Castle 저장소 점검 및 개선안

작성일: 2026-06-09
기준 커밋: `aae8033` (build: bump version to 0.1.2)

## 1. 현재 상태 요약

### 구현 진행 단계

커밋 이력 기준으로 작업은 아래 순서로 진행됐고, 각 단계가 TDD(테스트 선행 커밋 → 구현 커밋) 패턴을 일관되게 따르고 있다.

1. 문서 부트스트랩: PRD, tasks, subtasks, 템플릿, `_system` 규칙
2. Rust CLI 기본기: `init` / `validate` / `skill print` / `skill install`
3. raw ingest + note 스캐폴드: `ingest`, `note new`, `session normalize`
4. 세션 소스 자동화: `source list`, `sync` (codex / claude-code / opencode / pi-agent)
5. archive-only 전환: `40_posts` 게시 파이프라인 제거, README 재작성
6. archive intake agent workflow: SKILL.md 에 에이전트 운영 절차 명문화
7. 멀티 에이전트 기본 skill install (Pi / Claude Code / Codex / `~/.agents`)

### 품질 현황

- `cargo test`: 18개 통합 테스트 전부 통과
- `cargo fmt --check`, `cargo clippy --all-targets`: 경고 없음
- 의존성 0개 (std만 사용), edition 2024, 버전 0.1.2
- 테스트가 CLI 계약뿐 아니라 README/SKILL.md 문서 내용까지 고정하고 있어 문서-코드 동기화가 강제됨 (좋은 패턴)

### 알려진 배포 격차

`plans/readme-usage-implementation-audit.ko.md`(2026-06-07)가 이미 지적했듯, Homebrew tap stable은 `0.1.0`으로 archive-only 전환 이전 동작이다. 소스 빌드(`main`)와 brew 설치본의 동작이 다르다.

## 2. 검증된 버그 (이번 점검에서 실제 재현)

### 2.1 [P0] 한글 제목이 전부 `note.md` 슬러그로 떨어짐

`slugify`(`src/main.rs:573`)가 ASCII 영숫자만 허용해서 한글 제목은 슬러그가 비고, fallback `"note"`가 적용된다. 두 번째 한글 노트부터 생성이 막힌다.

```text
$ thought-castle note new knowledge . --title "중심극한정리" ...
created: 10_knowledge/note.md
$ thought-castle note new knowledge . --title "베이즈 정리" ...
error: refusing to overwrite existing file: 10_knowledge/note.md
```

`session normalize`도 같은 `slugify`를 쓰므로 한글 제목 세션은 모두 `01_sessions/note.md`로 충돌한다. 주 사용자가 한국어 사용자임을 감안하면 가장 시급한 결함이다.

개선안:
- `char::is_alphanumeric` 기반으로 유니코드 문자를 슬러그에 보존 (한글 파일명은 Obsidian/일반 파일시스템에서 문제없음)
- 슬러그 충돌 시 `-2`, `-3` 숫자 접미사 자동 부여 옵션 추가
- `_system/source-reference-schema.md`의 예시(`YYYY-MM-DD-session-slug.md`)에 맞춰 날짜 접두사를 기본 적용하면 충돌 확률도 줄어듦 (아래 3.3 참고)

### 2.2 [P0] 메타데이터 사이드카가 깨진 JSON이 될 수 있음

`json_escape`(`src/main.rs:595`)가 `\\`와 `"`만 이스케이프하고 개행/탭/제어문자는 그대로 통과시킨다. 제목에 개행이 들어가면 invalid JSON이 생성된다 (재현 확인).

```json
{
  "title": "line1
line2",
  ...
}
```

`yaml_escape`(`src/main.rs:599`)도 동일한 한계가 있어 `--session`/`--raw-file` 값에 개행이 들어가면 frontmatter가 깨진다.

개선안:
- 최소 수정: `\n` → `\\n`, `\r`, `\t`, 기타 제어문자(U+0000–U+001F)를 `\uXXXX`로 이스케이프
- 또는 zero-dependency 원칙을 완화해 `serde_json` 도입 (사이드카가 늘어날수록 수동 직렬화 유지비가 커짐)

## 3. PRD 대비 기능 격차

### 3.1 [P1] `session normalize`가 turn 분리를 하지 않음 (PRD R2 미충족)

PRD R2는 "모든 대화 turn에 안정적인 block id"(`### t0038 user ^t0038`)를 요구하지만, 현재 구현(`src/main.rs:506`)은 raw 전문을 `### t0001 source ^t0001` 단일 블록에 통째로 넣는 스캐폴드다. `source_refs`가 가리킬 수 있는 앵커가 사실상 t0001 하나뿐이라 "어느 지점에서 왔는지 추적"이라는 핵심 가치가 아직 동작하지 않는다.

개선안 (우선순위 순):
1. Codex / Claude Code / Pi Agent JSONL 파서: 라인 단위 JSON에서 role과 텍스트를 추출해 `t0001..tNNNN` per-turn 블록 생성. provider는 사이드카(`*.meta.json`)에서 읽을 수 있음
2. manual Markdown 캡처: 휴리스틱 turn 분리(예: `## User` / `## Assistant` 헤더, 또는 `--turn-separator` 플래그)
3. Pi tree 세션의 `id`/`parentId` branch 보존은 그 다음 단계
4. opencode SQLite 파싱은 의존성(sqlite)이 필요하므로 당분간 스냅샷 보존만 유지하고 명시적으로 비범위 처리

### 3.2 [P1] `validate`가 핵심 invariant를 검증하지 않음 (PRD R3/R4, task-07)

현재 `validate_lab`(`src/main.rs:145`)는 디렉터리 존재와 템플릿 파일 내용만 본다. 제품의 핵심 주장인 "evidence-gated 상태"를 강제하는 검증이 없다:

- `10_knowledge`/`20_thoughts`/`30_ideas` 실제 노트의 `source_refs` 존재 여부 미검사
- `source_refs`가 가리키는 세션 파일/블록 id(`#^tNNNN`) 실존 여부 미검사
- `status: verified`인데 `evidence: []`인 knowledge, `status: stable`인데 `user_confirmed: false`인 thought 같은 게이트 위반 미검사 (`_system/status-transition-rules.md`에 규칙은 있으나 실행 도구가 없음)

개선안: `validate`에 노트 스캔 단계를 추가하거나 별도 `thought-castle audit <lab>` 명령으로 분리. subtask `st-04-source-trace-validation.md`, `st-10-automation-gates.md`가 정확히 이 작업이며, SKILL.md의 "report what still needs human verification" 단계도 이 명령이 있어야 에이전트가 신뢰성 있게 수행 가능하다.

### 3.3 [P2] 파일명 규칙이 `_system` 스키마 문서와 불일치

`_system/source-reference-schema.md`는 `01_sessions/YYYY-MM-DD-session-slug.md`, `00_raw-sessions/YYYY-MM-DD-source-id.ext` 형식을 예시로 들지만, 구현은 날짜 접두사를 어디에도 붙이지 않는다. 둘 중 하나로 정렬 필요 (날짜 접두사 채택 권장 — 정렬성과 충돌 회피에 유리).

### 3.4 [P2] `note new`의 `source_refs`에 `source_type` 필드 누락

스키마 문서의 YAML 예시는 `source_type: ai_conversation`을 포함하지만 `create_note`(`src/main.rs:461`)가 생성하는 `source_refs`에는 없다. `confidence`도 `medium` 하드코딩이므로 `--source-type`, `--confidence` 선택 플래그 추가를 권장.

## 4. sync / ingest 데이터 안전성

### 4.1 [P1] `sync`가 파일명만으로 중복 판정 → 갱신 누락과 충돌 위험

`sync_sources`(`src/main.rs:334`)는 대상 파일명이 이미 존재하면 무조건 skip한다.

- Codex/Claude Code JSONL은 세션이 이어지면 같은 파일에 append되므로, 한 번 sync된 세션은 이후 갱신분이 영원히 아카이브에 반영되지 않음
- 서로 다른 하위 디렉터리에 같은 파일명이 있으면(루트를 평탄화해서 복사하므로) 두 번째 파일이 조용히 "skipped"로 집계되어 유실됨

개선안:
- 사이드카에 이미 기록 중인 `content_hash`/`byte_len`을 비교해 변경 감지 → `updated: N` 집계 추가 (원본 불변 원칙을 지키려면 덮어쓰기 대신 `<name>.v2.jsonl` 버전 보존 방식도 고려)
- 평탄화 대신 root 기준 상대 경로 보존 또는 파일명에 짧은 해시 접미사 부여
- `--dry-run` 플래그 추가 (skill 워크플로가 "list 후 sync"를 요구하는 것과 결이 맞음)

### 4.2 [P2] `content_hash`가 아카이브 용도로 부적합

`content_hash`(`src/main.rs:567`)는 `std::collections::hash_map::DefaultHasher`(SipHash 계열)를 쓰는데, 이 알고리즘은 Rust 릴리스 간 안정성이 보장되지 않는다. "장기 보존 아카이브의 무결성/중복 식별자"라는 용도에는 컴파일러 버전이 바뀌면 같은 파일의 해시가 달라질 수 있다는 점이 치명적이다. 의존성 없이 구현 가능한 SHA-256(직접 구현 또는 소형 vendored 구현)이나 `sha2` crate 도입을 권장. 4.1의 변경 감지를 해시 기반으로 만들려면 선행 과제다.

### 4.3 [P2] `ingest` 계열의 저장 경로 비일관

- `sync` → `00_raw-sessions/<provider>/`
- `ingest manual` → `00_raw-sessions/manual/`
- `ingest`(일반) → `00_raw-sessions/` 루트 (`src/main.rs:224`)

일반 ingest도 `00_raw-sessions/<provider>/`로 통일하는 것이 충돌 방지와 탐색성 면에서 일관적이다.

### 4.4 [P3] 기타 견고성

- `collect_sources`(`src/main.rs:434`)가 심볼릭 링크 디렉터리를 따라가며 방문 추적이 없어 순환 링크에서 깊은 재귀 후 에러로 종료함 — `symlink_metadata`로 링크 스킵 권장
- opencode `opencode.db`는 사용 중인 SQLite를 단순 파일 복사하므로 WAL 모드면 `-wal`/`-shm` 미반영·스냅샷 비일관 가능 — 한계를 문서화하거나 wal/shm 동반 복사

## 5. CLI 사용성

- [P1] `--version` / `--help` / `-h` 플래그 부재. Homebrew formula 테스트와 에이전트의 버전 확인 모두 `--version`을 기대하는 것이 관례
- [P2] `Flags::parse`(`src/main.rs:615`)가 모르는 플래그를 조용히 무시함 → 오타가 "missing required flag"라는 엉뚱한 에러나 무시로 이어짐. 알 수 없는 플래그는 에러 처리 권장. `--flag=value` 형식과 중복 플래그 검출도 미지원
- [P2] `CliError::Io`가 경로 정보 없이 "No such file or directory"만 출력 → 어떤 경로가 문제인지 알 수 없음. 경로 컨텍스트 포함 권장
- [P2] `note new` / `session normalize` / `ingest`가 lab 유효성(최소한 core 디렉터리 존재)을 확인하지 않아 임의 디렉터리에 구조를 생성함. `source list`/`sync`처럼 lab 검사 일관화
- [P3] `skill install`(기본)이 해당 에이전트 설치 여부와 무관하게 4개 디렉터리를 모두 생성함. 부모 디렉터리(`~/.claude` 등)가 존재하는 곳만 설치하고 나머지는 skipped로 보고하는 방식 고려
- [P3] 에이전트 자동화를 위해 `source list` / `sync` / `validate`에 `--json` 출력 모드 추가 고려

## 6. 코드 구조와 테스트

- [P2] `src/main.rs` 단일 파일 842줄. 현재는 감당 가능하지만 per-turn 파서(3.1)와 audit(3.2)가 들어오면 한계. `lib.rs` + 모듈 분리(`cli.rs`, `vault.rs`, `sync.rs`, `normalize.rs`, `templates.rs`)를 권장 — 라이브러리 크레이트로 분리하면 `slugify`/`Flags`/escape 함수의 **단위 테스트**가 가능해짐 (현재는 바이너리 경유 통합 테스트 18개뿐이라 2.1/2.2 같은 함수 단위 버그가 새기 쉬움)
- [P3] 통합 테스트의 `init`+`Command` 보일러플레이트를 헬퍼로 추출하면 테스트 추가 비용 감소

## 7. CI / 릴리스 / 문서 위생

### 7.1 [P0] CI 부재

`.github/workflows`가 없다. 최소 구성 제안:

```yaml
# ci.yml: push/PR 시
- cargo fmt --check
- cargo clippy --all-targets -- -D warnings
- cargo test
# (matrix: ubuntu + macos — 주 사용 환경이 macOS이므로)
```

테스트가 README/SKILL 문서까지 고정하는 구조라서 CI만 붙이면 문서 회귀도 자동 차단된다.

### 7.2 [P1] 릴리스 자동화와 Homebrew tap 갱신

소스는 0.1.2인데 tap stable은 0.1.0 (audit 문서 기준). 수동 릴리스가 병목이므로:

1. tag push 시 release workflow로 macOS/Linux 바이너리 빌드 및 GitHub Release 생성
2. tap formula 자동 bump (예: `brew bump-formula-pr` 또는 tap 저장소 dispatch)
3. `CHANGELOG.md` 도입 (커밋 메시지가 conventional 스타일이라 자동 생성 가능)
4. 단기 조치: 현재 main 기준으로 0.1.2 태그를 만들어 tap을 한 번 수동 갱신

### 7.3 [P2] 계획 문서 정리

- `tasks/*.md`와 `subtasks/`의 체크박스가 전부 미체크 상태지만 상당수는 이미 구현됨 (예: metadata sidecar schema, hash 기록 방식, canonical session template). 완료 항목 체크 또는 "구현 위치" 링크 추가로 문서를 현행화
- `plans/`와 `.codex/plans/`에 유사 주제 문서가 이원화되어 있음 (session-sync PRD/plan 등). 역할 구분(제품 계획 vs 에이전트 작업 로그)을 README나 `plans/README.md`에 한 줄로 명시 권장
- task-06, st-09 번호가 비어 있음 (posts 제거 흔적). 의도된 결번임을 주석으로 남기거나 재번호화

## 8. 우선순위 로드맵 제안

| 순위 | 항목 | 근거 |
| --- | --- | --- |
| P0 | 한글 슬러그 수정 (2.1) | 주 사용자 워크플로 차단 버그 |
| P0 | JSON/YAML 이스케이프 수정 (2.2) | 데이터 무결성, 수정 비용 낮음 |
| P0 | CI 추가 (7.1) | 이후 모든 변경의 안전망 |
| P1 | 릴리스 자동화 + tap 갱신 (7.2) | 설치 사용자와 main의 동작 불일치 해소 |
| P1 | sync 변경 감지 + 충돌 안전화 (4.1) | 아카이브 누락/유실 방지 |
| P1 | per-turn normalization 파서 (3.1) | 제품 핵심 가치(R2) 실현 |
| P1 | validate/audit 강화 (3.2) | 제품 핵심 가치(R3/R4) 실현 |
| P1 | `--version`/`--help` (5) | 배포 관례 |
| P2 | 해시 교체 (4.2), 경로 규칙 통일 (3.3, 4.3), 모듈 분리 (6) | 위 작업들의 기반 정비 |
| P3 | `--json` 출력, skill install 스마트 타겟, symlink/sqlite 견고성 | 점진 개선 |

P0 세 건은 서로 독립적이고 각각 작은 PR로 처리 가능하다. P1의 normalization 파서와 validate 강화는 모듈 분리(P2)를 먼저 하면 단위 테스트와 함께 진행하기 수월하므로, 착수 시점에 순서 재조정을 권장한다.
