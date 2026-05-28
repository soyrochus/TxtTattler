# Spec 003: Deterministic Language Support via `gpt-4o-mini-tts`

## Status

Draft

## 1. Purpose

Allow users to control the spoken language and voice style of the generated audio explicitly, rather than relying on implicit language detection from the input text.

OpenAI's `gpt-4o-mini-tts` model exposes an `instructions` field that accepts free-text natural-language guidance — including language, accent, and tone — independently of the content of the text being read. This is the only currently available mechanism for deterministic language control in the OpenAI TTS API. The older `tts-1` and `tts-1-hd` models have no equivalent.

This spec covers:

1. Adding `gpt-4o-mini-tts` as a supported model in TxtTattler.
2. Exposing the `--instructions` CLI option to pass language and style guidance to the model.
3. Exposing the seven voices already present in `async-openai` but not yet wired up in TxtTattler.
4. Handling model-specific constraints (speed, voice compatibility, instructions availability).
5. Keeping the cache key correct so different language instructions always produce different cached audio.

## 2. Background

### Why the existing models are not sufficient

`tts-1` and `tts-1-hd` detect language from the input text and render accordingly. If the text is in Spanish, the output is in Spanish. If the text is in English but the user wants a Dutch accent or explicit British English, there is no API mechanism to express that. The API silently ignores any `instructions` field sent to these models.

### What `gpt-4o-mini-tts` provides

`gpt-4o-mini-tts` accepts an `instructions` field in the TTS request. Typical usage:

```text
Speak in Brazilian Portuguese with a warm, conversational tone.
Speak in Dutch with a standard Amsterdam accent.
Speak in English with a British RP accent.
```

The model honors these instructions independently of what the text says. A Dutch `instructions` value causes the model to read English text in Dutch-accented English, or a Dutch translation of the text if the instructions say to translate — though translation is not the purpose of this feature.

This is the mechanism this spec targets.

### Crate readiness

`async-openai` version 0.40.2 (already in `Cargo.toml`) provides all required types with no version bump needed:

| Type | Status |
|---|---|
| `SpeechModel::Gpt4oMiniTts` | Present |
| `Voice::Ash`, `Ballad`, `Coral`, `Sage`, `Verse`, `Marin`, `Cedar` | Present |
| `CreateSpeechRequest::instructions: Option<String>` | Present |
| `CreateSpeechRequest::speed: Option<f32>` | Already optional |

## 3. Scope

### In scope

- Add `gpt-4o-mini-tts` to the `SpeechModelName` domain entity and all dependent layers.
- Expose the seven built-in voices already in `async-openai` (`ash`, `ballad`, `coral`, `sage`, `verse`, `marin`, `cedar`) by adding them to TxtTattler's `VoiceName` domain entity and all dependent layers.
- Add `--instructions <TEXT>` to the CLI.
- Thread `instructions` through config resolution, `TtsRequest`, and the OpenAI adapter.
- Include `instructions` in the cache key.
- Handle the speed constraint: `gpt-4o-mini-tts` does not reliably support `--speed` values other than `1.0`; the app must warn when speed is set with this model.
- Handle the instructions constraint: `instructions` is silently ignored by `tts-1` and `tts-1-hd`; the app must warn when `--instructions` is set with those models.
- Update `--list-voices` to show the new voices with descriptions.
- Update help text to explain the model and instructions options.

### Out of scope

- Streaming TTS responses (a separate concern).
- Automatic language detection from the input text.
- Translation of the input text.
- Custom voice creation (`CustomVoiceRef`).
- Any audio model beyond the three TTS models (`tts-1`, `tts-1-hd`, `gpt-4o-mini-tts`).
- Realtime API or audio chat endpoints.
- Model-specific voice hard-blocking at CLI parse time (see section 5.3).

## 4. Functional Requirements

### 4.1 Model support

The application shall accept `gpt-4o-mini-tts` as a value for `--model`.

```text
txttattler notes.txt --model gpt-4o-mini-tts --instructions "Speak in Dutch."
```

`gpt-4o-mini-tts` shall be:

- Accepted at parse time by the `--model` option.
- Listed when the user runs `txttattler --model --help` or inspects the generated help text.
- Accepted in the `model` field of the TOML config file and `TXT_TATTLER_MODEL` environment variable.
- Round-tripped correctly through `FromStr` / `Display`.

The existing models `tts-1` and `tts-1-hd` shall continue to work without change.

### 4.2 Voice support

The following seven voices are already present in `async-openai` 0.40.2 but are not yet exposed by TxtTattler's `VoiceName` enum, CLI parser, or `--list-voices` output. The application shall accept all seven:

| Voice | Description |
|---|---|
| `ash` | Direct and confident |
| `ballad` | Expressive and emotive |
| `coral` | Warm and conversational |
| `sage` | Calm and measured |
| `verse` | Versatile and natural |
| `marin` | Clear and bright |
| `cedar` | Rich and resonant |

These voices are supported by all three TTS models at the API level, though OpenAI documents them as optimised for `gpt-4o-mini-tts`. The application shall not block their use with `tts-1` or `tts-1-hd` at parse time; any incompatibility is surfaced by the API at runtime.

All voices shall appear in `--list-voices` output.

### 4.3 Instructions option

A new `--instructions <TEXT>` CLI option shall be added.

| Aspect | Requirement |
|---|---|
| Type | `Option<String>` |
| Default | `None` (not sent to the API) |
| Config file key | `instructions` under the top-level section |
| Environment variable | `TXT_TATTLER_INSTRUCTIONS` |
| Precedence | CLI → env → config file → `None` |

When `instructions` is `None`, the field is omitted entirely from the API request (the crate already uses `#[serde(skip_serializing_if = "Option::is_none")]`).

When `instructions` is `Some(_)` and the selected model is `tts-1` or `tts-1-hd`, the application shall emit a visible warning:

```text
⚠ Warning: --instructions has no effect with model tts-1 (or tts-1-hd). Use --model gpt-4o-mini-tts to enable language and style control.
```

The warning shall not abort the command. The text shall still be synthesized.

### 4.4 Speed constraint with `gpt-4o-mini-tts`

The `--speed` parameter is not reliably supported by `gpt-4o-mini-tts`. When the resolved speed is not `1.0` and the resolved model is `gpt-4o-mini-tts`, the application shall emit a visible warning:

```text
⚠ Warning: --speed is not supported by gpt-4o-mini-tts and will be ignored by the API.
```

The warning shall not abort the command. The speed value shall still be included in the API request as-is (the server will ignore it). The cache key shall still include the speed value so there is no ambiguity about what was requested.

### 4.5 Cache key

The `instructions` value shall be included in the cache key.

Cache key fields (null-byte separated, as per existing design):

```text
processed_text \0 voice \0 model \0 speed \0 pipeline_version \0 instructions
```

When `instructions` is `None`, the field shall contribute an empty string to the key (not the literal string `"None"`).

This guarantees that running the same text with different `--instructions` values produces different cache entries and different audio files.

### 4.6 Console output

Normal run with instructions:

```text
✓ File read: notes.txt (1 234 characters)
✓ Text processed: 1 198 characters after cleanup
  Voice:        coral
  Model:        gpt-4o-mini-tts
  Speed:        1.0
  Instructions: Speak in Dutch with a standard Amsterdam accent.
  Chunks:       1
  Cache:        miss → generating…
✓ Synthesis complete
✓ Cache written: ~/.cache/txttattler/abc123.mp3
✓ Playing audio…
```

The `Instructions:` line shall only appear when `--instructions` is set. It shall appear in the summary block alongside voice, model, and speed.

In `--verbose` mode, the instructions value shall be printed in full regardless of length.

In normal mode, instructions longer than 80 characters may be truncated with an ellipsis in the console output (but never truncated in the API request or cache key).

### 4.7 `--list-voices` update

`--list-voices` shall list all thirteen built-in voices with descriptions. The output shall visually separate the classic six voices from the seven new voices, or list them together alphabetically — the exact layout is an implementation choice, but the output must include all voices and remain exit-safe with no API call required.

Example grouping:

```text
Classic voices (all models):
  alloy   – balanced and versatile
  echo    – clear, crisp narration
  fable   – warm storytelling tone
  onyx    – deep and steady
  nova    – bright and expressive
  shimmer – soft and polished

Extended voices (gpt-4o-mini-tts recommended):
  ash     – direct and confident
  ballad  – expressive and emotive
  coral   – warm and conversational
  sage    – calm and measured
  verse   – versatile and natural
  marin   – clear and bright
  cedar   – rich and resonant
```

## 5. Architecture Requirements

### 5.1 Domain layer (`domain/entities.rs`)

Add `Gpt4oMiniTts` variant to `SpeechModelName`:

- `as_str()` returns `"gpt-4o-mini-tts"`.
- `FromStr` parses `"gpt-4o-mini-tts"` case-insensitively.
- `Display` delegates to `as_str()`.
- `SpeechModelName::ALL` includes all three models.

Add the seven variants already present in `async-openai`'s `Voice` enum but missing from TxtTattler's `VoiceName`:

- `as_str()`, `FromStr`, `Display`, and `description()` for each.
- `VoiceName::ALL` includes all thirteen voices.

Add `instructions: Option<String>` to `TtsRequest`:

- The field is `None` by default.
- The use-case passes it through to the `TtsProvider` unchanged.
- The cache key computation must include it.

`SynthesisOutcome` may optionally add `instructions: Option<String>` if the structured report should reflect what was requested.

### 5.2 Configuration layer (`config.rs`)

`FileConfig` gains:

```toml
instructions = "Speak in Dutch."
```

`ResolvedConfig` gains `instructions: Option<String>`.

Resolution order: CLI `--instructions` → `TXT_TATTLER_INSTRUCTIONS` env → `FileConfig.instructions` → `None`.

Warnings for mismatched model/instructions and model/speed combinations shall be emitted during config resolution or at the start of the use-case run, before any API call.

### 5.3 CLI layer (`cli.rs`)

Add to `Cli`:

```rust
#[arg(long, value_name = "TEXT")]
pub instructions: Option<String>,
```

The `--model` option's `PossibleValuesParser` shall include `"gpt-4o-mini-tts"`.

The `--voice` option's `PossibleValuesParser` shall include all thirteen voices.

No model-specific voice blocking shall be done at parse time. This avoids a combinatorial validation problem and mirrors how the API itself works.

### 5.4 OpenAI adapter (`infrastructure/tts/openai.rs`)

`map_model()` gains:

```rust
SpeechModelName::Gpt4oMiniTts => SpeechModel::Gpt4oMiniTts,
```

`map_voice()` gains seven new arms.

The `synthesize()` method shall conditionally set `instructions` on the request builder when `TtsRequest.instructions` is `Some(_)`.

Speed shall continue to be passed to the API request for all models. The warning about speed incompatibility is the app's responsibility, not the adapter's.

### 5.5 Cache key (`infrastructure/cache/`)

The cache key computation function shall append `\0` followed by the `instructions` value (or an empty string if `None`) to the existing key material.

Existing cache entries (without instructions) are effectively keyed as if `instructions` were an empty string. New runs with `--instructions` set will produce a different key and a different cache entry, which is the correct behavior.

### 5.6 Use-case layer (`application/text_to_speech.rs`)

No structural change required. The use-case already threads `TtsRequest` fields from `ResolvedConfig` through to the provider. Adding `instructions` follows the same pattern already used for `voice`, `model`, and `speed`.

The use-case shall emit the instruction-related warnings via the `Reporter` port (not via direct `eprintln!`), so tests can verify warning behavior without capturing stderr.

## 6. Acceptance Criteria

| ID | Criterion |
|---|---|
| AC-1 | `txttattler file.txt --model gpt-4o-mini-tts` works without `--instructions` |
| AC-2 | `txttattler file.txt --model gpt-4o-mini-tts --instructions "Speak in Dutch."` sends instructions to the API |
| AC-3 | A second run with identical arguments produces a cache hit |
| AC-4 | A run with `--instructions "Speak in English."` after a cached run with `--instructions "Speak in Dutch."` produces a cache miss |
| AC-5 | Running without `--instructions` after a cached run with `--instructions` produces a cache miss |
| AC-6 | `--instructions` with `tts-1` or `tts-1-hd` emits a warning and completes successfully |
| AC-7 | `--speed 1.5` with `gpt-4o-mini-tts` emits a warning and completes successfully |
| AC-8 | All thirteen voices are accepted by `--voice` and appear in `--list-voices` |
| AC-9 | New voices parse case-insensitively from CLI, env, and config file |
| AC-10 | `gpt-4o-mini-tts` parses case-insensitively from CLI, env, and config file |
| AC-11 | `instructions` resolves correctly: CLI overrides env, env overrides config file, config file overrides default (`None`) |
| AC-12 | The use-case layer does not import any OpenAI-specific types or emit warnings directly to stderr |
| AC-13 | `cargo fmt --check`, `cargo check`, and `cargo test` all pass |
| AC-14 | `--list-voices` exits cleanly with no FILE argument and no API call, and includes all thirteen voices |

## 7. Testing Requirements

### 7.1 Entity tests (`domain/entities.rs`)

| Test | Expected result |
|---|---|
| `gpt-4o-mini-tts` parses from string | `Ok(SpeechModelName::Gpt4oMiniTts)` |
| `GpT-4O-MiNi-TtS` parses case-insensitively | `Ok(SpeechModelName::Gpt4oMiniTts)` |
| `SpeechModelName::Gpt4oMiniTts` displays as `"gpt-4o-mini-tts"` | round-trip confirmed |
| `SpeechModelName::ALL` contains all three models | length 3 |
| Each new voice parses from its `as_str()` value | round-trip confirmed |
| `VoiceName::ALL` contains all thirteen voices | length 13 |
| Invalid model string returns error | `Err(...)` |
| Invalid voice string returns error | `Err(...)` |

### 7.2 Config tests (`config.rs`)

| Test | Expected result |
|---|---|
| `TXT_TATTLER_INSTRUCTIONS` env sets instructions | `ResolvedConfig.instructions == Some(...)` |
| CLI `--instructions` overrides env | CLI value wins |
| CLI `--instructions` overrides config file value | CLI value wins |
| Config file `instructions` used when CLI and env are absent | config value used |
| No instructions anywhere resolves to `None` | `ResolvedConfig.instructions == None` |

### 7.3 Cache key tests

| Test | Expected result |
|---|---|
| Same text, same settings, no instructions → same key | keys are equal |
| Same text, same settings, different instructions → different key | keys differ |
| Same text, same settings, instructions `None` vs `Some("")` | keys differ (empty string is not the same as absent) |
| Instructions value is not truncated in the key | full value hashed |

### 7.4 Warning tests

Using a `CapturingReporter` or a `Vec<String>` collecting warnings:

| Test | Expected result |
|---|---|
| `instructions` set with model `tts-1` | warning message emitted via reporter |
| `instructions` set with model `tts-1-hd` | warning message emitted via reporter |
| `instructions` set with `gpt-4o-mini-tts` | no warning emitted |
| speed ≠ 1.0 with `gpt-4o-mini-tts` | warning message emitted via reporter |
| speed ≠ 1.0 with `tts-1` | no warning emitted |

### 7.5 Adapter tests

Using a fake `TtsProvider` that captures the `TtsRequest`:

| Test | Expected result |
|---|---|
| `instructions` set in request → adapter receives `Some(...)` | field passed through |
| `instructions` not set → adapter receives `None` | field is `None` |

The adapter itself need not be tested against the live OpenAI API in unit or integration tests.

### 7.6 CLI tests

| Test | Expected result |
|---|---|
| `--model gpt-4o-mini-tts` is accepted at parse time | no error |
| `--voice ash` is accepted | no error |
| `--voice cedar` is accepted | no error |
| `--list-voices` exits `0` and output includes `ash` | assertion on output |
| `--list-voices` exits `0` and output includes all thirteen voice names | assertion on output |

## 8. Non-Functional Requirements

| Property | Requirement |
|---|---|
| Cross-platform | No platform-specific code introduced |
| No version bump | `async-openai` 0.40.2 already provides all required types |
| Backward compatible | Existing commands with `tts-1` or `tts-1-hd` work unchanged |
| No breaking config change | New `instructions` key is optional; existing config files remain valid |
| Testable without API | All new behavior testable with fake providers and reporters |
| No secrets logged | `instructions` may be printed in verbose output but is not a secret; API keys must never appear |

## 9. Migration Notes

No migration of existing cache entries is required. Existing cache entries were created without an `instructions` component in the key. After this change, the same command run without `--instructions` will produce a cache key where `instructions` contributes an empty string. This does not collide with any previously cached key because the null-byte separator and the empty string together produce a distinct key suffix compared to the old format (which had no trailing separator and no instructions field). In practice this means existing cache entries will not be reused after the upgrade — they become orphaned and will be re-generated on next use, which is safe and correct.

## 10. Example Usage

### Speak a text file in Dutch

```bash
txttattler notes.txt \
  --model gpt-4o-mini-tts \
  --voice coral \
  --instructions "Speak in Dutch with a warm, conversational tone."
```

### Speak in British English with a specific register

```bash
txttattler report.txt \
  --model gpt-4o-mini-tts \
  --voice ash \
  --instructions "Speak in British English RP with a formal, measured delivery."
```

### Persist instructions in the config file

```toml
# ~/.config/txttattler/txttattler.toml
model        = "gpt-4o-mini-tts"
voice        = "coral"
instructions = "Speak in Brazilian Portuguese with a friendly tone."
```

Then simply:

```bash
txttattler documento.txt
```
