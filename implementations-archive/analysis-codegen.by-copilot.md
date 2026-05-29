# Comparative Implementation Analysis

Implementations checked:

- `codex/`
- `grok/`
- `ghcopilot/`
- `claude/`

## 1.1 CLI Interface

**Codex: Partial.** The CLI exposes the required flags in `codex/src/cli.rs:1-68`, and parse-time validation exists for voice, model, and speed through `ValueEnum` and `parse_speed`. The gap is `file: Option<PathBuf>` without `required_unless_present`, so a missing file is handled after parsing in `codex/src/application/text_to_speech.rs:80-88` instead of being rejected by clap. The help text has no banner or ASCII-art hook in `codex/src/cli.rs`.

**Grok: Partial.** It enforces `required_unless_present = "list_voices"` for the file argument in `grok/src/cli.rs:24-26`, and voice/model are parse-validated with `ValueEnum` in `grok/src/cli.rs:92-120`. Speed remains a plain `f32` default in `grok/src/cli.rs:36-38`, so invalid values are not rejected at parse time. The verbose flag is not `-V`; Grok reserves that for version in `grok/src/cli.rs:72-75`.

**GitHub Copilot: ✓.** It has parse-time enforcement for `FILE` via `required_unless_present = "list_voices"` in `ghcopilot/src/cli.rs:29-30`, voice/model are validated through `PossibleValuesParser` in `ghcopilot/src/cli.rs:32-36`, and speed is checked by `parse_speed` in `ghcopilot/src/cli.rs:76-84`. Help includes an explicit banner via `before_help = HELP_HEADER` in `ghcopilot/src/cli.rs:7-24`.

**Claude: ✗.** The CLI leaves `file` optional in `claude/src/cli.rs:14-15` and handles missing-file errors later in `claude/src/main.rs:49-54`. Voice/model are raw `String` fields in `claude/src/cli.rs:17-23`, so clap does not reject invalid values. `-V` is wired to version in `claude/src/cli.rs:61-67`, not verbose.

## 1.2 Architecture — Hexagonal / Ports & Adapters

**Codex: Partial.** The required `FileReader`, `TtsProvider`, and `AudioPlayer` traits are present in `codex/src/domain/ports.rs:8-22`, but `FileReaderRegistry` lives in `codex/src/application/text_to_speech.rs:22-61` instead of `domain/ports`. The use-case also reaches directly into console, progress, and filesystem concerns at `codex/src/application/text_to_speech.rs:8-11`, so the boundaries are not cleanly hexagonal.

**Grok: Partial.** It places `FileReaderRegistry` in `domain/ports` at `grok/src/domain/ports.rs:24-62` and defines the core reading, TTS, and playback ports there. The miss is text processing: there is no `TextProcessor` port, and cleanup is hard-coded into the TXT adapter in `grok/src/infrastructure/file_readers/txt.rs:53-69`. The use-case also imports cache infrastructure directly in `grok/src/application/text_to_speech.rs:9`.

**GitHub Copilot: Partial.** It has the richest port set, including `TextProcessor`, `Reporter`, and `ProgressHandle` in `ghcopilot/src/domain/ports.rs:8-37`, and the use-case receives those dependencies in `ghcopilot/src/application/text_to_speech.rs:22-37`. The spec mismatch is that `FileReaderRegistry` lives in infrastructure at `ghcopilot/src/infrastructure/file_readers/mod.rs:11-56`, and the application layer imports it directly in `ghcopilot/src/application/text_to_speech.rs:16`.

**Claude: Partial.** Basic ports exist in `claude/src/domain/ports.rs:7-25`, and the use-case receives injected reader, TTS, player, and processor dependencies in `claude/src/application/text_to_speech.rs:26-45`. It still uses only one processor rather than a chain, and the use-case does direct filesystem I/O at `claude/src/application/text_to_speech.rs:90`, `claude/src/application/text_to_speech.rs:136`, and `claude/src/application/text_to_speech.rs:155`.

## 1.3 Configuration and Precedence

**Codex: Partial.** Environment overrides config for the provider fields through `apply_env_overrides` in `codex/src/config.rs:87-100`, and `.env` is loaded before config in `codex/src/main.rs:11-13`. The problem is that CLI voice/model/speed are concrete defaults in `codex/src/cli.rs:23-32`, so a config file cannot override a user-supplied default value because the code cannot distinguish omission from an explicit default.

**Grok: ✗.** The precedence bug is in `grok/src/config.rs:137-144`, where `file_cfg.openai.api_key.clone().or_else(|| std::env::var("OPENAI_API_KEY").ok())` gives the file higher priority than environment variables. It also uses concrete CLI defaults in `grok/src/cli.rs:29-38`, which means explicit default values are indistinguishable from omission.

**GitHub Copilot: Partial.** `.env` is loaded from the current directory in `ghcopilot/src/config.rs:141-148`, and provider secrets resolve with environment taking precedence over file values in `ghcopilot/src/config.rs:265-267`. The remaining flaw is speed: `ghcopilot/src/cli.rs:38-39` uses a concrete `f32`, and `resolve_speed` in `ghcopilot/src/config.rs:188-199` treats `1.0` as if it were not supplied, so config can still override an explicit default-speed CLI argument.

**Claude: Partial.** `.env` is loaded in `claude/src/main.rs:37-38`, but the voice/model/speed resolution logic in `claude/src/main.rs:75-96` still conflates explicit defaults with omission. Provider environment and file resolution is also embedded directly in `main` at `claude/src/main.rs:101-115`, so the precedence chain is not cleanly separated.

## 1.4 Caching

**Codex: Partial.** The cache key is content-based in `codex/src/application/text_to_speech.rs:239-246`, and `codex/src/utils.rs:88-93` uses null-byte separators. `--refresh` and `--no-cache` are respected in `codex/src/application/text_to_speech.rs:132-174`, but cache writes are direct `fs::write` calls in `codex/src/application/text_to_speech.rs:251-260`, so they are not atomic. `--output` writes from memory rather than copying the cached file in `codex/src/application/text_to_speech.rs:176-178`.

**Grok: Partial.** It does have atomic temp-file-then-rename writes in `grok/src/infrastructure/cache/mp3_cache.rs:87-99`, and cache hits can be copied to output in `grok/src/application/text_to_speech.rs:123-129`. The hash uses `b"|"` separators in `grok/src/infrastructure/cache/mp3_cache.rs:53-63`, which do not satisfy the null-byte separator requirement, and `CACHE_VERSION` is a fixed cache constant rather than a version derived from the text-processing pipeline.

**GitHub Copilot: Partial.** It computes a SHA-256 cache key with null separators over text, voice, model, speed, and pipeline version in `ghcopilot/src/application/text_to_speech.rs:214-231`. `--refresh`, `--no-cache`, and cache-copy-to-output behavior are covered in `ghcopilot/src/application/text_to_speech.rs:80-109` and `ghcopilot/src/application/text_to_speech.rs:197-207`. The missing piece is atomic writes: cache persistence still uses direct `fs::write` in `ghcopilot/src/application/text_to_speech.rs:104-106`.

**Claude: Partial.** Its cache key is content-based in `claude/src/utils.rs:5-18`, but it uses `b"|"` separators in `claude/src/utils.rs:8-16`, and there is no pipeline-version input. Refresh and no-cache behavior exists in `claude/src/application/text_to_speech.rs:83-92` and `claude/src/application/text_to_speech.rs:128-151`, but writes are not atomic and output does not copy from cache.

## 1.5 Text Chunking

**Codex: Partial.** It implements paragraph, sentence, and word splitting in `codex/src/utils.rs:65-123`, but it lacks a true character fallback when a single word still exceeds the limit. The chunker is tested for size limits in `codex/src/utils.rs:140-147`, but the algorithm can still overrun on long unbroken words.

**Grok: Partial.** The chunking logic is sentence-first and strips punctuation with `split(|c| c == '.' || c == '!' || c == '?')` in `grok/src/domain/entities.rs:210-218`, so it does not preserve sentence boundaries as required. It falls back to byte splitting in `grok/src/domain/entities.rs:226-235`, which is not the requested paragraph → sentence → word → character hierarchy.

**GitHub Copilot: ✓.** It has paragraph splitting in `ghcopilot/src/application/text_to_speech.rs:248-258`, sentence handling in `ghcopilot/src/application/text_to_speech.rs:286-309`, word fallback in `ghcopilot/src/application/text_to_speech.rs:311-339`, and character fallback in `ghcopilot/src/application/text_to_speech.rs:341-348`. The tests assert chunk size bounds in `ghcopilot/src/application/text_to_speech.rs:434-440`.

**Claude: Partial.** It provides paragraph, sentence, and character splitting in `claude/src/utils.rs:35-122`, but there is no word fallback stage and the code relies on byte lengths in places where character counts are required. That makes `max_chars` enforcement less trustworthy than in the strongest implementation.

## 1.6 Text Processing Middleware

**Codex: Partial.** It can chain processors via `Vec<Arc<dyn TextProcessor>>` in `codex/src/application/text_to_speech.rs:63-65` and applies them in a loop in `codex/src/application/text_to_speech.rs:99-102`. The problem is that `TextProcessor` has no stable `name()` in `codex/src/domain/ports.rs:12-14`, so the cache version cannot be derived from the pipeline, and the wiring is still effectively fixed in `codex/src/adapters.rs:23`.

**Grok: ✗.** There is no `TextProcessor` port in `grok/src/domain/ports.rs`, and cleanup is embedded directly in `TxtReader::read` at `grok/src/infrastructure/file_readers/txt.rs:53-69`. That means post-processing cannot be configured independently of the adapter.

**GitHub Copilot: ✓.** `TextProcessor` has both `name()` and `process()` in `ghcopilot/src/domain/ports.rs:21-24`, the use-case accepts a vector of processors in `ghcopilot/src/application/text_to_speech.rs:26`, and the pipeline version is derived from processor names in `ghcopilot/src/application/text_to_speech.rs:180-187`. Markdown stripping is activated from configuration rather than hard-coded in the use-case in `ghcopilot/src/adapters.rs:41-47`.

**Claude: Partial.** A `TextProcessor` trait exists in `claude/src/domain/ports.rs:23-25`, but it lacks `name()`, and the use-case only accepts a single processor in `claude/src/application/text_to_speech.rs:30`. The processor is wired directly in `claude/src/main.rs:124-128` instead of being composed as middleware.

## 1.7 OpenAI / Azure Integration

**Codex: Partial.** It supports both OpenAI and Azure OpenAI in `codex/src/infrastructure/tts/openai.rs:23-58`, and Azure API version is configurable there as well. The auto-detection rule is too broad in `codex/src/infrastructure/tts/openai.rs:20-22`, because the presence of an Azure endpoint alone is enough to switch modes even when a standard OpenAI key is also available.

**Grok: Partial.** It supports both provider paths in `grok/src/infrastructure/tts/openai.rs:17-20` and `grok/src/infrastructure/tts/openai.rs:31-88`, with API version control in `grok/src/infrastructure/tts/openai.rs:77-82`. Auto-detection is still too broad in `grok/src/infrastructure/tts/openai.rs:31-33`, because it checks only for the endpoint, not the absence of `OPENAI_API_KEY`.

**GitHub Copilot: ✓.** It supports both provider configs in `ghcopilot/src/config.rs:34-54`, and the auto-detection logic exactly matches the spec in `ghcopilot/src/config.rs:113-117`: Azure mode activates when the OpenAI key is absent and the Azure endpoint is present. Azure API version is configurable in `ghcopilot/src/config.rs:251-255`.

**Claude: Partial.** Standard and Azure providers exist in `claude/src/infrastructure/tts/openai.rs:16-35`, but Azure API version is hard-coded to `2024-02-01` in `claude/src/infrastructure/tts/openai.rs:29-33`. Auto-detection is broader than the spec in `claude/src/main.rs:100-102`.

## 1.8 Encoding

**Codex: Partial.** UTF-8 BOM stripping is present in `codex/src/infrastructure/file_readers/txt.rs:12-14`, and invalid UTF-8 is surfaced as an error in `codex/src/infrastructure/file_readers/txt.rs:16-21`. UTF-16LE and UTF-16BE are not handled.

**Grok: ✗.** It handles UTF-8 BOMs in `grok/src/infrastructure/file_readers/txt.rs:26-31`, but encoding problems are downgraded to warnings and replacement output in `grok/src/infrastructure/file_readers/txt.rs:34-64`. That violates the requirement that encoding errors be user-visible errors.

**GitHub Copilot: ✓.** It handles UTF-8 BOM, UTF-16LE, and UTF-16BE in `ghcopilot/src/infrastructure/file_readers/txt.rs:16-35`, and invalid encodings are returned as errors in `ghcopilot/src/infrastructure/file_readers/txt.rs:23-24`, `ghcopilot/src/infrastructure/file_readers/txt.rs:31-32`, and `ghcopilot/src/infrastructure/file_readers/txt.rs:37-42`.

**Claude: ✗.** UTF-8 BOM stripping exists in `claude/src/infrastructure/file_readers/txt.rs:14-19`, but invalid bytes are silently replaced with `String::from_utf8_lossy` in `claude/src/infrastructure/file_readers/txt.rs:21`. UTF-16 is not supported.

## 1.9 Output and Feedback

**Codex: Partial.** It labels the major phases in `codex/src/application/text_to_speech.rs:90-192` and shows a progress bar in `codex/src/application/text_to_speech.rs:210-225`. The reporting is still direct `println!` output from the use-case, so there is no dedicated reporter port.

**Grok: Partial.** It emits phase and progress output in `grok/src/application/text_to_speech.rs:77-89`, `grok/src/application/text_to_speech.rs:173-203`, and `grok/src/application/text_to_speech.rs:267-279`, but again this is direct console output from the use-case rather than an injected reporting abstraction.

**GitHub Copilot: ✓.** It routes output through `Reporter` and `ProgressHandle` in `ghcopilot/src/domain/ports.rs:26-37`, and the use-case consumes those ports in `ghcopilot/src/application/text_to_speech.rs:47-57`, `ghcopilot/src/application/text_to_speech.rs:80-91`, and `ghcopilot/src/application/text_to_speech.rs:120-137`. The phase labels are explicit for read, cleanup, cache, synthesis, output, playback, and completion.

**Claude: Partial.** It has phase labels and a progress bar in `claude/src/application/text_to_speech.rs:49-64`, `claude/src/application/text_to_speech.rs:97-123`, and `claude/src/application/text_to_speech.rs:163-178`, but all feedback still comes from direct `println!`/`eprintln!` calls in the use-case.

## 2.1 Idiomatic Type Usage

**Codex: Partial.** Voice and model are enums in `codex/src/domain/entities.rs:3-39`, but they do not implement `FromStr` or `Serialize`/`Deserialize`. The CLI uses concrete defaults for voice, model, and speed in `codex/src/cli.rs:23-32`, so the code cannot distinguish omission from an explicit default value.

**Grok: Partial.** Voice and model implement `Display`, `Serialize`/`Deserialize`, and `FromStr` in `grok/src/domain/entities.rs:9-100`. The issue is that the CLI still uses concrete defaults in `grok/src/cli.rs:29-38`, and production code contains an `expect("FILE is required")` in `grok/src/main.rs:76-78`.

**GitHub Copilot: ✓.** Voice and model derive `Serialize`/`Deserialize`, implement `Display` and `FromStr`, and use `const ALL` arrays in `ghcopilot/src/domain/entities.rs:6-107`. CLI voice and model are `Option<String>` in `ghcopilot/src/cli.rs:32-36`, which preserves the distinction between omitted and defaulted values.

**Claude: Partial.** Voice and model are enums with ad hoc parsing helpers in `claude/src/domain/entities.rs:3-83`, but not `FromStr` or `Serialize`/`Deserialize`. CLI defaults remain raw strings in `claude/src/cli.rs:17-23`.

## 2.2 Dependency Hygiene

**Codex: Partial.** The manifest has basic metadata and keeps test-only crates in dev-dependencies in `codex/Cargo.toml:1-35`, but the versions are broad rather than tightly pinned. That is acceptable but weaker than the best implementation.

**Grok: Partial.** Metadata is present in `grok/Cargo.toml:1-11`, but `async-openai = "0.26"` is older than the current line used by the stronger implementation, and `tempfile` is duplicated between dependencies and dev-dependencies in `grok/Cargo.toml:54` and `grok/Cargo.toml:62-64`.

**GitHub Copilot: ✓.** It includes the expected crate metadata in `ghcopilot/Cargo.toml:1-9`, pins the current dependency line in `ghcopilot/Cargo.toml:11-28`, and keeps test crates in dev-dependencies in `ghcopilot/Cargo.toml:30-33`.

**Claude: Partial.** The manifest has only minimal metadata in `claude/Cargo.toml:1-7`, omits keywords and categories, and uses broad version constraints such as `clap = "4"` and `tokio = "1"` in `claude/Cargo.toml:14-15`.

## 2.3 Dead Code and Logic Errors

**Codex: Partial.** There are no obvious noop guards in the reviewed paths, but `let _ = tracing_subscriber...try_init()` in `codex/src/utils.rs:19-22` silently discards initialization failure, and cache writes are non-atomic in `codex/src/application/text_to_speech.rs:251-260`.

**Grok: Partial.** It contains a literal no-op filter in `grok/src/infrastructure/file_readers/txt.rs:87-91`, and it also has production `#[allow(dead_code)]` suppressions in `grok/src/domain/ports.rs:71-73` and `grok/src/domain/entities.rs:33-34`.

**GitHub Copilot: ✓.** I did not find noop guards, production `expect`/`unwrap`, or similar obvious logic debt in the reviewed paths. Error contexts around cache and output operations are specific in `ghcopilot/src/application/text_to_speech.rs:85-106`.

**Claude: Partial.** The major logic issue is lossy decoding in `claude/src/infrastructure/file_readers/txt.rs:21`, which violates the encoding requirement. The rest of `claude/src/main.rs:25-143` is also overloaded with wiring and configuration logic.

## 2.4 Testability

**Codex: Partial.** It has a cache-reuse integration-style test in `codex/src/application/text_to_speech.rs:300-338`, but direct filesystem, cache, and console I/O remain inside the use-case in `codex/src/application/text_to_speech.rs:90-192` and `codex/src/application/text_to_speech.rs:251-260`.

**Grok: Partial.** Dependencies are injected into the orchestrator in `grok/src/application/text_to_speech.rs:25-36`, but the use-case still constructs cache infrastructure directly in `grok/src/application/text_to_speech.rs:91-96` and prints directly.

**GitHub Copilot: ✓.** It is the easiest implementation to test because the use-case depends on `Reporter`, `ProgressHandle`, and injected ports in `ghcopilot/src/application/text_to_speech.rs:22-37` and `ghcopilot/src/domain/ports.rs:26-37`.

**Claude: Partial.** Reader, TTS, player, and processor are injected in `claude/src/application/text_to_speech.rs:26-45`, but direct `std::fs` and `println!` use still makes the use-case harder to test cleanly.

## 2.5 Test Coverage

**Codex: Partial.** It covers UTF-8 BOM reading in `codex/src/infrastructure/file_readers/txt.rs:36-45`, chunk-size behavior in `codex/src/utils.rs:140-147`, cache-key sensitivity in `codex/src/utils.rs:149-152`, config parsing in `codex/src/config.rs:120-139`, and cache reuse in `codex/src/application/text_to_speech.rs:300-338`. It still lacks entity parsing tests and env/config precedence tests.

**Grok: Partial.** It has tests in `grok/src/infrastructure/file_readers/txt.rs:118-134`, but the reviewed source does not show case-insensitive entity parsing tests or a chunk-limit test that proves every chunk stays under the configured maximum.

**GitHub Copilot: ✓.** It tests entity parsing in `ghcopilot/src/domain/entities.rs:126-142`, chunk limits and cache-key changes in `ghcopilot/src/application/text_to_speech.rs:427-440`, processor behavior in `ghcopilot/src/application/text_to_speech.rs:442-469`, and text-reader behavior in `ghcopilot/src/infrastructure/file_readers/txt.rs:46-72`.

**Claude: ✗.** The reviewed Claude source does not show a comparable test suite, so the required coverage is not demonstrated.

## 2.6 Entry-Point Cleanliness

**Codex: Partial.** `main.rs` is close to the target size in `codex/src/main.rs:1-61`, and some setup is delegated to config helpers. It still performs `.env` loading and other startup behavior directly instead of pushing all runtime resolution into a dedicated bootstrap layer.

**Grok: ✗.** `grok/src/main.rs:26-131` is large and handles config resolution, provider selection, adapter wiring, document loading, and execution in one place.

**GitHub Copilot: ✓.** `ghcopilot/src/main.rs:1-28` is compact, delegates config resolution to `ResolvedConfig::from_cli`, and delegates wiring to `App::bootstrap`.

**Claude: ✗.** `claude/src/main.rs:25-143` contains environment reads, file checks, config merge logic, provider construction, registry lookup, processor creation, and execution, so it is far from a thin entry point.

## Feature Scorecard

| Criterion | Codex | Grok | GitHub Copilot | Claude |
| --- | --- | --- | --- | --- |
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

1. **GitHub Copilot** is the best production starting point because it is the only implementation that consistently combines parse-safe CLI handling, reporter/progress ports, proper middleware composition, robust chunking, UTF-16-aware decoding, and a thin entry point.
2. **Codex** is second because it is simpler and has a valuable cache-reuse test, but it still mixes too much I/O into the use-case and lacks UTF-16 support.
3. **Grok** is third because it gets atomic cache writes and a domain-level registry right, but it fails the config precedence requirement, lacks a real text-processing port, and has a no-op cleanup bug.
4. **Claude** is fourth because it keeps too much logic in `main`, uses lossy text decoding, and does not demonstrate the test coverage or CLI ergonomics required by the spec.

## Ideal Merge

- Start from **GitHub Copilot** as the base.
- Cherry-pick **Grok**’s atomic cache write pattern from `grok/src/infrastructure/cache/mp3_cache.rs:87-99`.
- Cherry-pick **Codex**’s cache-reuse test idea from `codex/src/application/text_to_speech.rs:300-338`.
- Keep **GitHub Copilot**’s `Reporter`, `ProgressHandle`, processor chain, chunker, and UTF-16 reader.
- Move or abstract `FileReaderRegistry` so the application layer does not import infrastructure directly.
- Make cache writes atomic in GitHub Copilot by replacing direct `fs::write` at `ghcopilot/src/application/text_to_speech.rs:104-106`.
- Fix GitHub Copilot’s speed precedence so explicit `--speed 1.0` is distinguishable from omission.
