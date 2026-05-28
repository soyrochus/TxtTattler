## 1. Domain Entities

- [x] 1.1 Add `Gpt4oMiniTts` variant to `SpeechModelName` in `txttattler/src/domain/entities.rs` with `as_str()`, `FromStr`, and `Display` implementations
- [x] 1.2 Add `Gpt4oMiniTts` to `SpeechModelName::ALL` and verify array length is 3
- [x] 1.3 Add seven new variants to `VoiceName` (`Ash`, `Ballad`, `Coral`, `Sage`, `Verse`, `Marin`, `Cedar`) with `as_str()`, `FromStr`, `Display`, and `description()` implementations
- [x] 1.4 Add all seven new variants to `VoiceName::ALL` and verify array length is 13
- [x] 1.5 Add `instructions: Option<String>` field to `TtsRequest`

## 2. CLI

- [x] 2.1 Add `"gpt-4o-mini-tts"` to the `PossibleValuesParser` for `--model` in `txttattler/src/cli.rs`
- [x] 2.2 Add all seven new voice strings to the `PossibleValuesParser` for `--voice`
- [x] 2.3 Add `--instructions <TEXT>` option (`Option<String>`) to the `Cli` struct
- [x] 2.4 Update `print_available_voices()` to list all thirteen voices, separating classic and extended groups

## 3. Configuration

- [x] 3.1 Add `instructions: Option<String>` field to `FileConfig` in `txttattler/src/config.rs`
- [x] 3.2 Add `instructions: Option<String>` field to `ResolvedConfig`
- [x] 3.3 Implement resolution logic for `instructions`: CLI → `TXT_TATTLER_INSTRUCTIONS` env → `FileConfig.instructions` → `None`
- [x] 3.4 Add compatibility warning (via `Reporter`) when `instructions` is `Some(_)` and model is `tts-1` or `tts-1-hd`
- [x] 3.5 Add compatibility warning (via `Reporter`) when speed ≠ `1.0` and model is `gpt-4o-mini-tts`

## 4. OpenAI Adapter

- [x] 4.1 Add `SpeechModelName::Gpt4oMiniTts => SpeechModel::Gpt4oMiniTts` arm to `map_model()` in `txttattler/src/infrastructure/tts/openai.rs`
- [x] 4.2 Add seven new arms to `map_voice()` mapping each new `VoiceName` variant to its `Voice` counterpart
- [x] 4.3 Update `synthesize()` to conditionally set `.instructions(...)` on the request builder when `TtsRequest.instructions` is `Some(_)`

## 5. Cache Key

- [x] 5.1 Locate the cache key computation function in `txttattler/src/infrastructure/cache/`
- [x] 5.2 Append `\0` followed by the `instructions` value (empty string when `None`) to the key material before hashing
- [x] 5.3 Verify existing cache tests still pass (orphaned entries are expected, not a regression)

## 6. Use-Case Wiring

- [x] 6.1 Update `TtsRequest` construction in `txttattler/src/application/text_to_speech.rs` to populate `instructions` from `ResolvedConfig.instructions`
- [x] 6.2 Emit model/instructions and model/speed warnings via the `Reporter` port before the first API call

## 7. Console Output

- [x] 7.1 Add `Instructions:` line to the synthesis summary output when `instructions` is `Some(_)`
- [x] 7.2 Truncate displayed instructions to 80 characters with an ellipsis in normal mode; show full value in `--verbose` mode

## 8. Tests — Entities

- [x] 8.1 Test `SpeechModelName::from_str("gpt-4o-mini-tts")` returns `Ok(Gpt4oMiniTts)`
- [x] 8.2 Test case-insensitive parse of `"GPT-4O-MINI-TTS"`
- [x] 8.3 Test `Gpt4oMiniTts.to_string()` returns `"gpt-4o-mini-tts"`
- [x] 8.4 Test `SpeechModelName::ALL.len() == 3`
- [x] 8.5 Test each new voice parses from its `as_str()` value (round-trip)
- [x] 8.6 Test `VoiceName::ALL.len() == 13`

## 9. Tests — Configuration

- [x] 9.1 Test `TXT_TATTLER_INSTRUCTIONS` env sets `ResolvedConfig.instructions`
- [x] 9.2 Test CLI `--instructions` overrides env value
- [x] 9.3 Test config file `instructions` used when CLI and env are absent
- [x] 9.4 Test no instructions source resolves to `None`

## 10. Tests — Cache Key

- [x] 10.1 Test that two requests differing only in `instructions` produce different cache keys
- [x] 10.2 Test that `instructions = None` and `instructions = Some("")` produce different keys
- [x] 10.3 Test that identical requests with the same instructions produce the same key
- [x] 10.4 Test that the full untruncated instructions string is used in key computation

## 11. Tests — Warnings

- [x] 11.1 Test that using `--instructions` with `tts-1` triggers a warning via the reporter
- [x] 11.2 Test that using `--instructions` with `tts-1-hd` triggers a warning via the reporter
- [x] 11.3 Test that using `--instructions` with `gpt-4o-mini-tts` does NOT trigger a warning
- [x] 11.4 Test that speed ≠ 1.0 with `gpt-4o-mini-tts` triggers a speed warning
- [x] 11.5 Test that speed ≠ 1.0 with `tts-1` does NOT trigger a speed warning

## 12. Tests — CLI

- [x] 12.1 Test `--model gpt-4o-mini-tts` is accepted at parse time
- [x] 12.2 Test `--voice ash` and `--voice cedar` are accepted at parse time
- [x] 12.3 Test `--list-voices` exits 0, requires no FILE, and output contains all thirteen voice names

## 13. Verification

- [x] 13.1 Run `cargo fmt --check` — no formatting issues
- [x] 13.2 Run `cargo check` — no compile errors
- [x] 13.3 Run `cargo test` — all tests pass
- [x] 13.4 Run `cargo run -- --list-voices` and verify all thirteen voices are shown
- [ ] 13.5 Run a live synthesis with `--model gpt-4o-mini-tts --instructions "Speak in Dutch."` and verify the audio output
- [ ] 13.6 Run the same command a second time and verify a cache hit is reported
