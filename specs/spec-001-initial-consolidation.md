# Spec 001: Initial Consolidation

## Status

Draft

## Context

The canonical TxtTattler app will live at:

```text
{workspace}/txttattler
```

This app is assumed to start as a copy of the `ghcopilot/` implementation. The goal of this spec is to consolidate the strongest features and fixes identified across the other implementations:

- `codex/`
- `grok/`
- `codex-gpt55-high/`
- `claude/`
- `claude-opus/`

`ghcopilot/` remains the baseline because it currently has the strongest overall shape: parse-safe CLI, reporter/progress ports, named text processors, OpenAI/Azure config resolution, UTF-16-capable text reading, solid chunking, and the best test coverage.

## Goals

- Produce one canonical implementation with the best features from all candidate implementations.
- Preserve `ghcopilot/` strengths unless explicitly replaced by a better design.
- Fix known gaps before new feature work begins.
- Keep the canonical app cleanly testable, especially the use-case layer.
- Keep user-facing behavior friendly, explicit, and predictable.

## Non-Goals

- Do not implement DOCX or PDF extraction yet.
- Do not add streaming playback yet.
- Do not add parallel TTS generation yet.
- Do not change the public app name or binary name.
- Do not preserve every stylistic flourish from every implementation; only preserve behavior or architecture that improves the canonical app.

## Baseline To Preserve From `ghcopilot/`

### CLI

Preserve:

- `clap` v4 derive-based CLI.
- `txttattler [OPTIONS] <FILE>` entrypoint.
- `--list-voices` must exit cleanly without requiring `FILE`.
- Parse-time validation for:
  - `--voice`
  - `--model`
  - `--speed`
- Pretty help output with emoji/ASCII banner.
- Required flags:
  - `-v, --voice <VOICE>`
  - `-m, --model <MODEL>`
  - `-s, --speed <FLOAT>`
  - `-o, --output <PATH>`
  - `--no-play`
  - `--no-cache`
  - `--refresh`
  - `--cache-dir <PATH>`
  - `--azure`
  - `--config <PATH>`
  - `--list-voices`
  - `-V, --verbose`

### Configuration

Preserve:

- Central `ResolvedConfig` style.
- `.env` loading from current working directory before config resolution.
- Environment variables overriding config file values for provider credentials.
- Azure auto-detection when `OPENAI_API_KEY` is absent and `AZURE_OPENAI_ENDPOINT` is present.
- Configurable Azure API version.

### Reporting

Preserve:

- `Reporter` port.
- `ProgressHandle` port.
- Console implementation outside the use-case.
- Use-case testability without capturing stdout.

### Text Processing

Preserve:

- `TextProcessor` trait with `name()`.
- Composable processor chain.
- Pipeline version derived from active processor names.
- Optional markdown-stripping processor activated by config.

### Text Reading

Preserve:

- UTF-8 BOM support.
- UTF-16LE BOM support.
- UTF-16BE BOM support.
- User-visible errors for invalid encoding.

### Chunking

Preserve:

- Paragraph splitting.
- Sentence splitting with punctuation preserved.
- Word splitting.
- Character fallback.
- Tests asserting no chunk exceeds the limit.

## Features To Cherry-Pick Or Adapt

### From `grok/`

#### Atomic MP3 Cache Writes

Adopt Grok's atomic write behavior:

- Write MP3 data to a temporary file in the target directory.
- Rename the temporary file into the final cache path.
- Never leave a partially written cache entry at the final path after a crash or interruption.

Acceptance criteria:

- Cache writes use temp-file-and-rename.
- Temporary file name is deterministic enough for cleanup but collision-safe enough for concurrent runs.
- Failed writes do not corrupt an existing valid cache file.
- Tests cover replacing an existing cache entry.

#### Cache Hit Output Copy

On cache hit with `--output`, copy from the cached MP3 path when possible instead of regenerating or re-encoding.

Acceptance criteria:

- `txttattler file.txt --output out.mp3` copies from cache if the cache entry is valid.
- No OpenAI call is made on cache hit.
- The output path is created with contextual errors if parent directories fail.

#### Voice List Personality

Adapt Grok's richer `--list-voices` descriptions, but keep `ghcopilot`'s parse-safe CLI.

Acceptance criteria:

- `--list-voices` prints every valid voice.
- Each voice has a short friendly description.
- The command exits without reading config credentials or requiring a file.

### From `codex/`

#### `.env` Fallback To Crate Directory

Preserve `ghcopilot`'s current-working-directory `.env` loading, and add Codex's fallback behavior for development workflows where Cargo is run from the workspace root:

1. Try `{cwd}/.env`.
2. If not found, try `{CARGO_MANIFEST_DIR}/.env`.

Acceptance criteria:

- Running inside `{workspace}/txttattler` loads `{workspace}/txttattler/.env`.
- Running from `{workspace}` with `--manifest-path txttattler/Cargo.toml` still loads `{workspace}/txttattler/.env`.
- Missing `.env` is not an error.
- Invalid `.env` syntax is reported with the file path.

#### Cache Reuse Test Pattern

Adapt Codex's integration-style cache test:

- Run the use-case twice with identical input and options.
- Inject a counting fake TTS provider.
- Assert the second run is a cache hit.
- Assert the fake TTS provider was called exactly once.

Acceptance criteria:

- The test does not call OpenAI.
- The test does not capture stdout.
- The test uses a temporary cache directory.

#### Null-Byte Cache Separators

Keep null-byte cache-key separators. This is already present in `ghcopilot/`, but it is also a Codex strength and must not regress to `|` delimiters.

Acceptance criteria:

- Cache-key fields are separated by `0x00` bytes.
- Cache key includes:
  - processed text
  - voice
  - model
  - speed
  - text-processing pipeline version
- Tests prove cache key changes when each field changes.

### From `codex-gpt55-high/`

#### Atomic Output Writes

Extend atomic-write behavior beyond the cache: user-visible `--output` writes should also be atomic.

Acceptance criteria:

- `--output` writes to a temp file and renames into place.
- Existing output files are not corrupted if writing fails.
- Parent directory creation errors include context.

#### Strong Dependency Hygiene

Adopt the dependency hygiene standard:

- Use current or near-current crate versions.
- Keep dev-only crates in `[dev-dependencies]`.
- Avoid unnecessary dependencies when standard formatting or existing crates suffice.
- Keep complete `Cargo.toml` metadata:
  - `edition`
  - `description`
  - `license`
  - `authors`
  - `repository`
  - `readme`
  - `keywords`
  - `categories`

Acceptance criteria:

- `cargo check` passes.
- `cargo test` passes.
- `cargo machete` or equivalent dependency review should find no obvious unused dependencies, if available.

#### Structured Speech Report

Return a structured outcome/report from the use-case and app layer.

Acceptance criteria:

The final report includes:

- input path
- output path, if any
- cache path, if caching was enabled
- cache hit/miss status
- chunk count
- character count
- voice
- model
- speed
- whether playback occurred

### From `claude/`

#### Simple Provider Enum Pattern

Consider Claude's `OpenAiTtsProvider::Standard` / `OpenAiTtsProvider::Azure` enum style only if it simplifies the existing `ghcopilot` provider without reducing configurability.

Acceptance criteria:

- Standard OpenAI and Azure OpenAI remain supported.
- Azure API version remains configurable.
- OpenAI org/project IDs remain supported.
- The provider remains easy to construct from `ResolvedConfig`.

This is optional. Do not refactor the provider solely for style.

### From `claude-opus/`

#### Thin `main.rs`

Adopt Claude Opus's entry-point cleanliness.

Acceptance criteria:

- `src/main.rs` should remain under 50 lines.
- `main.rs` should:
  - parse CLI
  - call a library/bootstrap function
  - print/report top-level fatal errors if needed
- No raw environment reads in `main.rs`.
- No provider construction in `main.rs`.
- No file I/O in `main.rs`.

#### Silent Reporter

Add a `SilentReporter` or `NoopReporter` implementation.

Acceptance criteria:

- Tests can run the full use-case without console output.
- The silent reporter implements the same `Reporter`/`ProgressHandle` ports as the console reporter.
- The silent reporter is used in unit/integration tests where output is irrelevant.

#### Entity Parsing And Tests

Adopt the stronger entity parsing discipline:

- `Voice` implements `Display`.
- `Voice` implements `FromStr`.
- `Voice` has a `const ALL`.
- `SpeechModel`/`Model` implements `Display`.
- `SpeechModel`/`Model` implements `FromStr`.
- `SpeechModel`/`Model` has a `const ALL`.

Acceptance criteria:

- Parsing is case-insensitive where appropriate.
- Tests cover valid values, invalid values, and display round trips.
- CLI parse-time validation remains intact.

## Required Fixes To The `ghcopilot` Baseline

### Fix `--speed` Default Ambiguity

The canonical app must distinguish between:

- user did not pass `--speed`
- user explicitly passed `--speed 1.0`

Requirement:

- Change the CLI field from concrete `f32` to `Option<f32>`.
- Keep parse-time speed validation.
- Resolve precedence as:
  1. CLI value, if supplied
  2. environment variable, if supported
  3. config file value
  4. default `1.0`

Acceptance criteria:

- If config says `speed = 1.5` and CLI omits `--speed`, final speed is `1.5`.
- If config says `speed = 1.5` and CLI passes `--speed 1.0`, final speed is `1.0`.
- Invalid speed fails at parse time.

### Move Or Abstract `FileReaderRegistry`

The use-case should not import `infrastructure::file_readers::FileReaderRegistry` directly.

Acceptable solutions:

- Move `FileReaderRegistry` to `domain::ports`.
- Or define a `FileReaderRegistry`/`DocumentReader` port in `domain::ports` and keep the concrete registry in infrastructure.

Acceptance criteria:

- `application/text_to_speech.rs` imports only domain ports/entities and standard library/application-local code.
- Adding a future `.docx` reader does not require changes to the use-case.

### Atomic Cache And Output Writes

Replace direct `fs::write` cache/output writes with a shared atomic write helper.

Acceptance criteria:

- Cache writes are atomic.
- `--output` writes are atomic.
- Existing files survive failed writes.
- Tests cover write success and replacement behavior.

### UTF-16 Tests

`ghcopilot` supports UTF-16LE/BE but does not yet test it.

Acceptance criteria:

- Test UTF-16LE BOM decoding.
- Test UTF-16BE BOM decoding.
- Test invalid UTF-16 surfaces an error.

### Cache Key Sensitivity Tests

The cache key must change when any output-affecting value changes.

Acceptance criteria:

Tests prove the key changes when:

- text changes
- voice changes
- model changes
- speed changes
- processor pipeline version changes

### Config Precedence Tests

Add tests for the full precedence chain.

Acceptance criteria:

- Environment variables override config file credentials.
- CLI voice/model/speed override config values.
- Explicit CLI defaults override config values.
- Config values override hard defaults when CLI/env are absent.
- `.env` values are loaded before provider config resolution.

## Canonical Runtime Behavior

### Normal Run

Command:

```bash
txttattler notes.txt
```

Expected behavior:

1. Load `.env` from CWD or manifest directory if present.
2. Resolve config and credentials.
3. Read the file with the registered reader.
4. Run text processor chain.
5. Compute cache key from processed text and active output-affecting settings.
6. Reuse cached MP3 if valid.
7. Otherwise synthesize chunks sequentially.
8. Atomically write the cache entry.
9. Play the audio unless disabled.
10. Print a concise final report.

### Cache Hit

Command:

```bash
txttattler notes.txt
```

When the cache entry exists:

- no OpenAI call is made
- cache status is reported
- playback uses cached audio
- `--output`, if supplied, copies or atomically writes from cached audio

### Refresh

Command:

```bash
txttattler notes.txt --refresh
```

Expected behavior:

- ignore existing cache entry
- synthesize new audio
- atomically replace cache entry
- play or save according to flags

### No Cache

Command:

```bash
txttattler notes.txt --no-cache
```

Expected behavior:

- skip cache read
- skip cache write
- synthesize audio
- play or save according to flags

If combined with `--no-play` and no `--output`, the app should fail early with a helpful "nothing to do" error.

## Console Feedback Requirements

Normal output should be concise and friendly. It must include:

- file read phase
- text cleanup phase
- raw and processed character count
- selected voice/model/speed
- chunk/API request count
- cache hit/miss/refresh/disabled status
- synthesis progress for multi-chunk runs
- cache write path, when written
- output path, when written
- playback start/skipped status
- final success summary

Verbose output may include:

- config source details
- active processor names
- cache key prefix, never secrets
- per-chunk character counts
- elapsed timing per phase

Never print API keys or full secret-bearing environment values.

## Architecture Requirements

The canonical app should keep this shape:

```text
src/
├── main.rs
├── lib.rs
├── cli.rs
├── config.rs
├── adapters.rs
├── domain/
│   ├── entities.rs
│   └── ports.rs
├── application/
│   └── text_to_speech.rs
├── infrastructure/
│   ├── audio/
│   ├── cache/
│   ├── file_readers/
│   └── tts/
└── utils.rs
```

Use-case rules:

- No direct console output.
- No direct provider construction.
- No direct concrete audio player construction.
- No OpenAI-specific types.
- No hard-coded reader implementations.
- File/cache I/O is acceptable only if deliberately kept as application behavior; prefer ports for testability where the logic grows.

## Testing Requirements

Minimum required tests:

- CLI:
  - `--list-voices` succeeds without `FILE`
  - invalid voice fails at parse time
  - invalid model fails at parse time
  - invalid speed fails at parse time
  - `-V/--verbose` is accepted
- Config:
  - `.env` is loaded from CWD
  - manifest-dir `.env` fallback works
  - env overrides config
  - CLI overrides config, including explicit default values
  - Azure auto-detection condition
- Text reader:
  - plain UTF-8
  - UTF-8 BOM
  - UTF-16LE BOM
  - UTF-16BE BOM
  - invalid encoding error
- Chunker:
  - short input remains one chunk
  - long input produces more than one chunk
  - all chunks are under limit
  - sentence punctuation is preserved
  - long word falls back to character chunks
- Text processors:
  - whitespace collapse
  - blank-line reduction
  - markdown strip, if enabled
  - processor names feed pipeline version
- Cache:
  - key changes for text/voice/model/speed/pipeline version
  - second identical run is a cache hit
  - `--refresh` regenerates
  - `--no-cache` skips read and write
  - atomic write replacement preserves old file on simulated failure where practical
- Reporter:
  - `SilentReporter` produces no console output
  - use-case can run with fake TTS/audio/reporter

## Migration Plan

1. Copy `ghcopilot/` to `{workspace}/txttattler`.
2. Make `txttattler/` the only canonical app directory.
3. Apply the required baseline fixes:
   - `Option<f32>` speed
   - atomic writes
   - registry abstraction
   - UTF-16 tests
   - cache/config tests
4. Add cherry-picked features:
   - Grok atomic cache behavior and voice descriptions
   - Codex `.env` fallback and cache-reuse test
   - Codex GPT-5.5 High atomic output writes and structured report fields
   - Claude Opus thin `main.rs`, `SilentReporter`, and entity parsing tests
5. Run:

```bash
cargo fmt --check
cargo check
cargo test
cargo run -- --help
cargo run -- --list-voices
```

6. Remove or archive non-canonical implementation directories only after the canonical app passes all tests and the user approves cleanup.

## Acceptance Criteria For This Spec

The consolidation is complete when:

- `{workspace}/txttattler` builds and tests successfully.
- The canonical app includes all "Required Fixes" above.
- All minimum tests pass.
- The app still performs a real OpenAI TTS run with `.env` credentials.
- A second identical run reuses cache and performs no OpenAI call.
- Help output remains pretty and parse-safe.
- `main.rs` remains under 50 lines.
- No API keys or secrets are printed in normal or verbose output.
