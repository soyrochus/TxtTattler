## Context

TxtTattler is a hexagonal-architecture Rust CLI that synthesises speech from text files using OpenAI TTS. The domain layer defines `VoiceName`, `SpeechModelName`, and `TtsRequest` in `entities.rs`; the infrastructure layer maps these to `async-openai` types in `openai.rs`; config resolution lives in `config.rs`; and the CLI is defined in `cli.rs` using `clap` v4 derive macros.

The current codebase supports two models (`tts-1`, `tts-1-hd`) and six voices (`alloy`, `echo`, `fable`, `onyx`, `nova`, `shimmer`). The pinned dependency `async-openai` 0.40.2 already contains `SpeechModel::Gpt4oMiniTts`, all seven extended voices, and `CreateSpeechRequest::instructions: Option<String>` — so the entire OpenAI API surface needed for this change is already available at the crate level. No version bump is required.

The cache key currently concatenates processed text, voice, model, speed, and pipeline version with null-byte separators and hashes the result with SHA-256.

## Goals / Non-Goals

**Goals:**
- Add `gpt-4o-mini-tts` as a first-class model choice throughout the stack.
- Expose `--instructions` as a CLI option and thread it through config → domain → adapter without leaking OpenAI concepts into the domain layer.
- Wire up the seven extended voices already in `async-openai` into TxtTattler's `VoiceName` enum, CLI parser, adapter, and `--list-voices`.
- Extend the cache key to include `instructions` so different language settings always produce distinct cache entries.
- Emit actionable warnings for known API limitations (instructions ignored by `tts-1`/`tts-1-hd`; speed unreliable with `gpt-4o-mini-tts`) without aborting execution.

**Non-Goals:**
- Automatic language detection from input text.
- Translation of text to another language.
- Custom voice creation or `CustomVoiceRef` support.
- Streaming TTS (`create_stream`).
- Any model beyond the three TTS models now supported.
- Blocking invalid model/voice combinations at parse time (left to the API).

## Decisions

### D1 — `instructions` lives on `TtsRequest`, not passed out-of-band

**Decision:** Add `instructions: Option<String>` directly to `TtsRequest` in the domain layer.

**Rationale:** `TtsRequest` is already the bundle of all synthesis parameters passed from the use-case to the `TtsProvider` port. Adding `instructions` there keeps the port interface self-contained and avoids introducing a second parameter or a side-channel. The domain type remains independent of OpenAI — `instructions` is just a `String`, not an OpenAI type.

**Alternative considered:** A separate `SynthesisHints` struct wrapping optional parameters. Rejected as premature abstraction — there is currently only one hint.

---

### D2 — Warnings emitted via `Reporter` port, not `eprintln!`

**Decision:** Compatibility warnings (instructions-with-old-model, speed-with-new-model) are issued through the existing `Reporter` port, not printed directly to stderr.

**Rationale:** The use-case layer must not own console output — this is an existing architectural constraint enforced by the `Reporter` abstraction. Emitting warnings through the reporter keeps the use-case testable without capturing stderr and consistent with how all other console output is handled.

**Alternative considered:** Emit warnings in `config.rs` during resolution (before the use-case runs). Rejected because the use-case is a better home — it has full resolved context and mirrors how other pre-synthesis validations work.

---

### D3 — No model-specific voice blocking at parse time

**Decision:** All thirteen voices are accepted for all models by the CLI parser. Invalid combinations are surfaced by the API at runtime.

**Rationale:** Voice-model compatibility is a moving target on the OpenAI side. Hard-coding a compile-time matrix creates maintenance burden and breaks valid future combinations. The API's error message is clear enough. The `--list-voices` output documents which voices are recommended for which models.

**Alternative considered:** A `valid_for_model()` predicate on `VoiceName` checked after config resolution. Rejected — a runtime check that produces a hard error is worse UX than an API error for an unusual case.

---

### D4 — `instructions` in cache key as empty string when `None`

**Decision:** When `instructions` is `None`, the cache key contribution is an empty string (not the literal `"None"`). The field is always appended with a null-byte separator regardless.

**Rationale:** Using a fixed sentinel like `"None"` risks collision with a user who literally passes the string `"None"` as instructions. An empty string is unambiguous because the API would never be called with an empty instructions string (the field is omitted when `None`). The null-byte separator ensures the empty field cannot merge with adjacent fields.

**Implication:** Existing cache entries (keyed without the trailing `\0` and instructions field) will never match new keys, so they are effectively orphaned. They are safe to leave in place — they will be regenerated on next use.

---

### D5 — Speed value still sent to API for `gpt-4o-mini-tts`

**Decision:** When speed ≠ 1.0 with `gpt-4o-mini-tts`, a warning is emitted but the speed value is still included in the API request.

**Rationale:** The API behaviour with speed on `gpt-4o-mini-tts` is "undefined but not an error". Silently dropping it would mean the cached audio was generated under one set of parameters the user requested but the cache key reflects different parameters. Sending it maintains parameter honesty; if OpenAI later supports speed on this model, the behaviour improves automatically.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| OpenAI changes which voices work with which models | No compile-time blocking (D3); model/voice errors surface as clear API errors |
| `gpt-4o-mini-tts` speed behaviour changes | Warning keeps users informed; sending the value means behaviour improves if OpenAI adds support |
| Orphaned cache entries after upgrade | Safe — re-generated on next run; old files are inert and will be cleaned by standard cache eviction if implemented |
| Instructions string in console output leaks sensitive content | Instructions are not credentials; standard truncation in normal mode (80 chars) limits noise without hiding content |
| `instructions` description strings invented for new voices are inaccurate | Descriptions should be verified against OpenAI's platform documentation before shipping |

## Migration Plan

1. No database or schema migrations required.
2. No breaking CLI changes — all new options are additive with defaults of `None`.
3. Existing config files remain valid — new `instructions` key is optional.
4. Existing cache entries are orphaned (see D4). No cleanup needed; they are re-synthesised on next run.
5. Deploy is a standard `cargo build --release` drop-in replacement.
6. Rollback: revert to previous binary. Old cache entries (without instructions key) will be reused correctly by the old binary.

## Open Questions

- **Voice descriptions**: The descriptions for the seven extended voices (`ash`, `ballad`, etc.) used in `--list-voices` are provisional. They should be verified against OpenAI's official documentation before the feature ships.
- **`gpt-4o-mini-tts` speed support**: OpenAI's documentation is ambiguous about whether speed is silently ignored or causes an API error. This should be tested against the live API during implementation and the warning text updated accordingly.
