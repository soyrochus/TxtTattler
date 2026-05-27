## Why

TxtTattler has several independent Rust implementations with useful but fragmented features. We need one canonical implementation at `{workspace}/txttattler`, based on `ghcopilot/`, that consolidates the strongest CLI, caching, configuration, text-processing, provider, reporting, and testability behaviors before future feature work continues.

## What Changes

- Establish `{workspace}/txttattler` as the canonical application, starting from a copy of `ghcopilot/`.
- Preserve the best baseline behavior from `ghcopilot/`: parse-safe pretty CLI, `Reporter`/`ProgressHandle` ports, named text processor chain, `.env` loading, OpenAI/Azure config resolution, UTF-16-aware text reading, robust chunking, and strong tests.
- Add atomic cache writes from `grok/` and atomic `--output` writes from `codex-gpt55-high/`.
- Add Codex-style `.env` fallback to `{CARGO_MANIFEST_DIR}/.env` and a cache-reuse integration test pattern.
- Add Grok-style friendly voice descriptions for `--list-voices`.
- Add Claude Opus-style thin `main.rs`, no-op reporter, and stronger entity parsing tests.
- Fix known baseline gaps: `--speed` explicit-default ambiguity, direct infrastructure registry import in the use-case, missing UTF-16 tests, incomplete cache-key sensitivity tests, and incomplete configuration precedence tests.
- Keep DOCX/PDF extraction, streaming playback, and parallel TTS generation out of scope for this consolidation.

## Capabilities

### New Capabilities

- `canonical-runtime`: Defines the canonical app location, runtime flow, cache behavior, output behavior, and command semantics.
- `cli-configuration`: Defines parse-safe CLI behavior, `.env` loading, config precedence, provider credential resolution, and friendly voice listing.
- `audio-cache`: Defines content-addressed MP3 caching, cache hits, refresh, no-cache behavior, atomic writes, and output copying/writing.
- `text-ingestion-processing`: Defines text file decoding, text processing middleware, cache pipeline versioning, and chunking behavior.
- `tts-provider-integration`: Defines standard OpenAI and Azure OpenAI provider behavior, auto-detection, API version configuration, and request construction.
- `reporting-testability`: Defines reporter/progress ports, silent reporter behavior, structured run reports, and testability constraints.
- `project-quality`: Defines architecture boundaries, dependency hygiene, thin entry point, and required verification/tests.

### Modified Capabilities

- None. There are no existing OpenSpec capabilities in `openspec/specs/` to modify.

## Impact

- Affected code: `{workspace}/txttattler/src/**`, `{workspace}/txttattler/tests/**`, `{workspace}/txttattler/Cargo.toml`, `{workspace}/txttattler/README.md`.
- Affected architecture: CLI/config resolution, application use-case boundaries, file reader registry abstraction, cache writing, reporter/progress wiring, text processor pipeline, provider construction.
- Affected dependencies: may require keeping or adding `dotenvy`, `sha2`, `encoding_rs`, `directories` or equivalent, `assert_cmd`, `predicates`, and `tempfile` in appropriate dependency sections.
- Affected behavior: repeated runs should reuse cached MP3s by default, user-visible output files should be written atomically, `.env` should load from CWD or manifest directory, and explicit CLI defaults should override config values.
