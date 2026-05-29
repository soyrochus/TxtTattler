# Comparative Implementation Analysis

Implementations checked:

- `codex/`
- `grok/`
- `ghcopilot/`
- `claude/`

Build status: all four implementations passed `cargo check` during this review.

## 1.1 CLI Interface

**Codex: Partial.** Required options are present, including cache flags: `--voice`, `--model`, `--speed`, `--output`, `--no-play`, `--no-cache`, `--refresh`, `--cache-dir`, `--azure`, `--config`, `--list-voices`, and `-V/--verbose` in `codex/src/cli.rs:22-68`. Voice/model invalid values are rejected at parse time via `value_enum` at `codex/src/cli.rs:23` and `codex/src/cli.rs:27`; speed is rejected by `parse_speed` at `codex/src/cli.rs:71-80`. However `file` is `Option<PathBuf>` without `required_unless_present`, and missing file is handled later with `file.unwrap_or_else(|| PathBuf::from(""))` at `codex/src/cli.rs:86` plus a use-case error at `codex/src/application/text_to_speech.rs:86-88`. The help command has no banner; `before_help` is absent in `codex/src/cli.rs:9-17`.

**Grok: Partial.** `--list-voices` is parse-safe because `file` uses `required_unless_present = "list_voices"` at `grok/src/cli.rs:24-26`. Voice/model use `ValueEnum` at `grok/src/cli.rs:92-120`, but speed is a plain `f32` default with no parse-time range validator at `grok/src/cli.rs:36-38`. The spec asks for `-V`; Grok makes verbose long-only and states `-V / --version is reserved` at `grok/src/cli.rs:72-75`. ASCII art exists only in doc comments at `grok/src/cli.rs:7-17`, not as a `before_help`/`after_help` banner.

**GitHub Copilot: ✓.** It has `required_unless_present = "list_voices"` at `ghcopilot/src/cli.rs:29-30`, parse-time voice/model validation with `PossibleValuesParser` at `ghcopilot/src/cli.rs:32-36`, speed validation in `parse_speed` at `ghcopilot/src/cli.rs:76-84`, and `-V/--verbose` at `ghcopilot/src/cli.rs:65-66`. Help includes an emoji/ASCII header via `before_help = HELP_HEADER` at `ghcopilot/src/cli.rs:7-24`.

**Claude: ✗.** `file` is optional without `required_unless_present` at `claude/src/cli.rs:14-15`; missing-file enforcement happens after parse in `claude/src/main.rs:49-54`. Voice/model are raw strings with defaults at `claude/src/cli.rs:17-23`, so invalid values are not rejected at parse time. Speed is a raw `f32` and checked in business logic at `claude/src/main.rs:63-69`. `-V` is assigned to version, not verbose, at `claude/src/cli.rs:61-67`.

## 1.2 Architecture

**Codex: Partial.** The required ports exist in `codex/src/domain/ports.rs:8-22`, and dependencies are injected into `TextToSpeechUseCase::new` at `codex/src/application/text_to_speech.rs:70-83`. The implementation falls short because `FileReaderRegistry` is in the use-case file, not `domain/ports`, at `codex/src/application/text_to_speech.rs:22-61`. The use-case also directly imports console/progress/filesystem concerns with `console::style`, `indicatif`, and `tokio::fs` at `codex/src/application/text_to_speech.rs:8-11`.

**Grok: Partial.** `FileReaderRegistry` is correctly in `domain/ports` at `grok/src/domain/ports.rs:24-62`, and `FileReader`, `TtsProvider`, and `AudioPlayer` are there too at `grok/src/domain/ports.rs:13-21` and `grok/src/domain/ports.rs:64-95`. But there is no `TextProcessor` port; text cleanup is inside the TXT adapter (`clean_text`) at `grok/src/infrastructure/file_readers/txt.rs:53-69`. The use-case imports infrastructure cache directly (`use crate::infrastructure::cache::Mp3Cache`) at `grok/src/application/text_to_speech.rs:9`.

**GitHub Copilot: Partial.** It has the richest ports: `FileReader`, `TtsProvider`, `AudioPlayer`, `TextProcessor`, `ProgressHandle`, and `Reporter` at `ghcopilot/src/domain/ports.rs:8-37`. The service injects registry, TTS, audio, processors, and reporter at `ghcopilot/src/application/text_to_speech.rs:22-37`. The main architecture blemish is that `FileReaderRegistry` lives in infrastructure, not `domain/ports`, at `ghcopilot/src/infrastructure/file_readers/mod.rs:11-56`, and the use-case imports that infrastructure type directly at `ghcopilot/src/application/text_to_speech.rs:16`.

**Claude: Partial.** Basic ports exist in `claude/src/domain/ports.rs:7-25`, and the use-case receives injected `reader`, `tts`, `player`, and `processor` at `claude/src/application/text_to_speech.rs:26-45`. But it is a single `processor`, not a middleware chain, at `claude/src/application/text_to_speech.rs:30`, and direct filesystem writes/reads are in the use-case through `std::fs` at `claude/src/application/text_to_speech.rs:1`, `claude/src/application/text_to_speech.rs:90`, `claude/src/application/text_to_speech.rs:136`, and `claude/src/application/text_to_speech.rs:155`.

## 1.3 Configuration And Precedence

**Codex: Partial.** Environment overrides TOML config through `apply_env_overrides` at `codex/src/config.rs:87-100`. `.env` is loaded before config in `codex/src/main.rs:11-13`. However CLI/default/config precedence for TTS settings is not resolved because `AppConfig` contains only OpenAI/Azure fields at `codex/src/config.rs:6-25`; CLI voice/model/speed are concrete defaults in `codex/src/cli.rs:23-32`, so a config-file voice/model/speed cannot override defaults.

**Grok: ✗.** It claims `CLI > Env > TOML` in `grok/src/config.rs:121`, but credential merge uses `file_cfg.openai.api_key.clone().or_else(|| std::env::var("OPENAI_API_KEY").ok())` at `grok/src/config.rs:137`, so file beats environment. CLI voice/model/speed are concrete defaults at `grok/src/cli.rs:29-38`, making explicit-default detection impossible.

**GitHub Copilot: Partial.** `.env` is loaded from CWD at `ghcopilot/src/config.rs:141-148`. Environment beats file for provider secrets via `env::var(key).ok().or(config_value)` at `ghcopilot/src/config.rs:265-267`. Voice/model use `Option<String>` on the CLI at `ghcopilot/src/cli.rs:32-36`, but speed is a concrete `f32` at `ghcopilot/src/cli.rs:38-39`; `resolve_speed` treats `1.0` as “not supplied” at `ghcopilot/src/config.rs:188-199`, so an explicit `--speed 1.0` cannot override a config value.

**Claude: Partial.** `.env` is loaded in `claude/src/main.rs:37-38`. Voice/model/speed try to let config override defaults by comparing against default values at `claude/src/main.rs:75-96`, but this has the same explicit-default problem: `--voice alloy`, `--model tts-1`, or `--speed 1.0` cannot be distinguished from omission. Provider env/file resolution is embedded in `main` at `claude/src/main.rs:101-115`.

## 1.4 Caching

**Codex: Partial.** The cache key includes a version, cleaned text, voice, model, and speed at `codex/src/application/text_to_speech.rs:239-246`, and separators are null bytes in `cache_key` at `codex/src/utils.rs:88-93`. `--refresh` regenerates at `codex/src/application/text_to_speech.rs:152-168`, and `--no-cache` disables reads/writes at `codex/src/application/text_to_speech.rs:132-136` and `codex/src/application/text_to_speech.rs:171-174`. The weakness is crash safety: `write_mp3` uses direct `fs::write` at `codex/src/application/text_to_speech.rs:251-260`, not temp-file-and-rename. On a cache hit plus `--output`, it writes bytes from memory rather than copying the cache file at `codex/src/application/text_to_speech.rs:176-178`.

**Grok: Partial.** It has atomic writes via temp file then rename in `grok/src/infrastructure/cache/mp3_cache.rs:87-99`, and cache hit output copies from cache at `grok/src/application/text_to_speech.rs:123-129`. Refresh removes the old cache entry at `grok/src/application/text_to_speech.rs:162-169`, and `--no-cache` maps to `None` at `grok/src/application/text_to_speech.rs:91-96`. However the hash uses `b"|"` separators at `grok/src/infrastructure/cache/mp3_cache.rs:53-63`, not null bytes, and `CACHE_VERSION` is a cache constant rather than a processor-chain version at `grok/src/infrastructure/cache/mp3_cache.rs:17-19`.

**GitHub Copilot: Partial.** It uses SHA-256 with null separators over text, voice, model, speed, and pipeline version at `ghcopilot/src/application/text_to_speech.rs:214-231`. `--refresh` blocks reads through `config.cache_enabled && !config.refresh && cache_path.exists()` at `ghcopilot/src/application/text_to_speech.rs:80-87`; `--no-cache` disables writes through `if config.cache_enabled` at `ghcopilot/src/application/text_to_speech.rs:97-109`. `--output` copies from cache when possible in `write_output` at `ghcopilot/src/application/text_to_speech.rs:197-207`. It does not write atomically; cache writes use direct `fs::write` at `ghcopilot/src/application/text_to_speech.rs:104-106`.

**Claude: Partial.** It computes a content/voice/model/speed/version cache key at `claude/src/utils.rs:5-18`, but uses `b"|"` separators at `claude/src/utils.rs:8-16`, not null bytes, and has no processor-chain version. Refresh/no-cache are handled at `claude/src/application/text_to_speech.rs:83-92` and `claude/src/application/text_to_speech.rs:128-151`. Writes are direct `fs::write` at `claude/src/application/text_to_speech.rs:136-137` and `claude/src/application/text_to_speech.rs:155-156`, so no atomic writes and no output copy from cache.

## 1.5 Text Chunking

**Codex: Partial.** It has paragraph handling at `codex/src/utils.rs:65-79`, sentence splitting with `split_inclusive(['.', '!', '?'])` at `codex/src/utils.rs:99-123`, and word fallback at `codex/src/utils.rs:108-120`. There is no character fallback for a single word longer than the limit; a very long word can exceed `limit` because it is pushed whole at `codex/src/utils.rs:116-119`. Tests assert chunk limits for a long space-separated input at `codex/src/utils.rs:140-147`.

**Grok: Partial.** Its chunker is sentence-first and loses punctuation via `split(|c| c == '.' || c == '!' || c == '?')` at `grok/src/domain/entities.rs:210-218`. Hard splitting uses byte chunks at `grok/src/domain/entities.rs:226-235`, which can drop invalid UTF-8 byte fragments. It does not implement the requested paragraph → sentence → word → character hierarchy.

**GitHub Copilot: ✓.** It implements paragraphs at `ghcopilot/src/application/text_to_speech.rs:248-258`, sentence splitting at `ghcopilot/src/application/text_to_speech.rs:286-309`, word splitting at `ghcopilot/src/application/text_to_speech.rs:311-339`, and character fallback at `ghcopilot/src/application/text_to_speech.rs:341-348`. Sentence punctuation is preserved by pushing the current character before boundary checks at `ghcopilot/src/application/text_to_speech.rs:290-294`. Tests assert all chunks stay under limit at `ghcopilot/src/application/text_to_speech.rs:434-440`.

**Claude: Partial.** It documents paragraph/sentence/character splitting at `claude/src/utils.rs:21-25`, implements paragraphs at `claude/src/utils.rs:35-69`, sentence splitting at `claude/src/utils.rs:85-108`, and character splitting at `claude/src/utils.rs:111-118`. It lacks a word fallback stage and uses byte length checks (`text.len()`, `current.len()`) at `claude/src/utils.rs:28`, `claude/src/utils.rs:40`, and `claude/src/utils.rs:50`, so `max_chars` is not consistently character-based.

## 1.6 Text Processing Middleware

**Codex: Partial.** Multiple processors are chainable via `Vec<Arc<dyn TextProcessor>>` at `codex/src/application/text_to_speech.rs:63-65` and looped at `codex/src/application/text_to_speech.rs:99-102`. But `TextProcessor` has no stable `name()` at `codex/src/domain/ports.rs:12-14`, and the cache version is a fixed constant at `codex/src/application/text_to_speech.rs:20`, not derived from processor names. The processor is hard-coded in wiring at `codex/src/adapters.rs:23`.

**Grok: ✗.** There is no `TextProcessor` trait in `grok/src/domain/ports.rs`; cleanup is hard-coded inside `TxtReader::read` at `grok/src/infrastructure/file_readers/txt.rs:53-69`, so changing processors requires adapter changes.

**GitHub Copilot: ✓.** `TextProcessor` has `name()` and `process()` at `ghcopilot/src/domain/ports.rs:21-24`; processors are a vector at `ghcopilot/src/application/text_to_speech.rs:26`; their names form the pipeline version at `ghcopilot/src/application/text_to_speech.rs:180-187`; and markdown stripping is externally activated by `config.strip_markdown` at `ghcopilot/src/adapters.rs:41-47`.

**Claude: Partial.** A `TextProcessor` exists at `claude/src/domain/ports.rs:23-25`, but it has no `name()`, and the use-case accepts only one processor at `claude/src/application/text_to_speech.rs:30`. The default processor is wired directly in `main` at `claude/src/main.rs:124-128`.

## 1.7 OpenAI / Azure Integration

**Codex: Partial.** Standard and Azure OpenAI are supported with `OpenAIConfig` and `AzureConfig` at `codex/src/infrastructure/tts/openai.rs:23-58`; Azure API version is configurable through `config.azure_api_version()` at `codex/src/infrastructure/tts/openai.rs:34-38`. Auto-detection is too broad: `let use_azure = force_azure || config.azure_endpoint().is_some()` at `codex/src/infrastructure/tts/openai.rs:20-22` chooses Azure whenever an endpoint exists, even if `OPENAI_API_KEY` also exists.

**Grok: Partial.** Standard and Azure are supported at `grok/src/infrastructure/tts/openai.rs:17-20` and `grok/src/infrastructure/tts/openai.rs:31-88`; Azure API version comes from `AZURE_OPENAI_API_VERSION` or a default at `grok/src/infrastructure/tts/openai.rs:77-82`. Auto-detection is also too broad: `force_azure || std::env::var("AZURE_OPENAI_ENDPOINT").is_ok()` at `grok/src/infrastructure/tts/openai.rs:31-33` does not check whether `OPENAI_API_KEY` is absent.

**GitHub Copilot: ✓.** It supports both provider configs at `ghcopilot/src/config.rs:34-54` and maps them into async-openai configs at `ghcopilot/src/infrastructure/tts/openai.rs:22-44`. The auto-detection condition exactly includes “OpenAI key absent and Azure endpoint present” at `ghcopilot/src/config.rs:113-117`. Azure API version is configurable with default fallback at `ghcopilot/src/config.rs:251-255`.

**Claude: Partial.** Standard and Azure providers exist at `claude/src/infrastructure/tts/openai.rs:16-35`, but Azure API version is hard-coded to `"2024-02-01"` at `claude/src/infrastructure/tts/openai.rs:29-33`. Auto-detection is too broad: `cli.azure || std::env::var("AZURE_OPENAI_ENDPOINT").is_ok()` at `claude/src/main.rs:100-102`.

## 1.8 Encoding

**Codex: Partial.** UTF-8 BOM is stripped at `codex/src/infrastructure/file_readers/txt.rs:12-14`, and invalid UTF-8 surfaces as an error at `codex/src/infrastructure/file_readers/txt.rs:16-21`. UTF-16LE/BE BOMs are not supported.

**Grok: ✗.** UTF-8 BOM is handled at `grok/src/infrastructure/file_readers/txt.rs:26-31`, but decode errors are only warnings at `grok/src/infrastructure/file_readers/txt.rs:34-49`; the code returns replacement-character output at `grok/src/infrastructure/file_readers/txt.rs:51-64`. UTF-16 is not supported.

**GitHub Copilot: ✓.** UTF-8 BOM support is at `ghcopilot/src/infrastructure/file_readers/txt.rs:16-19`; UTF-16LE at `ghcopilot/src/infrastructure/file_readers/txt.rs:21-27`; UTF-16BE at `ghcopilot/src/infrastructure/file_readers/txt.rs:29-35`; invalid encodings return user-visible errors at `ghcopilot/src/infrastructure/file_readers/txt.rs:23-24`, `ghcopilot/src/infrastructure/file_readers/txt.rs:31-32`, and `ghcopilot/src/infrastructure/file_readers/txt.rs:37-42`.

**Claude: ✗.** UTF-8 BOM is stripped at `claude/src/infrastructure/file_readers/txt.rs:14-19`, but invalid bytes are silently replaced via `String::from_utf8_lossy` at `claude/src/infrastructure/file_readers/txt.rs:21`. UTF-16 is not supported.

## 1.9 Output And Feedback

**Codex: Partial.** It has clear phase labels (`Reading`, `Cleaning text`, `Prepared`, `Settings`, `Cache hit/miss`, `OpenAI TTS`, `Playing audio`, `Done`) at `codex/src/application/text_to_speech.rs:90-192`, and a progress bar at `codex/src/application/text_to_speech.rs:210-225`. But output is hard-coded with `println!` in the use-case, not behind a `Reporter` port.

**Grok: Partial.** It has progress and phase output at `grok/src/application/text_to_speech.rs:77-89`, `grok/src/application/text_to_speech.rs:173-203`, and `grok/src/application/text_to_speech.rs:267-279`, but uses direct `println!` and `indicatif` in the use-case rather than a reporter port.

**GitHub Copilot: ✓.** Output is behind `Reporter`/`ProgressHandle` ports at `ghcopilot/src/domain/ports.rs:26-37`, and the use-case calls `self.reporter.status/detail/progress/success` at `ghcopilot/src/application/text_to_speech.rs:47-57`, `ghcopilot/src/application/text_to_speech.rs:80-91`, and `ghcopilot/src/application/text_to_speech.rs:120-137`. Phase labels include read, cleanup middleware, cache hit, generation, write, playback, and done.

**Claude: Partial.** It prints phase labels and uses a progress bar at `claude/src/application/text_to_speech.rs:49-64`, `claude/src/application/text_to_speech.rs:97-123`, and `claude/src/application/text_to_speech.rs:163-178`, but all output is direct `println!`/`eprintln!` in the use-case.

## 2.1 Idiomatic Type Usage

**Codex: Partial.** Voice/model are enums at `codex/src/domain/entities.rs:3-39`, but they lack `FromStr` and `Serialize`/`Deserialize`. CLI voice/model/speed use concrete defaults (`VoiceArg`, `ModelArg`, `f32`) at `codex/src/cli.rs:23-32`, not `Option<T>`. Non-test code avoids `unwrap`, except safe style-template `unwrap` at `codex/src/application/text_to_speech.rs:212-214` and tracing discard `let _ = ...` at `codex/src/utils.rs:19-22`.

**Grok: Partial.** Voice/model derive `Serialize`/`Deserialize`, implement `Display`, and implement `FromStr` at `grok/src/domain/entities.rs:9-66` and `grok/src/domain/entities.rs:68-100`. However CLI defaults are concrete at `grok/src/cli.rs:29-38`, `tempfile` is duplicated in dependencies and dev-dependencies at `grok/Cargo.toml:54` and `grok/Cargo.toml:62-64`, and production code has `expect("FILE is required")` at `grok/src/main.rs:76-78`.

**GitHub Copilot: ✓.** Voice/model derive `Serialize`/`Deserialize`, implement `Display`, `FromStr`, and use `const ALL` arrays at `ghcopilot/src/domain/entities.rs:6-64` and `ghcopilot/src/domain/entities.rs:67-107`. CLI voice/model are `Option<String>` at `ghcopilot/src/cli.rs:32-36`; speed remains concrete at `ghcopilot/src/cli.rs:38-39`.

**Claude: Partial.** Voice/model are enums with `Display` and ad hoc `from_str` methods at `claude/src/domain/entities.rs:3-83`, but not `FromStr` trait or `Serialize`/`Deserialize`. CLI uses raw `String` defaults at `claude/src/cli.rs:17-23`. Encoding errors are hidden by `from_utf8_lossy` at `claude/src/infrastructure/file_readers/txt.rs:21`.

## 2.2 Dependency Hygiene

**Codex: Partial.** Metadata is complete at `codex/Cargo.toml:1-10`, and `tempfile` is correctly dev-only at `codex/Cargo.toml:34-35`. Some versions are broad rather than pinned (`anyhow = "1"`, `async-trait = "0.1"`, etc.) at `codex/Cargo.toml:17-32`.

**Grok: Partial.** Metadata is good at `grok/Cargo.toml:1-11`, but `async-openai = "0.26"` at `grok/Cargo.toml:24-25` is older than the current line used by the other implementations, and `tempfile` appears in both dependencies and dev-dependencies at `grok/Cargo.toml:54` and `grok/Cargo.toml:62-64`.

**GitHub Copilot: ✓.** Metadata is present at `ghcopilot/Cargo.toml:1-9`, current versions are pinned at `ghcopilot/Cargo.toml:11-28`, and test crates are dev-dependencies at `ghcopilot/Cargo.toml:30-33`.

**Claude: Partial.** Basic metadata exists at `claude/Cargo.toml:1-7`, but `keywords` and `categories` are absent. Several dependencies are broad (`clap = "4"`, `tokio = "1"`) at `claude/Cargo.toml:14-15`, and `async-openai = "0.28"` at `claude/Cargo.toml:13` is older than the current `0.40.x` line used elsewhere.

## 2.3 Dead Code And Logic Errors

**Codex: Partial.** No obvious unreachable/noop guards, but `let _ = tracing_subscriber...try_init()` discards initialization errors at `codex/src/utils.rs:19-22`. `write_mp3` uses direct writes with no atomicity at `codex/src/application/text_to_speech.rs:251-260`.

**Grok: Partial.** The TXT cleanup contains a no-op filter `.filter(|l| !l.is_empty() || true)` at `grok/src/infrastructure/file_readers/txt.rs:87-91`. There are multiple `#[allow(dead_code)]` production suppressions, for example `grok/src/domain/ports.rs:71-73` and `grok/src/domain/entities.rs:33-34`.

**GitHub Copilot: ✓.** I found no `unwrap`/`expect` in non-test code, no noop guard, and no visible copy-paste dead blocks in the reviewed paths. Error contexts are generally specific, e.g. cache read/write errors at `ghcopilot/src/application/text_to_speech.rs:85-106`.

**Claude: Partial.** No major unreachable code found, but lossy decoding is a logic error against the spec at `claude/src/infrastructure/file_readers/txt.rs:21`, and `main.rs` performs large amounts of wiring/config logic inline at `claude/src/main.rs:25-143`.

## 2.4 Testability

**Codex: Partial.** The cache reuse integration-style unit test exists at `codex/src/application/text_to_speech.rs:300-338`. But use-case testability is harmed by direct filesystem/cache I/O and stdout/progress calls in `codex/src/application/text_to_speech.rs:90-192` and `codex/src/application/text_to_speech.rs:251-260`.

**Grok: Partial.** Dependencies are injected into `TextToSpeechOrchestrator::new` at `grok/src/application/text_to_speech.rs:25-36`, but the use-case directly constructs `Mp3Cache` at `grok/src/application/text_to_speech.rs:91-96` and prints directly. No reporter/noop reporter exists.

**GitHub Copilot: ✓.** The use-case accepts a `Reporter` and `ProgressHandle` abstraction at `ghcopilot/src/application/text_to_speech.rs:22-37`, and ports define the test handles at `ghcopilot/src/domain/ports.rs:26-37`. This is the easiest implementation to test without capturing stdout.

**Claude: Partial.** It injects reader/TTS/player/processor at `claude/src/application/text_to_speech.rs:26-45`, but direct `std::fs` and `println!` use prevents clean port-level testing.

## 2.5 Test Coverage

**Codex: Partial.** It tests UTF-8 BOM reading at `codex/src/infrastructure/file_readers/txt.rs:36-45`, chunk limits at `codex/src/utils.rs:140-147`, cache-key parameter sensitivity at `codex/src/utils.rs:149-152`, config parsing at `codex/src/config.rs:120-139`, and cache reuse at `codex/src/application/text_to_speech.rs:300-338`. It does not test entity parsing because there is no `FromStr`, nor env-over-file/CLI-over-env config precedence.

**Grok: Partial.** It has entity parsing code but no visible tests for case-insensitive entity parsing in `grok/src/domain/entities.rs`. TXT cleanup tests exist at `grok/src/infrastructure/file_readers/txt.rs:118-134`. The chunker lacks a test asserting every chunk stays under limit.

**GitHub Copilot: ✓.** It tests voice/model case-insensitive parsing at `ghcopilot/src/domain/entities.rs:126-142`, speed config/default behavior at `ghcopilot/src/config.rs:286-299`, chunk limits at `ghcopilot/src/application/text_to_speech.rs:434-440`, cache-key changes at `ghcopilot/src/application/text_to_speech.rs:427-432`, processors at `ghcopilot/src/application/text_to_speech.rs:442-469`, and text reader UTF-8 BOM/plain UTF-8 at `ghcopilot/src/infrastructure/file_readers/txt.rs:46-72`. It still lacks a UTF-16 reader test despite supporting UTF-16.

**Claude: ✗.** I found no `#[cfg(test)]` blocks in the reviewed Claude files; the implementation does not demonstrate the required coverage.

## 2.6 Entry Point Cleanliness

**Codex: Partial.** `main.rs` is just over the target at 61 lines (`codex/src/main.rs:1-61`) and includes `.env` loading and output formatting. Config resolution itself is delegated to `AppConfig::load` at `codex/src/main.rs:29`.

**Grok: ✗.** `main.rs` is 153 lines and performs config resolution, provider selection, adapter wiring, document loading, and execution in one place (`grok/src/main.rs:26-131`).

**GitHub Copilot: ✓.** `main.rs` is compact at 28 lines (`ghcopilot/src/main.rs:1-28`), delegates config resolution to `ResolvedConfig::from_cli` at `ghcopilot/src/main.rs:23`, and delegates wiring to `App::bootstrap` at `ghcopilot/src/main.rs:26`.

**Claude: ✗.** `main.rs` is 143 lines and contains raw environment reads, file existence checks, config merge logic, provider construction, registry lookup, processor creation, and use-case invocation at `claude/src/main.rs:25-143`.

## Feature Scorecard

| Criterion | Codex | Grok | GitHub Copilot | Claude |
|---|---:|---:|---:|---:|
| 1.1 CLI interface | Partial | Partial | ✓ | ✗ |
| 1.2 Architecture | Partial | Partial | Partial | Partial |
| 1.3 Config precedence | Partial | ✗ | Partial | Partial |
| 1.4 Caching | Partial | Partial | Partial | Partial |
| 1.5 Text chunking | Partial | Partial | ✓ | Partial |
| 1.6 Text middleware | Partial | ✗ | ✓ | Partial |
| 1.7 OpenAI/Azure | Partial | Partial | ✓ | Partial |
| 1.8 Encoding | Partial | ✗ | ✓ | ✗ |
| 1.9 Output/feedback | Partial | Partial | ✓ | Partial |
| 2.1 Type usage | Partial | Partial | ✓ | Partial |
| 2.2 Dependency hygiene | Partial | Partial | ✓ | Partial |
| 2.3 Dead code/logic | Partial | Partial | ✓ | Partial |
| 2.4 Testability | Partial | Partial | ✓ | Partial |
| 2.5 Test coverage | Partial | Partial | ✓ | ✗ |
| 2.6 Entry point | Partial | ✗ | ✓ | ✗ |

## Verdict

1. **GitHub Copilot** is the best starting point for production. It most consistently satisfies the newer requirements: parse-safe CLI (`ghcopilot/src/cli.rs:29-66`), `.env` loading from CWD (`ghcopilot/src/config.rs:141-148`), auto Azure detection exactly as specified (`ghcopilot/src/config.rs:113-117`), reporter/progress ports (`ghcopilot/src/domain/ports.rs:26-37`), named processor chain (`ghcopilot/src/application/text_to_speech.rs:180-187`), robust chunking (`ghcopilot/src/application/text_to_speech.rs:243-348`), and UTF-16-aware text reading (`ghcopilot/src/infrastructure/file_readers/txt.rs:21-35`). Its main production gap is non-atomic cache writes (`ghcopilot/src/application/text_to_speech.rs:104-106`).

2. **Codex** is second. It compiles, has simple wiring, null-byte cache separators (`codex/src/utils.rs:88-93`), `.env` loading with a crate-dir fallback (`codex/src/main.rs:11-13`), and a useful cache-hit test (`codex/src/application/text_to_speech.rs:300-338`). It is weaker than GitHub Copilot because console/file/cache concerns live directly in the use-case (`codex/src/application/text_to_speech.rs:8-11`, `codex/src/application/text_to_speech.rs:90-192`), config does not cover TTS defaults (`codex/src/config.rs:6-25`), and encoding lacks UTF-16 support.

3. **Grok** is third. It has a good domain-level registry (`grok/src/domain/ports.rs:24-62`) and the only atomic cache implementation (`grok/src/infrastructure/cache/mp3_cache.rs:87-99`). But it has config precedence bugs (`grok/src/config.rs:137-144`), no `TextProcessor` middleware port, direct infrastructure use in the use-case (`grok/src/application/text_to_speech.rs:9`), a noop filter (`grok/src/infrastructure/file_readers/txt.rs:87-91`), and older `async-openai` (`grok/Cargo.toml:24-25`).

4. **Claude** is fourth. It compiles and has the broad outline of ports/adapters, but it misses parse-time validation, uses `-V` for version instead of verbose (`claude/src/cli.rs:61-67`), silently replaces invalid encoding (`claude/src/infrastructure/file_readers/txt.rs:21`), keeps too much logic in `main.rs` (`claude/src/main.rs:25-143`), and has no visible tests in the reviewed source.

## Ideal Merge

- Start from **GitHub Copilot** as the base because it has the best config, reporter, middleware, chunking, encoding, and test structure.
- Cherry-pick **Grok**’s atomic cache writes: temp file then rename from `grok/src/infrastructure/cache/mp3_cache.rs:87-99`.
- Cherry-pick **Codex**’s null-byte-separated cache helper style from `codex/src/utils.rs:88-93` if not keeping GitHub Copilot’s existing null-byte `build_cache_key` at `ghcopilot/src/application/text_to_speech.rs:221-231`.
- Fix GitHub Copilot’s speed CLI to be `Option<f32>` rather than `f32` so explicit `--speed 1.0` can override config; current ambiguity is in `ghcopilot/src/cli.rs:38-39` and `ghcopilot/src/config.rs:188-199`.
- Move or abstract `FileReaderRegistry` so the use-case does not import infrastructure directly; current import is `ghcopilot/src/application/text_to_speech.rs:16`.
- Add UTF-16 reader tests to GitHub Copilot to match its implementation in `ghcopilot/src/infrastructure/file_readers/txt.rs:21-35`.
- Make cache writes atomic in GitHub Copilot by replacing direct `fs::write` at `ghcopilot/src/application/text_to_speech.rs:104-106` with temp-file-and-rename.
- Preserve GitHub Copilot’s `Reporter` and `ProgressHandle` ports; do not regress to direct `println!` in the use-case.
