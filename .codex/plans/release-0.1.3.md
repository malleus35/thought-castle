# Thought Castle 0.1.3 Release Plan

작성일: 2026-06-10
대상 브랜치: `chore/PROJ-0-audit-remediation-p0`
목표 버전: `0.1.3`
사용 워크플로: `agora` -> `plan-stress-test`

## 목적

`chore/PROJ-0-audit-remediation-p0`의 audit remediation 변경을 `main`에 반영하고, tagged source release, Homebrew tap formula, 로컬 Homebrew 설치본, 로컬 agent skill install까지 최신으로 맞춘다.

## 현재 확인된 stale 지점

- `main`은 아직 `aae8033` / `v0.1.2`에 머물러 있다.
- feature branch HEAD는 `159c297`이며 `main`보다 9개 커밋 앞서 있다.
- `/opt/homebrew/Library/Taps/malleus35/homebrew-tap/Formula/thought-castle.rb`는 `v0.1.2` tarball을 가리킨다.
- `/opt/homebrew/bin/thought-castle --version`은 실패한다. 설치본은 새 CLI hardening 변경을 아직 받지 않았다.

## Doubt List

### Happy Path Doubts

- Doubt: feature branch만 push하고 release가 끝났다고 착각할 수 있다.
- Verification: `main`, git tag, tap formula, installed binary를 각각 확인한다.

### Edge Case Doubts

- Doubt: GitHub tag tarball sha256을 tag push 전에 계산하면 formula가 깨진다.
- Verification: `v0.1.3` tag push 이후 GitHub archive URL을 다운로드해 sha256을 계산한다.

- Doubt: Homebrew formula는 바뀌었지만 로컬 설치본이 cache/old formula를 계속 쓸 수 있다.
- Verification: `brew reinstall malleus35/tap/thought-castle`, `thought-castle --version`, `brew test malleus35/tap/thought-castle`를 실행한다.

### Boundary Doubts

- Doubt: tap repo가 dirty하면 formula commit이 unrelated changes를 포함할 수 있다.
- Verification: tap repo `git status --short --branch`를 확인하고 formula만 stage한다.

- Doubt: feature branch의 RED commits를 main에 fast-forward하면 main history에 failing commits가 남는다.
- Verification: 이 repo의 규칙은 RED/GREEN 커밋 분리를 요구하므로 의도된 형태로 본다. release tag는 최종 GREEN HEAD에만 붙인다.

### Ambiguity Doubts

- Doubt: "release"가 tag만 의미하는지 GitHub Release page까지 의미하는지 애매하다.
- Verification: tag push는 필수로 수행하고, `gh release create v0.1.3`도 가능한 경우 수행한다. 실패 시 tag+tap release는 완료로 보고 실패 이유를 기록한다.

### Evil Demon Scenarios

- Doubt: remote main이 그 사이 바뀌었는데 로컬 main 기준으로 밀어버릴 수 있다.
- Verification: push 전 `git fetch origin --tags` 후 main ancestry를 확인한다. fast-forward만 사용한다.

## 실행 순서

1. Release plan 문서 commit.
2. RED: CLI version contract를 `thought-castle 0.1.3`으로 고정하고 실패 확인.
3. GREEN: `Cargo.toml` version을 `0.1.3`으로 bump하고 전체 검증.
4. `main`으로 fast-forward merge.
5. `main` push.
6. `v0.1.3` tag 생성 및 push.
7. GitHub Release 생성 시도.
8. GitHub tag tarball sha256 계산.
9. Homebrew tap formula를 `v0.1.3` URL/sha256으로 갱신, commit, push.
10. `brew reinstall malleus35/tap/thought-castle`.
11. `brew test malleus35/tap/thought-castle`.
12. `/opt/homebrew/bin/thought-castle --version`이 `thought-castle 0.1.3`인지 확인.
13. `/opt/homebrew/bin/thought-castle skill install`로 로컬 agent skill copies refresh.

## 완료 조건

- `origin/main`이 release commit을 포함한다.
- remote tag `v0.1.3`가 존재한다.
- Homebrew tap formula가 `v0.1.3` URL과 matching sha256을 가진다.
- installed binary가 `thought-castle 0.1.3`을 출력한다.
- `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `brew test`가 통과한다.
