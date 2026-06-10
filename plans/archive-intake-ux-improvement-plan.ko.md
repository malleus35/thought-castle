# Thought Castle Archive Intake UX 개선 계획

작성일: 2026-06-10
작성 언어: 한국어
사용 워크플로: `agora` 기반 간소화 리뷰
입력 근거: 사용자 실사용 피드백, 로컬 repo/vault 확인, subagent UX 리뷰, 웹 자료 조사

## 1. 결론

현재 Thought Castle은 raw 보존과 traceable note 구조는 잡혔지만, 사용자가 기대한 "대화 세션을 읽을 수 있게 정리하고, 주제별 지식/생각/아이디어로 자연스럽게 뽑아주는 경험"까지는 아직 부족하다.

가장 큰 문제는 세 가지다.

1. 언어가 제품 계약에 없다. 그래서 대화는 한국어였는데 템플릿과 스킬 본문이 영어라 결과물이 영어로 흐른다.
2. archive intake가 "전체 후보를 보여주고 고르게 하는 과정"이 아니라 에이전트 재량 추출에 가깝다. 그래서 CartPole PPO, IELTS 같은 세션이 조용히 빠졌다.
3. `session normalize`가 사람이 읽을 대화 문서가 아니라 raw JSONL 덤프에 가까운 Markdown을 만든다.

개선 방향은 `sync -> normalize -> note new`를 그대로 노출하는 것이 아니라, `configure -> inventory -> readable normalize -> extraction review -> note creation -> audit report`로 묶는 것이다.

다만 2026-06-10 추가 검토 결과, 이 문서의 전체 계획을 한 번에 구현하는 것은 과하다. 지금 필요한 것은 full intake product가 아니라, 사용자가 실제로 불편을 느낀 직접 원인 세 가지를 먼저 닫는 축소판이다.

현재 구현 Go 범위:

1. Codex JSONL을 사람이 읽을 수 있는 turn-level Markdown으로 normalize한다.
2. normalized session별 추출 상태를 보여주는 inventory/coverage report를 제공한다.
3. Thought Castle skill이 모든 normalized session을 먼저 inventory하고, 추출/스킵/보류 이유를 보고하도록 한다.

보류 범위:

- `thought-castle intake` 상위 명령
- `thought-castle extract` 의미 추출 명령
- vault language preference 파일과 interactive language prompt
- full audit command
- BagIt/manifest/stable hash 개선

이 항목들은 방향은 맞지만, readable normalize와 inventory를 먼저 사용해본 뒤에 결정한다.

## 1.1 추가 검토 반영

사용자 요청에 따라 subagent 2개와 웹 근거를 사용해 이 계획이 실제로 필요한지 재검토했다.

제품/UX 관점 subagent 결론:

- `Readable Codex normalize`는 지금 필요하다. 현재 `01_sessions`는 raw JSONL 덤프에 가깝고 README가 약속하는 normalized Markdown UX와 맞지 않는다.
- `session inventory / coverage report`도 필요하다. CartPole PPO, IELTS가 빠진 문제는 추출 품질보다 "무엇이 빠졌는지 보이지 않는 문제"가 더 크다.
- skill 문서에는 "모든 normalized session을 먼저 inventory하고 skip reason을 남긴다"를 즉시 넣어야 한다.
- `intake`, `extract`, language preference, manifest/hash는 지금 P0가 아니다.

구현/TDD 관점 subagent 결론:

- `Readable Codex normalize`는 구현 복잡도가 크지만, 가장 작은 TDD slice로 가치를 검증할 수 있다.
- `language preference`는 중요하지만 템플릿/normalize/CLI flag까지 번져 첫 PR 범위로는 크다.
- `extract`는 CLI 단독으로 의미 추출 품질을 검증하기 어려우므로 readable normalize와 inventory 뒤로 미룬다.
- 권장 순서는 `Readable Codex normalize 최소판 -> inventory/coverage -> skill 규칙 보강 -> language preference 최소판`이다.

웹 근거 해석:

- RFC 8493 BagIt은 raw payload와 manifest/metadata 분리를 지지한다. 즉 raw JSONL은 보존하고, 사람이 읽는 Markdown은 별도 표현으로 만드는 것이 맞다.
- Obsidian block links는 `#^id` 참조를 지원한다. Thought Castle의 `source_refs`가 가치 있으려면 모든 노트가 `#^t0001` 같은 coarse anchor만 가리키면 안 된다.
- Evergreen note 원칙은 파생 note를 atomic하게 만들라는 장기 방향을 지지하지만, 지금 당장 필요한 구현은 추출 엔진보다 readable session과 coverage report다.

## 2. 이번 사용에서 드러난 문제

### 2.1 언어 선택 부재

사용자는 한국어 대화를 했기 때문에 한국어 결과를 기대했다. 하지만 현재 템플릿, README, skill 지침이 영어 중심이고, vault나 skill에 "preferred language" 설정이 없다.

결과적으로 에이전트는 다음 신호를 더 강하게 받아들였다.

- 템플릿 섹션 제목이 영어다.
- `skills/thought-castle/SKILL.md`의 운영 규칙이 영어다.
- normalized session 안에는 코딩 로그, 영어 명령, 영어 final answer가 많이 섞여 있다.

따라서 "대화 언어를 따라간다"는 암묵 기대를 제품 계약으로 바꿔야 한다.

### 2.2 CartPole PPO와 IELTS 세션 누락

내가 앞 단계에서 CartPole PPO와 IELTS를 knowledge로 추출하지 않은 이유는 다음과 같다.

- 당시 목표를 "Thought Castle 자체 운영 개선"으로 좁혀 해석했다.
- skill의 `Extraction Rules`가 "fewer high-signal notes"를 강조하지만, "정규화한 모든 세션을 inventory하고 추출/스킵 이유를 보고하라"는 강제 규칙은 없다.
- 현재 CLI에는 `extract all`, `coverage report`, `topic routing`이 없다.
- `note new`는 저수준 scaffold 명령이라 실제 내용 추출 기준을 CLI가 보장하지 않는다.

이 판단은 사용자 기대와 달랐다. 사용자는 `01_sessions`에 들어간 세션이라면 CartPole PPO, IELTS, Pi Harness, Thought Castle 운영 세션이 모두 후보로 보이고, 어떤 것은 knowledge/thought/idea로 만들어지거나 최소한 "왜 스킵했는지" 보고되기를 기대했다.

### 2.3 normalize 결과가 너무 raw함

현재 `session normalize`는 provider별 turn parser가 없어서 raw 전문을 `### t0001 source ^t0001` 하나에 붙인다. 이 방식은 보존에는 안전하지만, 사용자가 읽고 판단하기에는 좋지 않다.

특히 Codex JSONL은 base instructions, tool call, encrypted reasoning, event logs, token counts가 섞여 있다. 사용자는 대부분 다음만 보고 싶다.

- user message
- assistant final answer
- 중요한 tool 결과 요약
- 사람이 확인할 필요가 있는 결정/오류/검증 결과

raw는 반드시 보존하되, `01_sessions`는 "읽을 수 있는 대화 문서"가 되어야 한다.

## 3. 웹 조사에서 얻은 설계 원칙

### 3.1 원본 보존과 사람이 읽는 표현을 분리한다

BagIt RFC 8493은 payload 파일을 opaque content로 보고, manifest와 checksum으로 보존/이동 신뢰성을 만든다. Thought Castle도 같은 방향이 맞다. raw JSONL은 `00_raw-sessions`에 원형 보존하고, 읽기 좋은 Markdown은 `01_sessions`에 별도 생성한다.

적용:

- raw 원문은 절대 요약본으로 대체하지 않는다.
- normalize 결과에는 "필터링됨 / 보존 위치 / raw hash / dropped event count"를 명시한다.
- 장기적으로 `manifest-sha256.txt` 또는 vault-local manifest를 둔다.

근거:

- RFC 8493 BagIt: https://www.rfc-editor.org/rfc/rfc8493.html

### 3.2 block link는 Thought Castle의 핵심 UX다

Obsidian은 note link, heading link, block link를 지원하고, block id는 `#^id` 형태로 링크된다. Thought Castle의 `source_refs`는 이 모델과 잘 맞지만, 지금처럼 모든 것이 `#^t0001`이면 검증성이 약하다.

적용:

- `### t0001 user ^t0001`, `### t0002 assistant ^t0002`처럼 turn-level anchor를 만든다.
- derived note는 가능하면 coarse `t0001 source`가 아니라 실제 user/assistant turn을 가리킨다.
- Obsidian properties는 작고 기계가 읽을 수 있는 값만 담고, 긴 설명은 본문에 둔다.

근거:

- Obsidian internal links/block links: https://help.obsidian.md/links
- Obsidian properties/YAML: https://help.obsidian.md/properties

### 3.3 파생 note는 atomic해야 한다

Andy Matuschak의 evergreen note 원칙은 한 노트가 "한 가지"를 다루되 너무 잘게 찢지 않는 균형을 강조한다. Thought Castle에서는 이 원칙이 `knowledge`, `thought`, `idea` 분리에 잘 맞는다.

적용:

- 세션 전체 요약을 knowledge로 만들지 않는다.
- 한 claim, 한 판단, 한 실험 아이디어를 한 노트로 만든다.
- 단, session-level summary는 `01_sessions` 안에 남겨 탐색성을 높인다.

근거:

- Evergreen notes should be atomic: https://notes.andymatuschak.org/Evergreen_notes_should_be_atomic

### 3.4 개인 정보 관리는 re-finding 비용을 줄여야 한다

Personal Information Management 연구에서는 정보의 저장보다 다시 찾기와 유지보수가 실제 문제다. Thought Castle도 "많이 넣기"보다 "다시 찾고 검증할 수 있게 넣기"가 중요하다.

적용:

- session inventory에 cwd, provider, title, topic, language, extraction status를 기록한다.
- "세션은 있는데 파생 노트가 없음"을 audit에서 보여준다.
- 스킵한 세션도 스킵 이유를 남긴다.

근거:

- Personal information management 개요: https://en.wikipedia.org/wiki/Personal_information_management

### 3.5 보존 시스템은 환경 변화 감시가 필요하다

Digital Preservation Coalition의 preservation planning 설명은 저장 포맷, 도구, 접근 방식, 사용자 기대가 계속 변하기 때문에 기술 감시와 risk trigger가 필요하다고 설명한다. LLM provider JSONL 형식은 안정 API가 아니므로 Thought Castle도 이 관점이 필요하다.

적용:

- provider parser에 schema version과 fallback path를 둔다.
- parser가 모르는 event를 버릴 때 dropped count와 raw link를 남긴다.
- `sync`와 `normalize`는 "성공"뿐 아니라 "부분 파싱 / fallback"도 보고한다.

근거:

- DPC Preservation Planning: https://www.dpconline.org/handbook/organisational-activities/preservation-planning

## 4. Subagent 리뷰 요약

별도 subagent는 파일을 수정하지 않고 UX/product-risk 관점으로 검토했다.

핵심 지적은 다음과 같다.

- 언어 설정이 제품 계약에 없다. `--language`, vault locale, 대화 언어 추론 규칙이 필요하다.
- `session normalize`가 provider parser 없이 raw 전체를 `t0001 source` 하나로 붙인다.
- README는 agent intake를 말하지만 실제 CLI는 저수준 명령 조합만 제공한다.
- 로컬 vault에는 CartPole PPO/IELTS session은 있지만 파생 지식 노트가 없다.
- `thought-castle intake <vault>` 같은 상위 명령이 필요하다.
- skill은 `validate -> language detect -> source inventory -> topic shortlist -> normalize -> extract -> user confirmation` 라우터가 되어야 한다.

이 리뷰는 현재 사용자 피드백과 일치한다.

## 5. 제안하는 사용자 경험

### 5.1 설치/초기화 시 언어 설정

권장 UX:

```bash
thought-castle init ~/thought-castle --language ko
```

또는 interactive TTY에서는:

```text
Preferred archive language?
1. 한국어 (ko)
2. English (en)
3. Auto-detect from user messages (auto)
```

저장 위치:

```text
_system/preferences.toml
```

예시:

```toml
language = "ko"
fallback_language = "auto"
note_title_language = "ko"
preserve_source_language = true
```

규칙:

- `--language <ko|en|auto>` CLI flag가 가장 우선한다.
- vault preference가 그다음이다.
- 둘 다 없으면 `auto`다.
- `auto`는 최근 user message 언어를 우선한다.
- source가 영어 학습 자료이거나 코드라면 원문은 보존하되 Summary/Context/노트 설명은 preferred language로 작성한다.

### 5.2 `intake` 상위 명령 추가

사용자는 다음처럼 쓰는 것이 목표다.

```bash
thought-castle intake ~/thought-castle --provider codex --language ko
```

동작:

1. `validate`
2. provider source list
3. 신규/변경 raw sync
4. normalize candidate 생성
5. session inventory 표시
6. topic/language/category 추론
7. 사용자에게 추출 후보 preview
8. 승인된 후보만 `knowledge`, `thought`, `idea` 생성
9. coverage report 출력

처음에는 완전 자동보다 `--review` 기본값이 안전하다.

```bash
thought-castle intake ~/thought-castle --provider codex --language ko --review
```

### 5.3 `session render` 또는 readable normalize 추가

현재 `session normalize`는 raw 보존용에 가깝다. 아래 중 하나가 필요하다.

옵션 A: `session normalize` 자체를 readable default로 바꾼다.

```bash
thought-castle session normalize ~/thought-castle <raw-file> \
  --provider codex \
  --title "CartPole PPO Line By Line Study" \
  --language ko
```

옵션 B: raw-preserving normalize와 readable render를 분리한다.

```bash
thought-castle session normalize ~/thought-castle <raw-file> --mode raw
thought-castle session render ~/thought-castle <session> --mode conversation --language ko
```

권장은 옵션 A다. raw는 이미 `00_raw-sessions`에 보존되므로 `01_sessions`는 기본적으로 읽을 수 있어야 한다.

출력 구조:

```markdown
---
type: session
status: normalized
source_type: ai_conversation
provider: codex
language: ko
raw_file: ...
raw_hash: sha256:...
events_total: 142
events_rendered: 31
events_omitted: 111
---

# CartPole PPO Line By Line Study

## Summary

이 세션은 CartPole PPO 구현을 한 줄씩 학습하기 위해...

## Extractable Topics

- PPO rollout collection
- GAE 계산
- policy/value loss

## Conversation

### t0001 user ^t0001

...

### t0002 assistant ^t0002

...

## Omitted Events

- system/base instructions: 3
- tool call logs: 18
- encrypted reasoning: 22
- token count events: 8
```

### 5.4 추출 coverage report

모든 normalized session에 대해 아래 상태를 보여줘야 한다.

```text
Session                                      Status
Thought Castle Audit Remediation             extracted: 7 notes
CartPole PPO Line By Line Study               pending extraction
CartPole PPO Learning Roadmap                 pending extraction
IELTS Speaking Archive Build                  pending extraction
Pi Harness Installable Package                skipped: implementation evidence, no user approval
```

이 report가 있어야 사용자가 "왜 내 중요한 세션이 knowledge로 안 들어갔지?"를 바로 알 수 있다.

### 5.5 멀티 스킬 라우팅

Thought Castle skill은 모든 도메인 추출을 직접 잘하려고 하면 안 된다. 대신 archive coordinator가 되어야 한다.

라우팅 예:

- CartPole PPO / RL / Python 학습 세션
  - `python-development:*`, RL 관련 로컬 학습 규칙, 코드 학습 스타일을 사용
  - Thought Castle은 source trace와 note 상태만 책임
- IELTS 세션
  - study/archive workflow로 라우팅
  - speaking practice material은 `knowledge`보다 `thought` 또는 별도 study note로 분류할 수 있음
- Thought Castle repo 운영 세션
  - repo/workflow/release skill로 라우팅
  - release/audit/idea notes 생성

skill 문구 변경:

```text
Before extracting, build a session inventory. For each normalized session,
classify domain, language, and extraction route. Do not silently skip sessions.
When a domain-specific skill is available, use it for interpretation and use
Thought Castle only for source trace, note status, and vault placement.
```

## 6. 구현 로드맵

### PR 1: Readable Codex Normalize 최소판

목표: Codex JSONL을 사람이 읽을 수 있는 user/assistant turn Markdown으로 변환한다.

RED:

- `session normalize`에 `--provider codex`를 허용한다.
- Codex JSONL fixture에서 `### t0001 user ^t0001`, `### t0002 assistant ^t0002`를 만든다.
- raw JSONL 전체 덤프와 system/tool/token count event는 본문에 들어가지 않는다.
- `events_total`, `events_rendered`, `events_omitted` metadata를 기록한다.

GREEN:

- provider가 `codex`일 때 line-delimited JSON에서 user/assistant message text를 best-effort로 추출한다.
- 알 수 없는 provider 또는 parser 실패 시 기존 `t0001 source` fallback을 유지한다.
- raw source는 계속 `00_raw-sessions`에 보존한다.

Acceptance:

- `01_sessions`를 열었을 때 raw JSONL 덤프가 아니라 대화처럼 읽힌다.
- derived note가 coarse `t0001 source`보다 구체적인 turn anchor를 가리킬 기반이 생긴다.

### PR 2: Session Inventory / Coverage Report

목표: normalized session별 추출 상태와 누락을 사용자가 볼 수 있게 한다.

RED:

- `thought-castle inventory <lab>`이 `01_sessions` 파일 목록과 각 파일의 derived note count를 출력한다.
- 파생 노트가 없는 session은 `pending extraction`으로 표시한다.
- `#^t0001` coarse source refs만 있는 경우 `coarse trace` 경고를 표시한다.

GREEN:

- `01_sessions`, `10_knowledge`, `20_thoughts`, `30_ideas`를 스캔한다.
- derived note의 `source_refs.session` 문자열을 기준으로 session별 count를 집계한다.
- 우선 사람용 text table만 구현한다.

Acceptance:

- CartPole PPO와 IELTS 세션이 조용히 빠지지 않고 `pending extraction`으로 보인다.

### PR 3: Archive Intake Skill Router 보강

목표: 구현 전/후 모두 agent가 세션을 조용히 누락하지 않게 한다.

변경:

- archive intake 시작 시 `inventory` 또는 수동 session listing을 먼저 수행한다.
- 각 session에 대해 `extract`, `skip`, `defer` 중 하나와 이유를 보고한다.
- 사용자 요청 언어를 우선하고, 없으면 최근 user message 언어를 따른다.
- CartPole PPO, IELTS 같은 도메인 세션은 Thought Castle 운영 세션으로만 해석하지 말고 도메인별 관점으로 읽는다.

Acceptance:

- 에이전트가 임의로 Thought Castle 운영 세션만 골라 추출하지 않는다.
- 스킵한 세션은 반드시 이유가 남는다.

### PR 4: Vault Language Preference 최소판

목표: 사용자가 vault 단위 선호 언어를 설정할 수 있게 한다. 단, 첫 구현에서는 템플릿 전체 번역까지 요구하지 않는다.

RED:

- `thought-castle init <lab> --language ko`가 `_system/preferences.toml`을 만든다는 테스트
- `thought-castle init <lab> --language xx`가 실패한다는 테스트
- README/SKILL quickstart가 language 설정을 안내한다는 테스트

GREEN:

- `--language ko|en|auto` 추가
- `_system/preferences.toml` 생성
- skill workflow가 preferences를 읽도록 문서화

Acceptance:

- archive intake agent가 vault preference를 근거로 한국어 결과를 우선 생성한다.

### 보류 PR A: Extract Command

목표: `note new`보다 높은 수준의 추출 명령을 제공한다.

보류 이유:

- CLI 단독으로 CartPole PPO/IELTS 의미 추출 품질을 검증하기 어렵다.
- readable normalize와 inventory가 먼저 있어야 추출 대상을 안정적으로 고를 수 있다.

### 보류 PR B: Intake Command

목표: provider sync, normalize, inventory, extraction review를 한 명령으로 묶는다.

보류 이유:

- 하위 primitive의 UX가 검증되기 전 상위 명령을 만들면 책임 경계가 흔들린다.

### 보류 PR C: Full Audit / Manifest / Stable Hash

목표: source_refs, evidence gate, raw manifest, stable hash를 검증한다.

보류 이유:

- 장기 보존 관점에서는 맞지만, 현재 사용성 문제의 직접 원인은 아니다.

## 6.1 원래 제안: Vault Language Preference

목표: 사용자가 vault 단위 선호 언어를 설정할 수 있게 한다.

RED:

- `thought-castle init <lab> --language ko`가 `_system/preferences.toml`을 만든다는 테스트
- `thought-castle init <lab> --language xx`가 실패한다는 테스트
- README/SKILL quickstart가 language 설정을 안내한다는 테스트

GREEN:

- `--language ko|en|auto` 추가
- `_system/preferences.toml` 생성
- 템플릿의 설명/placeholder를 language-aware하게 만들 기반 추가

Acceptance:

- 한국어 vault에서 새 note/session의 Summary/Context/TODO가 한국어로 생성된다.

### 원래 제안: Readable Codex Normalize

목표: Codex JSONL을 사람이 읽을 수 있는 user/assistant turn 문서로 변환한다.

RED:

- Codex JSONL fixture에서 `### t0001 user ^t0001` 생성
- assistant final answer가 `### t0002 assistant ^t0002`로 생성
- base instructions/encrypted reasoning/token count는 본문에서 제외되고 omitted count에 잡힘

GREEN:

- Codex JSONL parser 추가
- raw fallback 유지
- `events_total`, `events_rendered`, `events_omitted` metadata 추가

Acceptance:

- `01_sessions`를 Obsidian에서 열었을 때 raw JSONL 덤프가 아니라 대화처럼 읽힌다.

### 원래 제안: Session Inventory And Coverage Report

목표: normalize/extract 대상과 누락을 보여준다.

RED:

- `thought-castle inventory <lab>`이 sessions와 derived note count를 출력한다는 테스트
- session은 있는데 notes가 없으면 `pending extraction`으로 표시
- coarse `#^t0001` source refs만 있는 경우 `coarse trace` 경고

GREEN:

- session metadata scan
- derived note source_refs scan
- human-readable table 출력

Acceptance:

- CartPole PPO, IELTS 세션이 "pending extraction"으로 보인다.

### 원래 제안: Extract Command

목표: `note new`보다 높은 수준의 추출 명령을 제공한다.

예시:

```bash
thought-castle extract ~/thought-castle \
  01_sessions/cartpole-ppo-line-by-line-study.md \
  --kinds knowledge,thought \
  --language ko \
  --review
```

RED:

- session에서 최소 1개 note draft를 만든다.
- 모든 note에 `source_refs`가 들어간다.
- knowledge는 `candidate`, thought는 `draft`, idea는 `raw`로 생성된다.

GREEN:

- 초기 버전은 LLM이 아니라 rule/template 기반으로 "extraction packet"을 만든다.
- agent skill이 이 packet을 읽고 실제 note content를 채우도록 한다.

Acceptance:

- CartPole PPO 세션에서 PPO/GAE/rollout 관련 candidate가 생긴다.
- IELTS 세션에서 speaking archive/practice workflow 관련 thought/knowledge draft가 생긴다.

### 원래 제안: Archive Intake Skill Router

목표: skill이 사용자 대신 전체 intake loop를 안정적으로 수행하게 한다.

변경:

- 시작 시 vault language 확인
- `inventory`를 먼저 실행
- 모든 session에 대해 extract/skip decision을 report
- domain-specific skill routing 지침 추가
- 마지막에 "created / skipped / needs user confirmation" 보고

Acceptance:

- 에이전트가 임의로 Thought Castle 운영 세션만 고르지 않는다.
- 스킵한 세션은 반드시 이유가 남는다.

### 원래 제안: Audit Command

목표: archive 품질을 사용자가 직접 확인할 수 있게 한다.

검사:

- derived note에 `source_refs`가 있는가
- session file이 존재하는가
- block id가 존재하는가
- raw_file이 존재하는가
- `verified`인데 evidence가 비어 있지 않은가
- `stable` thought인데 `user_confirmed`가 true인가
- normalized session 중 파생 note가 없는 것이 몇 개인가

Acceptance:

- `thought-castle audit ~/thought-castle`가 사용자에게 다음 행동을 알려준다.

## 7. 우선순위

수정된 우선순위:

1. P0: Readable Codex normalize 최소판
2. P0: Session inventory / coverage report
3. P0: Archive intake skill router 보강
4. P1: Vault language preference 최소판
5. P2: Extract command
6. P2: Intake command
7. P2: Audit command
8. P3: Stable manifest/hash improvements

이 순서가 맞는 이유는 간단하다. 지금 사용자가 바로 불편함을 느낀 지점은 "읽을 수 없음"과 "조용히 누락됨"이다. 언어도 중요하지만, readable session과 coverage가 없으면 language preference를 넣어도 체감 가치가 약하다. hash/audit은 중요하지만, 먼저 archive intake 경험이 신뢰 가능해야 한다.

## 8. 열린 질문

1. `language` 설정은 `init`에서 interactive prompt를 띄울 것인가, 아니면 flag만 둘 것인가?
2. `skill install --language ko`가 한국어 skill copy를 설치해야 하는가, 아니면 vault preference만 따르게 할 것인가?
3. IELTS 같은 학습 자료는 `10_knowledge`에 넣을지, 별도 `40_study` 같은 영역이 필요한가?
4. `extract`를 CLI가 직접 LLM 없이 할 수 있는 범위는 어디까지인가?
5. multi-skill routing은 Thought Castle skill 문서만으로 충분한가, 아니면 별도 orchestrator skill이 필요한가?

## 9. 즉시 적용 가능한 운영 규칙

구현 전까지는 agent workflow를 다음처럼 운영한다.

1. 정규화된 모든 세션 목록을 먼저 보여준다.
2. 사용자가 원하는 언어를 확인한다. 기본은 한국어다.
3. 각 세션에 대해 `extract`, `skip`, `defer` 중 하나를 표시한다.
4. CartPole PPO, IELTS처럼 도메인이 다른 세션은 도메인별 관점으로 읽는다.
5. 모든 생성 노트는 `candidate`/`draft`/`raw` 상태로 둔다.
6. 마지막 보고서에 생성한 노트와 스킵한 세션을 함께 쓴다.

## 10. 참고 링크

- RFC 8493 BagIt File Packaging Format: https://www.rfc-editor.org/rfc/rfc8493.html
- Obsidian internal links and block links: https://help.obsidian.md/links
- Obsidian properties: https://help.obsidian.md/properties
- Evergreen notes should be atomic: https://notes.andymatuschak.org/Evergreen_notes_should_be_atomic
- Digital Preservation Coalition, Preservation Planning: https://www.dpconline.org/handbook/organisational-activities/preservation-planning
- Personal information management overview: https://en.wikipedia.org/wiki/Personal_information_management
