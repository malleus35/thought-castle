# Thought Castle 세션 동기화 및 검증 가능한 지식체계 계획

## 목적

이 계획서는 LLM 대화 세션을 자동 또는 반자동으로 수집하고, 안정적인 세션 기록으로 정규화한 뒤, 검증 가능한 개인 지식 저장소로 컴파일하기 위한 조사 결과와 구현 방향을 정리한다.

초기 목표였던 아이디어 생성기는 계속 중요하다. 하지만 현재 더 중요한 목표는 사용자가 LLM들과 공부하며 형성한 이해, 질문, 목표, 판단을 추적 가능한 지식체계로 만드는 것이다.

## 현재 제품 상태

Thought Castle은 이미 다섯 계층의 파이프라인을 정의한다.

```text
00_raw-sessions  -> 원본 세션 보존
01_sessions      -> 안정적인 block reference를 가진 Markdown 정규화 세션
10_knowledge     -> 객관 지식 후보와 검증된 사실
20_thoughts      -> 사용자의 이해, 판단, 감정, 맥락
30_ideas         -> 창의적 조합과 실험 후보
40_posts         -> 플랫폼별 초안과 발행 기록
```

현재 Rust CLI가 지원하는 기능은 다음과 같다.

- `init`: vault 골격, 템플릿, 시스템 문서 생성
- `validate`: 필수 폴더/템플릿과 기본 invariant 검증
- `ingest`: raw 파일을 `00_raw-sessions`로 복사하고 metadata sidecar 작성
- `session normalize`: 최소 형태의 canonical Markdown session 생성
- `note new`: `knowledge`, `thought`, `idea`, `post` 초안 생성 및 `source_refs` 주입
- `skill print`, `skill install`: Thought Castle agent skill 출력/설치

현재 테스트는 통과하지만, provider별 session sync, topic routing, claim extraction, evidence verification, graphify 연동은 아직 구현되지 않았다.

## 조사 요약

### ChatGPT

일반 소비자 ChatGPT 계정에서 공식적으로 지원되는 경로는 ChatGPT Settings/Data Controls 또는 OpenAI Privacy Portal을 통한 데이터 export다. OpenAI 도움말은 export zip에 chat history와 관련 account data가 포함된다고 설명한다.

ChatGPT macOS 앱은 웹 제품과 같은 데이터 보존 정책을 따른다. OpenAI의 macOS data retention 문서는 macOS 앱으로 업로드한 파일이 로컬이 아니라 클라우드에 저장되고 OpenAI 계정에 연결된다고 설명한다.

2024년에는 ChatGPT macOS 앱이 `~/Library/Application Support/com.openai.chat` 아래 대화를 plaintext로 저장했다는 보도가 있었다. 이후 OpenAI가 앱을 업데이트했고, 현재 로컬 desktop storage를 안정적인 공개 인터페이스로 간주하면 안 된다.

Enterprise/Edu workspace에는 Compliance Platform/Compliance API가 있고, workspace의 logs와 metadata를 가져올 수 있다. 하지만 일반 consumer 계정에서는 사용할 수 없다.

결론: consumer ChatGPT는 export zip ingestion부터 시작해야 한다. live periodic sync는 공식 공개 API가 없으므로 browser automation, browser extension, share-link capture 같은 비공식 방식에 의존해야 한다. 이는 가능은 하지만 취약하고 실험 기능으로 취급해야 한다.

### Claude Web 및 Claude Desktop

Claude는 web app 또는 Claude Desktop의 Settings/Privacy에서 공식 data export를 지원한다. export에는 conversation data와 account data가 포함된다.

Claude Desktop에는 별도의 local agent/Cowork 저장소도 있다. Anthropic 문서는 `local-agent-mode-sessions/`를 Cowork conversation history로, `claude-code-sessions/`를 Code tab conversation history로 설명한다. `IndexedDB`, `Local Storage`, `Session Storage`는 주요 conversation 원본이 아니라 renderer-side UI state로 문서화되어 있다.

이 머신에서는 `~/Library/Application Support/Claude`가 존재하고, 그 안에 `IndexedDB`, `Local Storage`, `local-agent-mode-sessions`, `claude-code`, `claude-code-vm`가 확인되었다.

결론: Claude web history는 먼저 official export를 써야 한다. Claude Desktop Cowork/3P session은 사용자가 명시적으로 허용할 때 read-only local sync 대상으로 삼을 수 있다.

### Claude Code

Claude Code는 강한 local sync 경로가 있다. Anthropic 문서는 각 message, tool use, result가 `~/.claude/projects/` 아래 plaintext JSONL로 기록된다고 설명한다.

이 머신에서도 `~/.claude/projects` 아래 다수의 JSONL session file이 확인되었다.

결론: Claude Code는 자동 sync adapter의 1순위다.

### Codex

이 머신에서는 Codex session data가 `~/.codex/sessions/2026/.../*.jsonl` 아래 존재한다. 추가로 `~/.codex/session_index.jsonl`, `~/.codex/history.jsonl` 같은 index/history 파일도 확인되었다.

결론: Codex도 자동 sync adapter의 1순위다. 구현 시 discovery 단계에서는 파일 metadata만 보고, 실제 session content ingest는 명시적인 sync 명령에서 수행해야 한다.

### OpenCode

이 머신에서는 OpenCode local data가 `~/.local/share/opencode/opencode.db`에 존재한다.

결론: OpenCode는 read-only SQLite adapter로 다루는 것이 맞다. 첫 구현은 schema inspection만 수행하고, fixture 기반 failing test 이후 parser를 추가해야 한다.

### Pi Agent

Pi Agent는 강한 local sync 경로가 있다. Pi sessions 문서는 session이 `~/.pi/agent/sessions/` 아래 working directory별로 자동 저장되며, 각 session이 tree structure를 가진 JSONL file이라고 설명한다. session format 문서는 구체적인 path 형태도 `~/.pi/agent/sessions/--<path>--/<timestamp>_<uuid>.jsonl`로 제시한다.

Pi session file에는 header, message, model change, thinking-level change, compaction, branch summary, custom entry, label이 들어가며 `id`와 `parentId`로 tree link를 구성한다.

결론: Pi Agent는 자동 sync adapter의 1순위다.

### Consumer Pi Chatbot

이 머신에서는 consumer Pi용 `/Applications/Pi.app`, `/Applications/Inflection Pi.app`, `heypi`, `Inflection` 계열 local storage가 확인되지 않았다. 공개 검색에서도 안정적인 first-party local sync API는 확인하지 못했다. export 또는 sample 확보가 우선이다.

결론: consumer Pi 챗봇은 export format 또는 안정적인 storage path가 확인될 때까지 manual/sample-needed 상태로 둔다.

## 자동 동기화는 불가능한가?

불가능하지 않다. 다만 provider별로 전략이 달라야 한다.

## 라우팅 결정

Thought Castle ingestion은 두 갈래로 분리한다.

### Automatic Lane

이미 durable local session file 또는 database를 자동으로 쓰는 source만 자동화한다.

- Codex local JSONL sessions
- Claude Code local JSONL sessions
- OpenCode local SQLite sessions
- Pi Agent local JSONL sessions

이 lane은 source of truth가 이미 local에 지속 저장되므로 scheduled sync, `thought-castle sync`, background watcher로 확장할 수 있다.

### Manual or Export Lane

웹/데스크탑 앱은 사용자의 명시적 행동으로 Thought Castle에 들어온다.

- ChatGPT와 Claude의 official export zip ingestion
- Perplexity에서 사용자가 다운로드한 thread export
- 단일 대화 copy/paste capture
- Markdown, text, HTML, JSON, PDF, DOCX 같은 manual raw session file

이 lane도 raw file, metadata sidecar, normalized session, source refs를 만든다. 다만 live automatic sync인 것처럼 취급하지 않는다.

### 가능하고 우선 적용해야 하는 경로

- Claude Code: `~/.claude/projects` JSONL sync
- Codex: `~/.codex/sessions` JSONL sync
- OpenCode: `~/.local/share/opencode/opencode.db` read-only SQLite sync
- Pi Agent: `~/.pi/agent/sessions` JSONL sync
- Claude Desktop Cowork/3P: 문서화된 `local-agent-mode-sessions` read-only sync. 단, 사용자의 명시적 승인 필요

### 가능하지만 수동 또는 반자동인 경로

- ChatGPT consumer: official export zip ingestion
- Claude web/standard account: official export zip ingestion
- Perplexity: 사용자 다운로드 export 또는 manual copy/paste ingestion
- Consumer Pi chatbot: export 또는 sample 확보 후 ingestion

### 가능하지만 실험 기능으로 봐야 하는 경로

- 로그인된 브라우저에서 ChatGPT/Claude 대화를 export하는 browser extension
- 로그인된 브라우저나 desktop app UI를 자동화하는 Playwright/브라우저 automation
- ChatGPT share link 기반 selected conversation capture
- web sidebar/conversation page scraping

이 방식들은 완전히 불가능하지 않다. 하지만 UI 변경에 취약하고, 긴 대화의 pagination/hidden message를 놓칠 수 있으며, 플랫폼 약관이나 개인정보 리스크가 있다. 따라서 핵심 ingestion 경로가 아니라 experimental adapter로 분리해야 한다.

### 전략적 대안

앞으로 새로 하는 대화는 forward capture 방식이 가장 안정적이다. 즉, Thought Castle CLI/skill/MCP workflow를 학습 세션의 입구로 쓰고, 그 workflow가 대화를 기록하면서 target LLM/API에 질문을 보내거나 사용자가 답변을 붙여넣게 한다.

이 방식은 private web-app storage에 의존하지 않고, 처음부터 source trace를 보존할 수 있다.

## Karpathy LLM Wiki와 비교

Karpathy의 LLM Wiki pattern은 세 계층을 가진다.

- immutable raw sources
- LLM이 유지보수하는 Markdown wiki pages
- ingest/query/lint workflow를 지시하는 schema 또는 instructions file

핵심은 compilation이다. LLM이 매 질문마다 raw source를 다시 RAG처럼 찾고 조합하는 것이 아니라, 읽은 내용을 지속적인 wiki에 반영해 지식이 복리로 축적되게 한다.

Thought Castle은 이 write-back 원칙을 받아들이되, 더 엄격한 개인 epistemology를 가져야 한다.

- `10_knowledge`는 곧바로 truth가 아니다. 기본은 `candidate`다.
- `verified`는 evidence가 있어야 한다.
- `20_thoughts`는 objective claim과 사용자의 해석을 분리한다.
- `30_ideas`는 raw chat fragment가 아니라 knowledge/thought에서 파생되어야 한다.
- 모든 파생 노트는 `source_refs`를 보존해야 한다.

요약하면 Karpathy LLM Wiki는 LLM이 유지하는 지식 wiki이고, Thought Castle은 source, claim, interpretation, idea, publication을 더 명확히 분리하는 검증형 학습/아이디어 시스템이어야 한다.

## graphify와 비교

graphify는 corpus를 knowledge graph로 만들고, community detection, graph JSON, visualization, audit report를 생성한다. 문서 간 비자명한 연결을 찾고, edge를 extracted, inferred, ambiguous로 구분하는 데 강하다.

Thought Castle은 graphify로 대체되면 안 된다. 두 시스템의 역할을 나눠야 한다.

- Thought Castle은 source lifecycle, verification gate, user-facing knowledge file을 소유한다.
- graphify는 vault를 분석해 관계, community, surprising connection을 보여준다.
- graphify 결과는 canonical truth가 아니라 analytical artifact로 취급한다.

권장 통합 구조:

```text
Thought Castle vault
  -> 01_sessions, 10_knowledge, 20_thoughts, 30_ideas를 graphify로 분석
  -> graphify-out/graph.json 및 GRAPH_REPORT.md 생성
  -> 선택된 insight만 source_refs를 가진 30_ideas 또는 10_knowledge candidate로 승격
```

## 제안 아키텍처

### Source Adapter

각 adapter는 같은 계약을 가져야 한다.

```text
discover -> 전체 본문을 읽지 않고 candidate source session 목록만 확인
sync     -> 선택된 raw session을 00_raw-sessions로 copy/snapshot
parse    -> provider format을 normalized session turn으로 변환
index    -> sync status, provider id, hash, title, timestamp, topic hint 기록
```

초기 adapter:

- `codex-local-jsonl`
- `claude-code-jsonl`
- `opencode-sqlite`
- `pi-agent-jsonl`
- `claude-desktop-cowork`
- `manual-raw-session`
- `chatgpt-export-zip`
- `claude-export-zip`
- `perplexity-export-or-manual`
- `pi-consumer-export-sample`

### Raw Session Index

`_system/session-index.jsonl` 또는 `_system/session-index.sqlite`에 durable session index를 둔다.

필수 field:

- `provider`
- `provider_session_id`
- `source_path`
- `raw_path`
- `content_hash`
- `title`
- `created_at`
- `updated_at`
- `last_synced_at`
- `sync_status`
- `privacy_review`
- `topic_hints`
- `normalized_session_path`

### Sync Commands

제안 CLI:

```bash
thought-castle source list <lab> --provider codex
thought-castle sync <lab> --provider codex --since 7d
thought-castle sync <lab> --provider claude-code --project /path/to/project
thought-castle sync <lab> --provider opencode
thought-castle sync <lab> --provider pi-agent
thought-castle sync <lab> --provider chatgpt-export --file ~/Downloads/chatgpt-export.zip
thought-castle sync <lab> --provider claude-export --file ~/Downloads/claude-export.zip
thought-castle ingest manual <lab> --provider perplexity --title "Thread Title" --file ./thread.md
thought-castle ingest paste <lab> --provider chatgpt --title "Conversation Title"
thought-castle normalize pending <lab>
thought-castle extract knowledge <lab> --session 01_sessions/foo.md
thought-castle verify <lab> --knowledge 10_knowledge/foo.md
```

### Topic Routing

Topic classification은 truth를 결정하면 안 된다. review 가능한 folder/tag routing만 수행해야 한다.

예상 metadata:

```yaml
topics:
  - machine-learning/llm
  - software-engineering/agents
learning_intent:
  - understand
  - debug
  - compare
  - design
```

### Knowledge Extraction

`10_knowledge` candidate는 검증 가능한 claim일 때만 생성한다.

필수 요소:

- claim
- source_refs
- confidence
- verification.status
- evidence
- caveats
- contradicted_by

규칙:

- LLM 답변 자체는 evidence가 아니다.
- 사용자의 가설은 사실이 아니다.
- 중요한 claim이어도 `needs_verification`에 머물 수 있다.
- `verified`는 공식 문서, primary source, 재현 가능한 코드, 명시적 evidence가 필요하다.

### Thought Extraction

`20_thoughts`는 사용자의 이해, 혼란, 선호, 목표, 해석을 담는다.

규칙:

- 기본 상태는 `draft`다.
- `user_confirmed`는 기본 false다.
- agent가 추론한 감정이나 의도는 반드시 inferred로 표시한다.

### Idea Generation

아이디어는 raw chat fragment만으로 생성하지 않는다. 검토된 재료에서 생성한다.

입력:

- verified 또는 high-value knowledge candidate
- confirmed 또는 reviewing thought
- graphify surprise connection
- 반복해서 등장하는 학습 질문

출력:

- `30_ideas/*.md`
- method
- source_refs
- input materials
- smallest next experiment
- risk review

## 구현 단계

### Phase 1: Local Session Inventory

목표: full content를 import하지 않고 provider별 session source를 안전하게 발견한다.

작업:

- provider inventory 문서 추가
- Codex, Claude Code, OpenCode, Pi Agent, ChatGPT export, Claude export, manual raw session fixture 추가
- `source list` failing test 작성
- local JSONL 및 SQLite 위치 read-only discovery 구현

완료 기준:

- CLI가 conversation 본문을 출력하지 않고 local source candidate를 나열할 수 있다.

### Phase 2: Raw Sync

목표: 선택된 session을 `00_raw-sessions`로 copy/snapshot한다.

작업:

- session index 추가
- sync command 추가
- 원본 raw 보존
- content hash 및 duplicate detection 추가

완료 기준:

- sync를 다시 실행해도 unchanged session이 중복 생성되지 않는다.
- raw file과 sidecar가 deterministic하게 생성된다.

### Phase 3: Provider Parsers

목표: provider별 raw format을 canonical `01_sessions`로 변환한다.

작업:

- Codex JSONL parser
- Claude Code JSONL parser
- OpenCode SQLite fixture parser
- Pi Agent JSONL parser
- ChatGPT export zip parser
- Claude export zip parser
- manual Markdown/text/HTML paste file normalize

완료 기준:

- 각 provider fixture가 stable turn id와 source refs를 가진 Markdown session으로 변환된다.

### Phase 4: Knowledge Compilation

목표: normalized session에서 검토 가능한 후보를 생성한다.

작업:

- claim extraction workflow 추가
- thought extraction workflow 추가
- topic routing 추가
- 모든 derived note에 source refs가 있는지 검증

완료 기준:

- 추출된 knowledge는 `candidate` 또는 `needs_verification` 상태다.
- 추출된 thought는 `draft`, `user_confirmed: false` 상태다.

### Phase 5: Verification and Graph Loop

목표: 시스템을 신뢰 가능하고 synthesis에 유용하게 만든다.

작업:

- verification checklist 추가
- source existence validator 추가
- graphify export/run/report integration 추가
- 선택된 graph insight를 idea 또는 knowledge candidate로 반영

완료 기준:

- `verified`, `stable`, `published` 상태는 gate 조건 없이는 막힌다.
- graphify report는 생성 가능하지만 자동으로 canonical truth가 되지 않는다.

## Privacy and Safety Rules

- discovery는 full conversation content를 읽거나 출력하지 않는다.
- sync는 provider별 opt-in이어야 한다.
- encrypted/private app database를 조용히 reverse engineer하지 않는다.
- browser automation과 extension은 experimental이며 명시적 사용자 승인이 필요하다.
- raw session은 immutable이어야 한다.
- 민감한 session은 extraction 전 privacy review status를 가져야 한다.
- 생성된 knowledge는 evidence 없이 `verified`가 될 수 없다.

## 즉시 다음 단계

다음 구현 작업은 local-first provider의 `source list`가 가장 적절하다.

1. Codex local JSONL discovery
2. Claude Code local JSONL discovery
3. OpenCode SQLite schema discovery
4. Pi Agent local JSONL discovery

이 단계가 생기면 brittle한 web-app scraping을 먼저 건드리지 않고도 자동 주기 sync의 기반을 만들 수 있다.

## 사용한 자료

- OpenAI ChatGPT data export: https://help.openai.com/en/articles/7260999-how-do-i-export-my-chatgpt-history-and-data
- OpenAI ChatGPT macOS retention: https://help.openai.com/en/articles/9268871-how-is-data-retained-in-the-macos-app
- OpenAI Compliance Platform: https://help.openai.com/en/articles/9261474-compliance-api-for-chatgpt-enterprise-edu-and-chatgpt-for-teachers
- Claude data export: https://support.claude.com/en/articles/9450526-how-can-i-export-my-claude-data
- Claude Desktop local data: https://claude.com/docs/cowork/3p/data-storage
- Claude Code sessions: https://code.claude.com/docs/en/how-claude-code-works
- Pi Agent sessions: https://pi.dev/docs/latest/sessions
- Pi Agent session format: https://pi.dev/docs/latest/session-format
- Karpathy LLM Wiki: https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f
