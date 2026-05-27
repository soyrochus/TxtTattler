## 1. Canonical App Setup

- [x] 1.1 Verify whether `{workspace}/txttattler` already exists and avoid deleting any candidate implementation directories without explicit approval
- [x] 1.2 Copy `ghcopilot/` to `{workspace}/txttattler` as the canonical baseline
- [x] 1.3 Run baseline verification in `{workspace}/txttattler`: `cargo fmt --check`, `cargo check`, and `cargo test`
- [x] 1.4 Update package metadata in `Cargo.toml` to include edition, description, license, authors, repository, readme, keywords, and categories
- [x] 1.5 Review dependency sections and move test-only crates to `[dev-dependencies]`

## 2. CLI And Configuration

- [x] 2.1 Change configurable CLI fields that need explicit-default detection, especially `--speed`, to use `Option<T>` internally
- [x] 2.2 Preserve parse-time validation for `--voice`, `--model`, and `--speed`
- [x] 2.3 Implement config precedence as CLI, then supported environment variable, then config file, then hard default
- [x] 2.4 Add `.env` loading from current working directory before config resolution
- [x] 2.5 Add fallback `.env` loading from `{CARGO_MANIFEST_DIR}/.env` when CWD `.env` is absent
- [x] 2.6 Ensure invalid `.env` syntax reports the path and missing `.env` is not an error
- [x] 2.7 Preserve `--list-voices` behavior without requiring `FILE` or provider credentials
- [x] 2.8 Add Grok-style friendly descriptions to `--list-voices`

## 3. Architecture Boundaries

- [x] 3.1 Keep `src/main.rs` under 50 lines by delegating bootstrap and config resolution to library/application code
- [x] 3.2 Add a `DocumentReader` or equivalent domain port so the use-case no longer imports infrastructure file-reader registry types
- [x] 3.3 Update the concrete file-reader registry to implement the new domain reader port
- [x] 3.4 Ensure the use-case imports domain ports/entities rather than concrete OpenAI, rodio, or infrastructure file-reader types
- [x] 3.5 Keep provider construction in bootstrap/adapters rather than in `main.rs` or the use-case

## 4. Reporting And Testability

- [x] 4.1 Preserve the `Reporter` and `ProgressHandle` ports for phase and progress output
- [x] 4.2 Add a `SilentReporter` or `NoopReporter` implementation for tests
- [x] 4.3 Ensure the text-to-speech use-case has no direct `println!` or `eprintln!` phase output
- [x] 4.4 Ensure normal console output reports read, cleanup, cache check, synthesis, write/copy, playback, and completion phases
- [x] 4.5 Ensure multi-chunk synthesis reports progress through the progress handle abstraction
- [x] 4.6 Return a structured speech report containing input, output, cache, chunk, character, voice, model, speed, playback, and cache-hit metadata

## 5. Audio Cache And Output Writes

- [x] 5.1 Implement a shared atomic write helper that writes a temp file in the target directory and renames it into place
- [x] 5.2 Replace direct cache writes with the shared atomic write helper
- [x] 5.3 Replace direct `--output` writes with the shared atomic write helper
- [x] 5.4 Preserve or implement cache hit output copying from cached MP3 when possible
- [x] 5.5 Ensure cache keys are SHA-256 over processed text, voice, model, speed, and processor pipeline version
- [x] 5.6 Ensure cache-key fields use null-byte separators
- [x] 5.7 Ensure `--refresh` ignores existing cache and atomically replaces the entry
- [x] 5.8 Ensure `--no-cache` skips both cache reads and cache writes
- [x] 5.9 Add early failure for `--no-play --no-cache` without `--output`

## 6. Text Ingestion, Processing, And Chunking

- [x] 6.1 Preserve plain UTF-8 and UTF-8 BOM text reading
- [x] 6.2 Preserve UTF-16LE and UTF-16BE BOM decoding
- [x] 6.3 Ensure invalid text encoding returns a user-visible error rather than silently replacing bytes
- [x] 6.4 Preserve named `TextProcessor` middleware and processor-chain execution
- [x] 6.5 Ensure active processor names feed the cache pipeline version
- [x] 6.6 Preserve optional markdown stripping via configuration
- [x] 6.7 Preserve paragraph, sentence, word, and character fallback chunking
- [x] 6.8 Ensure sentence punctuation remains attached to emitted chunks

## 7. OpenAI And Azure Providers

- [x] 7.1 Preserve standard OpenAI provider construction with API key, optional base URL, org ID, and project ID
- [x] 7.2 Preserve Azure provider construction with endpoint, API key, deployment, and configurable API version
- [x] 7.3 Implement Azure auto-detection only when OpenAI key is absent and Azure endpoint is present, unless Azure is explicitly forced
- [x] 7.4 Ensure provider request construction uses selected voice, model, and speed
- [x] 7.5 Ensure normal and verbose output never prints API keys or secret-bearing values

## 8. Tests

- [x] 8.1 Add CLI tests for `--list-voices`, invalid voice, invalid model, invalid speed, and `-V/--verbose`
- [x] 8.2 Add config tests for CWD `.env`, manifest-dir `.env` fallback, env-over-config credentials, CLI-over-config values, and explicit default overrides
- [x] 8.3 Add provider resolution tests for standard OpenAI, forced Azure, Azure auto-detection, and OpenAI-key-prevents-accidental-Azure cases
- [x] 8.4 Add text reader tests for plain UTF-8, UTF-8 BOM, UTF-16LE BOM, UTF-16BE BOM, and invalid encoding errors
- [x] 8.5 Add chunker tests for short input, long input, all chunks under limit, punctuation preservation, and long-word character fallback
- [x] 8.6 Add text processor tests for whitespace collapse, blank-line reduction, markdown stripping, and pipeline-version construction
- [x] 8.7 Add cache-key sensitivity tests for text, voice, model, speed, and pipeline version changes
- [x] 8.8 Add a cache-reuse test that runs the use-case twice with a counting fake TTS provider and asserts one provider call
- [x] 8.9 Add cache behavior tests for `--refresh`, `--no-cache`, cache hit output copy, and atomic replacement
- [x] 8.10 Add reporter tests proving the silent reporter allows use-case execution without stdout capture

## 9. Documentation And Final Verification

- [x] 9.1 Update README usage examples for cache reuse, `--refresh`, `--no-cache`, `.env`, and Azure auto-detection
- [x] 9.2 Document the canonical architecture and how to add future file readers without changing the use-case
- [x] 9.3 Run `cargo fmt --check` in `{workspace}/txttattler`
- [x] 9.4 Run `cargo check` in `{workspace}/txttattler`
- [x] 9.5 Run `cargo test` in `{workspace}/txttattler`
- [x] 9.6 Run `cargo run -- --help` in `{workspace}/txttattler`
- [x] 9.7 Run `cargo run -- --list-voices` in `{workspace}/txttattler`
- [ ] 9.8 Confirm a real OpenAI TTS run works with `.env` credentials when the user chooses to test it
- [ ] 9.9 Confirm a second identical run reuses cache and performs no OpenAI call
