# Thought Castle 개선안 v2 (0.1.3 반영 후 재점검)

작성일: 2026-06-10
기준 커밋: `28a61c0` (v0.1.3)
이전 점검: `plans/repo-improvement-proposal.ko.md` (2026-06-09, 기준 `aae8033`)
관련 로드맵: `.codex/plans/audit-remediation-roadmap.md` (메인테이너의 이전 점검안 검토 결과)

## 1. 이번 반영분 확인 (`aae8033` → `28a61c0`)

12개 커밋이 추가됐고 RED(테스트 선행)/GREEN(구현) 커밋 분리가 유지됐다. 이전 점검안의 **P0 전부와 P1 일부가 해소**됐다. 아래 표의 검증은 이번 재점검에서 v0.1.3 빌드로 실제 재현한 결과다.

| 이전 점검 항목 | 상태 | 반영 커밋 | 재검증 결과 |
| --- | --- | --- | --- |
| 한글 슬러그가 `note.md`로 충돌 (P0) | **해결** | `110f808` | `중심극한정리.md`, `베이즈-정리.md` 생성 확인. 동일 제목 반복 시 `중심극한정리-2.md` 접미사 부여 확인 |
| JSON/YAML 제어문자 이스케이프 (P0) | **해결** | `c625d27` | 개행 포함 제목의 사이드카가 valid JSON으로 파싱되고 원문 복원됨 (`json.load` 확인) |
| CI 부재 (P0) | **해결** | `999f780` | ubuntu+macos matrix에서 fmt/clippy/test. CI 파일 내용을 고정하는 계약 테스트까지 추가됨 |
| `--version`/`--help`/`-h` 부재 (P1) | **해결** | `159c297` | `thought-castle 0.1.3` 출력 확인 |
| unknown/duplicate flag 무시, `--flag=value` 미지원 (P2) | **해결** | `159c297` | `unknown flag: --unknown` 에러, `--title=값` 동작 확인 |
| 릴리스 격차 (P1) | **대부분 해소** | `28a61c0` | remote tag `v0.1.3` + GitHub Release 발행(2026-06-09) 확인. tap formula·로컬 설치본 갱신은 이 환경에서 검증 불가 — `release-0.1.3.md` 9~13단계로 로컬 확인 필요 |

테스트는 18 → 25개로 늘었고(`cargo test` 전체 통과), fmt/clippy 모두 클린 유지.

### 이전 점검안에 대한 메인테이너 보정 수용

`audit-remediation-roadmap.md`의 보정 두 건은 타당하므로 v2에서 그대로 받아들인다.

- "Homebrew tap stable 0.1.0" 주장은 stale evidence였다 (당시 실제 0.1.2, 현재 0.1.3 릴리스 완료). 이후 릴리스 관련 점검은 로드맵의 원칙대로 `Cargo.toml` / git tag / tap formula / 설치본을 **동시에** 확인한다.
- 날짜 접두사 기본 적용과 smart skill install은 파일명·제품 계약 변경이므로 후순위 결정이 맞다.

## 2. 남은 핵심 작업 — 로드맵 PR 4~8에 동의, 보완 의견만 추가

남은 큰 덩어리는 메인테이너 로드맵이 이미 PR 단위로 정리했다. 순서(모듈 분리 → sync 안전성 → per-turn → audit → 문서 위생)에 동의하며, 재점검에서 확인한 보완 사항만 덧붙인다.

### PR 4: 모듈 분리

`src/main.rs`가 911줄로 늘었다(0.1.2 대비 +69). PR 5~7이 들어오기 전 분리가 여전히 선행 조건으로 유효하다. 아래 3.1의 `CliError::Io` 경로 컨텍스트 작업을 이 PR의 에러 타입 개편과 묶는 것을 권장.

### PR 5: sync 안전성 + SHA-256

재확인 결과 `sync`는 여전히 파일명 존재만으로 skip하고(`src/main.rs` `sync_sources`), `content_hash`는 `DefaultHasher` 기반이다. 로드맵 계획(변경 감지, root-relative 경로 또는 해시 접미사, hash algorithm version 필드)에 동의. 추가 제안:

- **`ingest`(일반)의 저장 경로 비일관도 이 PR에서 함께 정리 권장.** 재확인 결과 일반 ingest는 여전히 `00_raw-sessions/` 루트에 저장된다 (`sync`는 `<provider>/`, manual은 `manual/`). 이 항목은 로드맵에 배정되어 있지 않다.

### PR 6: per-turn normalization

재확인 결과 `session normalize`는 여전히 raw 전문을 `### t0001 source ^t0001` 단일 블록에 넣는다. 로드맵의 provider별 fixture 선행 접근에 동의.

### PR 7: audit 명령

`validate`(구조 검사)와 `audit`(invariant 검사) 분리 설계에 동의 — 기존 vault를 갑자기 invalid로 만들지 않는다는 근거가 적절하다. 아래 3.3의 lab 검증 일관화를 이 PR 또는 별도 소PR에 배정 권장.

### PR 8: 문서/릴리스 위생

- `plans/readme-usage-implementation-audit.ko.md`의 "Homebrew stable 0.1.0" 서술은 이제 이중으로 낡았다 (0.1.3 릴리스 완료). 갱신하거나 "historical audit" 표기 필요.
- `tasks/`·`subtasks/` 체크박스 현행화, `plans/` vs `.codex/plans/` 역할 구분 명시는 그대로 유효.

## 3. 이번 재점검의 신규 발견

### 3.1 [P2] 로드맵 PR 3에 계획된 "IO 에러 path context"가 미구현

로드맵 PR 3 GREEN 목록에 "I/O 에러에 path context를 붙이는 helper 추가"가 있으나 `159c297`에 포함되지 않았다. 재현:

```text
$ thought-castle ingest manual . --provider c --title t --file /nonexistent.md
error: No such file or directory (os error 2)   # 어느 경로인지 알 수 없음
```

PR 4의 에러 타입 개편(예: `CliError::Io { path, source }`)과 함께 처리하는 재배정을 권장.

### 3.2 [P3] `skill install --target=<path>` equals 형식 미지원

PR 3에서 `--flag=value`가 전 명령에 도입됐지만, `skill install --target`만 `Flags`를 거치지 않고 positional 패턴 매칭으로 파싱되어 equals 형식이 usage 에러가 된다 (재현 확인). `skill install`도 `Flags::parse` + `reject_unless(&["--target"])` 경유로 통일하면 해소되고 match arm 3개도 줄어든다.

### 3.3 [P3] `note new`·`session normalize`·`ingest`가 vault 밖에서도 동작 (이전 점검 항목, 로드맵 미배정)

재확인: `note new knowledge /tmp/아무경로`가 그 자리에 `10_knowledge/`를 만들며 성공한다. `source list`/`sync`만 lab 검사를 한다. 최소한 core 디렉터리 존재 검사를 쓰기 계열 명령에 일관 적용 권장. 로드맵에 배정되지 않은 항목이므로 PR 7 전후로 배치 필요.

### 3.4 [P3] help/usage 텍스트와 README에 `--version`/`--help` 미표기

`print_help`(`src/main.rs:138`)와 usage 에러 문자열, README 모두 새 플래그를 안내하지 않는다. 한 줄 추가 수준의 작업.

### 3.5 [P3] CI 미세 조정

- `on: push` + `pull_request`라서 PR 브랜치에서 push/PR 이벤트가 **중복 실행**된다. push 트리거를 `branches: [main]` + tags로 제한하거나 `concurrency` 그룹 추가 권장.
- Rust 빌드 캐시 없음(`Swatinem/rust-cache` 등). 저장소가 작아 시급하지 않음.
- 버전 계약 테스트가 `thought-castle 0.1.3` 정확 문자열을 고정 → 릴리스마다 RED 커밋으로 갱신하는 의도된 ritual로 보인다. 유지해도 무방하나, 릴리스 자동화(3.6) 도입 시에는 `CARGO_PKG_VERSION` 비교로 완화하는 편이 맞물린다.

### 3.6 [P2] 릴리스가 여전히 수동 ritual

`release-0.1.3.md`는 잘 짜인 수동 체크리스트지만 사람이 13단계를 수행한다. tag push 시 GitHub Release 생성 + tarball sha256 계산 + tap formula bump까지 워크플로화하면, 로드맵 Revisit Trigger("release/tap 상태가 source version과 다시 어긋나는 경우")를 구조적으로 예방한다. 메인테이너가 검증 단계가 있는 수동 절차를 선호하는 것으로 보이므로 P2로 유지.

## 4. 우선순위 갱신

| 순위 | 항목 | 비고 |
| --- | --- | --- |
| P0 | 없음 | 이전 P0 전부 해소 |
| P1 | 로드맵 PR 4 → 5 → 6 → 7 순차 진행 | 순서 동의. PR 4에 3.1(IO context)·3.2(skill install 플래그 통일) 흡수 |
| P2 | 3.6 릴리스 자동화, PR 8 문서 위생(+3.4) | tap 격차 재발 방지 |
| P3 | 3.3 lab 검증 일관화, 3.5 CI 조정, source_refs `source_type`/`--confidence` 플래그(이전 3.4항, 여전히 미반영) | 소규모, 인접 PR에 편승 |

## 5. 결론

0.1.3은 이전 점검안의 P0를 전부 해소했고, 검증 결과 모두 의도대로 동작한다. 남은 작업은 메인테이너 로드맵 PR 4~8 순서대로 진행하면 되며, 이번 재점검의 신규 발견은 전부 소규모로 기존 PR에 흡수 가능하다: **PR 4에 3.1·3.2, PR 5에 ingest 경로 통일, PR 7 부근에 3.3, PR 8에 3.4·3.5.** 독립적으로 새로 시작할 작업은 릴리스 자동화(3.6) 하나다.
