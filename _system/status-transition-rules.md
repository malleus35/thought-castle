# Status Transition Rules

## Knowledge

```text
candidate -> needs_verification -> verified
candidate -> disputed
candidate -> discarded
```

`verified`는 근거가 있을 때만 허용한다.

## Thought

```text
draft -> reviewing -> stable
draft -> discarded
```

`stable`은 사용자가 자신의 생각으로 승인했을 때만 허용한다.

## Idea

```text
raw -> reviewing -> experimenting -> validated
raw -> discarded
```

`validated`는 작은 실험이나 검토를 통과했을 때만 허용한다.
