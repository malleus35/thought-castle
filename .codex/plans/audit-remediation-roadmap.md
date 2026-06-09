# Thought Castle Audit Remediation Roadmap

작성일: 2026-06-10
검토 대상: 2026-06-09 외부 에이전트 점검안
기준 커밋: `aae8033`
사용 워크플로: `agora` -> `plan-stress-test` 간소화 (`doubt-list` + `decision-memo`)

## 확인 결과

- 현재 `cargo test`는 18개 통합 테스트가 통과한다.
- 현재 `cargo fmt --check`는 통과한다.
- 현재 `cargo clippy --all-targets -- -D warnings`는 통과한다.
- 현재 `Cargo.toml`은 `0.1.2`다.
- 현재 로컬 Homebrew tap formula는 `v0.1.2` tarball을 가리킨다.
- `HOMEBREW_NO_INSTALL_FROM_API=1 brew list --versions thought-castle` 기준 설치본도 `0.1.2`다.
- `.github/workflows`는 없다.
- `src/main.rs`는 여전히 단일 파일이며 CLI, vault logic, sync, normalize, templates가 함께 있다.

## 점검안 보정

### 그대로 받아들일 항목

- 한글 제목이 `note.md`로 떨어지는 문제는 실제 코드 구조상 맞다. `slugify`가 ASCII 영숫자만 보존한다.
- JSON/YAML escape가 제어문자를 처리하지 못하는 문제는 맞다.
- `session normalize`가 raw 전체를 `t0001` 하나로 넣는다는 지적은 맞다.
- `validate`가 실제 note invariant를 보지 않는다는 지적은 맞다.
- `sync`가 파일명 존재만으로 skip하는 지적은 맞다.
- `content_hash`가 `DefaultHasher` 기반이라 장기 archive hash로 부적합하다는 지적은 맞다.
- `--version`, `--help`, `-h`, unknown flag 검증이 부족하다는 지적은 맞다.
- CI 부재는 현재도 맞다.

### 보정할 항목

- Homebrew stable이 `0.1.0`이라는 주장은 현재 기준 stale이다. 로컬 tap과 설치본은 `0.1.2`다. 이 항목은 "tap 갱신"이 아니라 "release/tap 검증 자동화"로 낮춘다.
- 날짜 접두사를 모든 파일명 기본값으로 즉시 바꾸는 것은 P0에 넣지 않는다. 현재 테스트와 사용 문서의 파일명 계약을 바꾸는 동작 변경이므로 별도 compatibility decision이 필요하다.
- `skill install`이 네 개 기본 경로를 모두 만드는 동작은 최근 의도된 기능이다. "존재하는 agent만 설치"는 현재 제품 방향과 충돌할 수 있으므로 P3 검토 항목으로 둔다.
- opencode SQLite row parsing은 provider dependency와 schema 확인이 필요하므로 당장은 raw snapshot 한계 문서화까지만 한다.

## Doubt List

### Happy Path Doubts

- Doubt: P0 세 건을 한 PR에 몰면 RED/GREEN 경계가 흐려진다.
- Verification: CI, slug, escaping을 각각 별도 PR로 쪼개고 각 PR마다 실패 테스트 출력과 GREEN 출력 기록.

### Edge Case Doubts

- Doubt: Unicode slug fix가 한글은 살리지만 공백/이모지/일본어/대소문자/중복 제목에서 새 충돌을 만들 수 있다.
- Verification: 한글, mixed ASCII+한글, punctuation-only title, duplicate title 테스트를 추가.

- Doubt: JSON/YAML escape를 수동 구현하면 다른 제어문자를 다시 놓칠 수 있다.
- Verification: `\n`, `\r`, `\t`, `\u0000`, quote, backslash를 포함한 metadata/frontmatter 계약 테스트 추가.

### Boundary Doubts

- Doubt: `sync` 변경 감지를 root-relative path로 바꾸면 기존 flattened archive와 경로가 달라진다.
- Verification: 기존 파일명 방식과 새 root-relative 방식의 migration/compat behavior를 PRD에 먼저 고정.

- Doubt: `validate`를 강하게 만들면 기존 draft vault가 갑자기 invalid가 될 수 있다.
- Verification: 구조 검사용 `validate`와 content invariant 검사용 `audit`을 분리하는 방향으로 설계.

### Ambiguity Doubts

- Doubt: "source_refs가 있어야 한다"는 말이 빈 배열 금지인지, session/raw_file/block id 실존 검증까지 포함하는지 애매하다.
- Verification: `audit` PRD에서 최소 invariant와 strict invariant를 분리.

- Doubt: "per-turn normalization"이 provider별 완전 파서인지, 우선 role/text만 뽑는 best-effort parser인지 애매하다.
- Verification: Codex/Claude/Pi JSONL fixture별 acceptance fixture를 먼저 만든다.

### Evil Demon Scenarios

- Doubt: 외부 점검안의 release 항목처럼 stale evidence가 roadmap 우선순위를 왜곡할 수 있다.
- Verification: release 관련 작업은 항상 `Cargo.toml`, git tag, local tap formula, installed binary를 동시에 확인한다.

- Doubt: archive hash가 바뀌면 이미 만들어진 sidecar와 새 sidecar가 섞여 중복 판단을 망칠 수 있다.
- Verification: hash algorithm version 필드를 sidecar에 추가하거나 migration note를 둔다.

## 실행 계획

### PR 0: CI Safety Net

목표: 이후 변경을 자동으로 막는 최소 CI를 추가한다.

RED:
- `tests/cli_contract.rs`에 `.github/workflows/ci.yml` 존재와 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` 포함을 확인하는 contract test 추가.

GREEN:
- `.github/workflows/ci.yml` 추가.
- ubuntu + macos matrix에서 fmt, clippy, test 실행.

검증:
- `rtk cargo test`
- `rtk cargo fmt --check`
- `rtk cargo clippy --all-targets -- -D warnings`

### PR 1: Unicode Slug And Collision Safety

목표: 한글 제목으로 note/session을 만들 수 있게 하고, 같은 slug 충돌 시 안전한 suffix를 부여한다.

RED:
- `note new knowledge --title "중심극한정리"`가 `10_knowledge/중심극한정리.md`를 만든다는 테스트.
- 다른 한글 제목이 `note.md`와 충돌하지 않는다는 테스트.
- 같은 제목 두 번 생성 시 두 번째가 `중심극한정리-2.md`로 저장된다는 테스트.
- `session normalize --title "베이즈 정리"`가 `01_sessions/베이즈-정리.md`를 만든다는 테스트.

GREEN:
- `slugify`를 Unicode `char::is_alphanumeric` 기반으로 변경.
- collision-safe destination helper 추가.
- 기존 ASCII slug 계약은 유지.

보류:
- 날짜 접두사 기본 적용은 이 PR에 넣지 않는다.

### PR 2: Metadata And Frontmatter Escaping

목표: sidecar JSON과 generated YAML frontmatter가 제어문자 때문에 깨지지 않게 한다.

RED:
- manual ingest title에 newline/tab/quote/backslash가 있어도 metadata가 escaped JSON 문자열을 만든다는 테스트.
- `note new`의 `--session`, `--raw-file` 값에 newline/tab/quote/backslash가 있어도 YAML double-quoted string으로 안전하게 escape된다는 테스트.

GREEN:
- `json_escape`가 `\n`, `\r`, `\t`, `\b`, `\f`, U+0000..U+001F를 처리하게 변경.
- `yaml_escape`도 double-quoted YAML escape set을 처리하게 변경.

검토:
- `serde_json` 도입은 PR 5/6에서 dependency 정책과 함께 결정한다. 이 PR은 최소 수정으로 끝낸다.

### PR 3: CLI Contract Hardening

목표: 에이전트/사용자가 CLI를 안전하게 탐색할 수 있게 한다.

RED:
- `thought-castle --help`, `thought-castle -h`가 help를 출력한다는 테스트.
- `thought-castle --version`이 `thought-castle 0.1.2` 형식을 출력한다는 테스트.
- unknown flag가 조용히 무시되지 않고 에러를 낸다는 테스트.
- duplicate flag와 `--flag=value` 처리 정책 테스트.

GREEN:
- top-level help/version flag 처리.
- `Flags::parse`에 allowed flag set 또는 command별 validation 추가.
- I/O 에러에 path context를 붙이는 helper 추가.

### PR 4: Refactor Into Library Modules

목표: per-turn parser와 audit을 넣기 전에 테스트 가능한 구조로 나눈다.

조건:
- PR 0-3 GREEN 이후에만 진행.

작업:
- `src/lib.rs` 생성.
- `cli`, `vault`, `sync`, `normalize`, `templates` 모듈로 분리.
- `slugify`, escape, hash, parser helpers는 단위 테스트 가능한 위치로 이동.

검증:
- behavior change 없이 기존 테스트 전부 GREEN.

### PR 5: Stable Hash And Sync Safety

목표: append되는 세션과 filename collision 때문에 raw archive가 누락되지 않게 한다.

RED:
- 같은 filename이 서로 다른 하위 디렉터리에 있을 때 둘 다 sync된다는 테스트.
- 같은 source path가 append되어 byte_len/hash가 바뀌면 `updated: 1` 또는 versioned copy가 생긴다는 테스트.
- `sync --dry-run`이 파일을 쓰지 않고 planned counts를 출력한다는 테스트.

GREEN:
- root-relative destination path 보존 또는 short stable hash suffix 중 하나를 PRD에서 확정 후 구현.
- `content_hash`를 stable SHA-256으로 변경.
- sidecar에 hash algorithm version을 기록.

Dependency decision:
- `sha2` crate를 쓰는 방안을 우선 검토한다. dependency 도입이 어렵다면 known-vector 테스트가 있는 internal SHA-256 implementation만 허용한다.

### PR 6: Provider Turn Normalization

목표: raw session을 실제 turn-level block ids로 normalize한다.

RED:
- Codex JSONL fixture가 `### t0001 user ^t0001`, `### t0002 assistant ^t0002`를 만든다는 테스트.
- Claude Code JSONL fixture가 role/text를 추출한다는 테스트.
- Pi Agent JSONL fixture가 role/content를 추출한다는 테스트.
- unknown/manual text는 기존 `t0001 source` fallback을 유지한다는 테스트.

GREEN:
- provider metadata 또는 `--provider` 기반 parser dispatch.
- structured JSON parser 도입. 이 단계에서는 `serde_json` 도입을 허용하는 쪽이 유지보수상 합리적이다.

Non-goal:
- opencode SQLite row parsing은 제외. WAL/SHM snapshot 한계만 문서화.

### PR 7: Audit Command For Evidence Gates

목표: 제품 핵심 주장인 source trace와 evidence gate를 실행 가능한 검사로 만든다.

RED:
- `audit <lab>`이 `source_refs: []`인 derived note를 실패 처리한다는 테스트.
- `source_refs.session`의 `#^tNNNN` block id가 없는 경우 실패한다는 테스트.
- knowledge가 `status: verified`인데 `verification.evidence: []`면 실패한다는 테스트.
- thought가 `status: stable`인데 `user_confirmed: false`면 실패한다는 테스트.

GREEN:
- `validate`는 scaffold 검사로 유지.
- `audit`은 note/session/source_refs/gate invariant 검사로 분리.
- output은 사람용 text부터 구현하고, `--json`은 후속 PR로 둔다.

### PR 8: Docs And Release Hygiene

목표: 계획 문서와 release evidence를 현재 상태로 맞춘다.

작업:
- `plans/readme-usage-implementation-audit.ko.md`의 stale Homebrew `0.1.0` 내용을 `0.1.2` 기준으로 갱신하거나 "historical audit"로 명시.
- `tasks/`, `subtasks/` 체크박스에 구현 위치 링크 추가.
- `plans/`와 `.codex/plans/` 역할 구분을 README 또는 `plans/README.md`에 명시.
- release checklist에 `Cargo.toml`, git tag, local tap formula, installed binary 확인을 포함.

## 추천 순서

1. PR 0 CI
2. PR 1 Unicode slug
3. PR 2 escaping
4. PR 3 CLI hardening
5. PR 4 module refactor
6. PR 5 sync safety
7. PR 6 turn normalization
8. PR 7 audit command
9. PR 8 docs/release hygiene

## Decision Memo

### Decision Summary

외부 점검안은 대체로 유효하지만, release/tap 격차는 현재 기준 stale로 판정한다. 즉시 보완은 CI, 한글 slug, metadata escaping 세 축으로 시작하고, 제품 핵심 가치인 turn normalization과 evidence audit은 module split 이후 별도 PR로 구현한다.

### Rationale

- P0 버그는 사용자의 한국어 workflow와 데이터 무결성을 직접 막는다.
- CI가 없으면 이후 TDD 계약이 로컬에만 남는다.
- per-turn parser와 audit은 중요하지만 blast radius가 크므로 구조 분리 후 처리해야 한다.
- 날짜 접두사, smart skill install, opencode row parsing은 현재 제품 계약을 흔들 수 있어 후순위가 맞다.

### Assumptions

- 기존 `note new` ASCII filename 계약은 유지해야 한다.
- 한글 파일명은 macOS, Obsidian, git에서 허용 가능한 기본 경로로 본다.
- `validate`를 갑자기 strict하게 만드는 것보다 `audit`을 추가하는 편이 기존 vault 사용자에게 덜 위험하다.
- dependency 0개 원칙은 장점이지만, structured JSON parsing과 SHA-256에는 예외를 검토할 수 있다.

### Dissent / Residual Concerns

- `serde_json`/`sha2` 도입은 install size와 dependency policy를 바꾼다.
- root-relative sync는 기존 flattened raw archive와 경로가 달라져 migration 문서가 필요하다.
- 날짜 접두사를 나중에 도입하면 또 한 번 filename contract를 바꿔야 할 수 있다.

### Revisit Trigger

- PR 1-3 이후에도 한국어 archive workflow가 막히는 경우.
- `sync`가 실제 사용자 session append를 누락하는 사례가 다시 확인되는 경우.
- release/tap 상태가 source version과 다시 어긋나는 경우.
