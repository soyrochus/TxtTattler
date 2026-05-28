## Why

TxtTattler currently relies on implicit language detection from input text, which makes it impossible to control the spoken language, accent, or register independently of the text content. OpenAI's `gpt-4o-mini-tts` model exposes an `instructions` field that enables explicit language and style control — the only available mechanism in the OpenAI TTS API for deterministic language output.

## What Changes

- **New model**: `gpt-4o-mini-tts` is added as a third supported model alongside `tts-1` and `tts-1-hd`.
- **New CLI option**: `--instructions <TEXT>` passes free-text language and style guidance to the model (e.g. `"Speak in Dutch with a warm tone."`).
- **New voices exposed**: Seven voices already present in `async-openai` 0.40.2 but not yet wired into TxtTattler are made available: `ash`, `ballad`, `coral`, `sage`, `verse`, `marin`, `cedar`.
- **Cache key extended**: The `instructions` value is included in the cache key so different instructions always produce distinct cached audio.
- **Compatibility warnings**: The app warns (without aborting) when `--instructions` is used with `tts-1`/`tts-1-hd` (silently ignored by the API), or when `--speed` ≠ `1.0` is used with `gpt-4o-mini-tts` (not reliably supported).
- **`--list-voices` updated**: All thirteen built-in voices are listed, separated into classic and extended groups.
- **No `async-openai` version bump required**: All needed types (`SpeechModel::Gpt4oMiniTts`, `Voice::Ash` etc., `CreateSpeechRequest::instructions`) are present in the already-pinned version 0.40.2.

## Capabilities

### New Capabilities

- `gpt4o-mini-tts-model`: Support for the `gpt-4o-mini-tts` model — adding it to `SpeechModelName`, CLI, config resolution, and the OpenAI adapter.
- `instructions-option`: The `--instructions` CLI option and its full resolution chain (CLI → env → config file → `None`), including warnings for unsupported model combinations.
- `extended-voices`: Exposing the seven additional voices (`ash`, `ballad`, `coral`, `sage`, `verse`, `marin`, `cedar`) in `VoiceName`, the CLI parser, `map_voice()`, and `--list-voices`.
- `instructions-cache-key`: Extending the cache key to include the `instructions` field so language changes always invalidate the cache correctly.

### Modified Capabilities

*(none — no existing spec-level requirements are changed; existing model and voice behaviour is strictly additive)*

## Impact

| Area | Change |
|---|---|
| `txttattler/src/domain/entities.rs` | `SpeechModelName` +1 variant; `VoiceName` +7 variants; `TtsRequest` gains `instructions: Option<String>` |
| `txttattler/src/cli.rs` | `--model` parser extended; `--voice` parser extended; `--instructions` option added |
| `txttattler/src/config.rs` | `FileConfig` and `ResolvedConfig` gain `instructions`; resolution logic and compatibility warnings added |
| `txttattler/src/infrastructure/tts/openai.rs` | `map_model()` and `map_voice()` extended; `instructions` conditionally sent in request |
| `txttattler/src/infrastructure/cache/` | Cache key computation includes `instructions` field |
| `txttattler/src/application/text_to_speech.rs` | `TtsRequest` construction threads `instructions` from config |
| Dependencies | None — `async-openai` 0.40.2 already provides all required types |
| Existing cache entries | Orphaned after upgrade (safe; re-generated on next run) |
