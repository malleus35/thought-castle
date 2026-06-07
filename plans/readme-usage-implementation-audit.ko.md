# README 사용법 및 구현 괴리 조사

작성일: 2026-06-07

## 결론

현재 Thought Castle의 방향은 README와 코드 모두 "검증된 지식 아카이빙"으로 맞춰졌다. `40_posts` 기반 게시 파이프라인은 새 vault 구조, CLI note 생성, skill 문서, PRD/task 문서에서 제거됐다.

다만 배포 경로에는 중요한 괴리가 있다. `main`의 소스 빌드는 archive-only 계약을 따르지만, Homebrew stable은 아직 `0.1.0`이고 설치된 바이너리도 `0.1.0`이다. 따라서 Homebrew 설치 사용자는 다음 릴리스 전까지 최신 README와 다른 동작을 볼 수 있다.

## README 기준 사용 방법

### 1. 설치

릴리스 버전 설치:

```bash
brew install malleus35/tap/thought-castle
```

최신 `main` 기준 archive-only 기능을 바로 쓰려면 source install을 사용한다.

```bash
git clone https://github.com/malleus35/thought-castle.git
cd thought-castle
cargo install --path .
```

### 2. vault 생성과 검증

```bash
thought-castle init ~/thought-castle-lab
thought-castle validate ~/thought-castle-lab
cd ~/thought-castle-lab
```

생성되는 핵심 구조:

```text
00_raw-sessions
01_sessions
10_knowledge
20_thoughts
30_ideas
_templates
_system
plans
tasks
subtasks
```

`40_posts`는 더 이상 생성되지 않는다.

### 3. 자동 저장되는 로컬 agent 세션 확인

아래 명령은 후보 파일 경로와 개수만 보여준다. 메시지 본문을 출력하지 않는다.

```bash
thought-castle source list . --provider codex --root ~/.codex/sessions
thought-castle source list . --provider claude-code --root ~/.claude/projects
thought-castle source list . --provider opencode --root ~/.local/share/opencode
thought-castle source list . --provider pi-agent --root ~/.pi/agent/sessions
```

현재 구현 기준 provider별 discovery 방식:

| Provider | 구현된 discovery |
| --- | --- |
| `codex` | 지정 root 아래 `*.jsonl` 재귀 탐색 |
| `claude-code` | 지정 root 아래 `*.jsonl` 재귀 탐색 |
| `pi-agent` | 지정 root 아래 `*.jsonl` 재귀 탐색 |
| `opencode` | 지정 root 아래 `opencode.db` 파일 탐색 |

### 4. 자동 저장 세션 sync

```bash
thought-castle sync . --provider codex --root ~/.codex/sessions
thought-castle sync . --provider claude-code --root ~/.claude/projects
thought-castle sync . --provider opencode --root ~/.local/share/opencode
thought-castle sync . --provider pi-agent --root ~/.pi/agent/sessions
```

sync 결과는 `00_raw-sessions/<provider>/` 아래로 복사되고, `<filename>.meta.json` sidecar가 생성된다.

주의: 현재 sync는 raw artifact 복사 단계다. Codex, Claude Code, Pi Agent JSONL을 의미 단위로 파싱하지 않고, OpenCode도 SQLite row를 읽지 않고 DB snapshot 파일을 복사한다.

### 5. 수동 캡처 ingest

ChatGPT, Claude 웹/데스크탑, Perplexity, export zip에서 꺼낸 파일, 직접 복사한 transcript는 manual ingest를 쓴다.

```bash
thought-castle ingest manual . \
  --provider chatgpt \
  --title "LLM Wiki Conversation" \
  --file ./thread.md
```

결과는 `00_raw-sessions/manual/` 아래에 저장되고 metadata sidecar가 생성된다.

### 6. session normalize

```bash
thought-castle session normalize . 00_raw-sessions/manual/thread.md \
  --title "LLM Wiki Conversation" \
  --source-type ai_conversation
```

현재 구현은 raw file 전체를 `t0001` block 하나로 넣는다. 요약, context, turn 분리, speaker 분리는 아직 자동화되어 있지 않다.

### 7. traceable note 생성

```bash
thought-castle note new knowledge . \
  --title "Persistent Wiki Pattern" \
  --session "[[01_sessions/llm-wiki-conversation.md#^t0001]]" \
  --raw-file "00_raw-sessions/manual/thread.md"

thought-castle note new thought . \
  --title "My Verification Bias" \
  --session "[[01_sessions/llm-wiki-conversation.md#^t0001]]" \
  --raw-file "00_raw-sessions/manual/thread.md"

thought-castle note new idea . \
  --title "Session Evidence Review Loop" \
  --session "[[01_sessions/llm-wiki-conversation.md#^t0001]]" \
  --raw-file "00_raw-sessions/manual/thread.md"
```

지원 note kind는 `knowledge`, `thought`, `idea` 세 가지뿐이다.

## README와 코드 구현 대조

| README 내용 | 코드 구현 상태 | 판정 |
| --- | --- | --- |
| verified knowledge archive 목적 | README, skill, PRD, CLI contract가 같은 방향으로 정리됨 | 일치 |
| Homebrew 설치 | tap stable은 `0.1.0`; 현재 `main` 변경과 릴리스가 아직 연결되지 않음 | 괴리 있음 |
| source install | `cargo install --path .`로 현재 소스 설치 가능 | 일치 |
| `init` 핵심 구조 | `CORE_DIRS`가 README 구조와 일치하고 `40_posts` 없음 | 일치 |
| `validate` | core folders, templates, `source_refs`, thought `user_confirmed` 검증 | 부분 구현 |
| `source list` | provider별 후보 탐색 구현 | 일치 |
| `sync` | raw file 또는 DB snapshot 복사와 metadata sidecar 구현 | 부분 구현 |
| manual ingest | `00_raw-sessions/manual/` 복사와 metadata sidecar 구현 | 일치 |
| session normalize | canonical Markdown 생성, block id `^t0001` 생성 | 부분 구현 |
| note new | `knowledge`, `thought`, `idea` 생성 및 `source_refs` 주입 | 일치 |
| verification model | template에는 verification field가 있으나 실제 검증 자동화는 없음 | 수동 필요 |
| graphify 차별점 | README에 전략 설명만 있음. graphify 연동 코드는 없음 | 의도된 non-goal |

## 확인된 괴리와 리스크

### 1. Homebrew stable이 최신 코드와 다름

확인 결과:

- `Cargo.toml` version: `0.1.0`
- git tag: `v0.1.0`
- `brew info malleus35/tap/thought-castle`: stable `0.1.0`
- 설치된 `thought-castle` help는 아직 `note new <kind>`처럼 generic kind를 보여준다.
- 현재 source build help는 `note new <knowledge|thought|idea>`를 보여준다.

따라서 README의 Homebrew 설치만 따라가면 archive-only 변경 전 바이너리를 사용할 가능성이 있다. 다음 릴리스에서 버전 bump, tag, tap formula update가 필요하다.

### 2. 자동 sync는 "수집"이지 "이해"가 아니다

`sync`는 원본 파일을 안전하게 가져오는 단계다. 아직 다음 기능은 없다.

- session format별 turn parsing
- topic routing
- claim extraction
- evidence extraction
- knowledge verification
- thought confirmation
- idea evaluation

### 3. OpenCode는 DB snapshot만 복사한다

README의 "sync automatic local sessions" 표현은 큰 방향에서는 맞지만, OpenCode의 현재 구현은 `opencode.db`를 raw artifact로 복사하는 수준이다. SQLite schema 분석과 row-level session extraction은 별도 구현이 필요하다.

### 4. normalize는 최소 canonical form이다

현재 `session normalize`는 raw text 전체를 하나의 `t0001` source block으로 저장한다. 실제 학습 세션 archive로 쓰려면 사람이 또는 agent가 이후에 turn split, summary, context, extracted candidates를 보완해야 한다.

### 5. 검증 상태 전이는 사람이 관리해야 한다

`10_knowledge` template에는 verification field가 있지만 CLI가 외부 근거를 찾아 검증하거나 `verified`로 승격하지 않는다. `20_thoughts`의 `user_confirmed: false`도 CLI가 자동으로 true로 바꾸지 않는다.

## 어디까지 수동으로 관리해야 하는가

### 매번 수동 또는 반자동 관리가 필요한 부분

- ChatGPT, Claude 웹/데스크탑, Perplexity 대화 export 또는 복사
- export zip에서 필요한 transcript 파일 추출
- manual ingest 대상 파일명과 title 지정
- normalized session의 summary, context 작성
- raw text를 실제 turn 단위로 분리하는 작업
- topic별 분류와 tag 지정
- knowledge claim이 사실인지 외부 근거로 검증
- `verification.status`, `method`, `evidence`, `verified_at` 입력
- thought가 실제 사용자 생각인지 확인하고 `user_confirmed` 갱신
- idea의 채택, 폐기, 실험 여부 판단
- graphify 실행과 graph 결과 해석
- Homebrew 릴리스 전까지 source install 또는 로컬 빌드 선택

### 현재 자동화된 부분

- vault skeleton 생성
- 기본 template 생성
- 기본 구조 validation
- Codex, Claude Code, Pi Agent JSONL 후보 탐색
- OpenCode DB 후보 탐색
- raw artifact sync와 metadata sidecar 생성
- manual capture copy와 metadata sidecar 생성
- 최소 session Markdown 생성
- `knowledge`, `thought`, `idea` note 초안 생성
- `source_refs` 주입

## 권장 다음 작업

1. `0.1.1` 릴리스 준비
   - `Cargo.toml` version bump
   - 새 tag 생성
   - Homebrew tap formula update
   - 설치된 Homebrew 바이너리 help가 README와 일치하는지 확인

2. normalize 고도화
   - JSONL provider별 turn parser 추가
   - Pi Agent tree 구조 parser 추가
   - Codex/Claude Code message role 추출 추가

3. verification workflow 추가
   - `knowledge verify` 또는 `knowledge evidence add` 명령 설계
   - `verified` 승격 조건을 CLI에서 검증

4. manual capture workflow 정교화
   - export zip ingest
   - copied transcript ingest guide
   - provider별 manual capture naming convention

5. graphify 연동은 source-of-truth가 아니라 analysis layer로 설계
   - Thought Castle vault를 입력으로 graphify 실행
   - graph 결과는 후보 관계로만 취급
   - verified knowledge 상태를 graphify inferred edge와 섞지 않기
