## Context

TxtTattler currently has multiple Rust implementations with overlapping strengths. The canonical implementation will live at `{workspace}/txttattler` and begin as a copy of `ghcopilot/`, because that version has the best baseline architecture: parse-safe CLI, reporter/progress ports, named text processor middleware, OpenAI/Azure config resolution, UTF-16-capable text reading, robust chunking, and the broadest tests.

The consolidation should turn that baseline into the single app for future development. The change is cross-cutting: it touches CLI resolution, configuration, cache storage, file reading, text processing, provider construction, reporting, tests, and project metadata.

## Goals / Non-Goals

**Goals:**

- Make `{workspace}/txttattler` the canonical app root.
- Preserve `ghcopilot/` as the architectural baseline.
- Add atomic cache and output writes.
- Fix configuration precedence, especially explicit default CLI values.
- Keep output behind reporter/progress ports.
- Keep the use-case layer testable without stdout capture or real OpenAI calls.
- Add no-op reporting for tests.
- Add missing UTF-16, cache-key, cache-reuse, and precedence tests.
- Keep `main.rs` thin and delegate bootstrap/resolution to library code.

**Non-Goals:**

- Implementing DOCX or PDF extraction.
- Streaming audio playback before all chunks are synthesized.
- Parallel TTS generation.
- Replacing `async-openai`.
- Rewriting the app around a different baseline than `ghcopilot/`.

## Decisions

### Use `ghcopilot/` As The Baseline

Use `ghcopilot/` as the copied starting point because it already has:

- `Reporter` and `ProgressHandle` ports.
- `TextProcessor::name()` and a processor chain.
- `.env` loading from CWD.
- OpenAI/Azure runtime config types.
- UTF-16LE/BE text decoding.
- Paragraph/sentence/word/character chunking.
- CLI and unit tests.

Alternative considered: start from `codex-gpt55-high/`, which has better atomic-write primitives. That implementation has weaker reporter/middleware architecture and config bugs, so its atomic write helper should be cherry-picked rather than using it as the base.

### Resolve CLI And Config With `Option<T>` For User-Supplied Defaults

CLI arguments that can also be configured should use `Option<T>` internally. Defaults should be applied only after merging sources.

Precedence:

1. CLI argument, if supplied
2. environment variable, if supported for that setting
3. config file
4. hard default

This fixes the case where a user explicitly passes `--speed 1.0` and expects it to override `speed = 1.5` from config.

Alternative considered: compare concrete values to defaults (`if cli.speed != 1.0`). This is ambiguous and already caused bugs in several implementations.

### Keep Reporting Behind Ports

Normal and verbose console feedback should be emitted through the existing reporter/progress abstractions. Add a `SilentReporter`/`NoopReporter` that implements the same traits for tests.

Alternative considered: direct `println!` in the use-case. This is simpler but makes tests noisy and requires stdout capture.

### Add Shared Atomic Write Utility

Create a shared atomic write helper for both cache writes and user-visible `--output` writes:

1. Create parent directory.
2. Write to a temporary file in the same directory.
3. Flush/sync where practical.
4. Rename to final path.

Use same-directory rename to preserve atomicity on supported filesystems.

Alternative considered: direct `fs::write`. This is shorter but can leave corrupt MP3s after crashes or interruptions.

### Keep Cache Keys Content-Based And Pipeline-Aware

Cache keys should be SHA-256 over:

- processed text
- voice
- model
- speed
- processor pipeline version

Fields must be separated by null bytes. The pipeline version should include the stable `name()` of each active processor, plus a top-level cache/schema version string.

Alternative considered: use `|` delimiters or filename-based cache keys. Delimiters can collide with text, and filename keys miss content changes.

### Abstract File Reading From The Use-Case

The use-case should not import `infrastructure::file_readers::FileReaderRegistry` directly. Either move the registry to `domain::ports` or introduce a domain port such as `DocumentReader` that the infrastructure registry implements.

Preferred approach: define a `DocumentReader` port in `domain::ports` with `read(&Path) -> Result<String>`, and keep the concrete registry in infrastructure. This preserves hexagonal boundaries while allowing the registry implementation to stay adapter-oriented.

Alternative considered: move the whole registry into domain. This matches one earlier interpretation of the spec but forces extension-map implementation details into the domain layer.

### Preserve Provider Runtime Config

Keep provider selection driven by `ResolvedConfig`:

- standard OpenAI when `OPENAI_API_KEY` is present and Azure is not forced
- Azure when `--azure`, config/env Azure flag, or `OPENAI_API_KEY` absent plus `AZURE_OPENAI_ENDPOINT` present
- configurable Azure API version
- support OpenAI base URL, org ID, and project ID

The existing boxed `async_openai::config::Config` approach is acceptable. Claude's enum provider pattern may be used only if it simplifies the code without losing config support.

## Risks / Trade-offs

- **Risk: copying `ghcopilot/` into `{workspace}/txttattler` may conflict with existing directories.** → Verify target directory state before copying; do not delete non-canonical implementation directories without explicit approval.
- **Risk: atomic rename behavior can vary on Windows when replacing existing files.** → Use a tested helper and include replacement tests; remove existing temp files on failure where possible.
- **Risk: config precedence tests may need controlled environment mutation.** → Isolate env-var tests with a serial test guard or environment helper to avoid cross-test contamination.
- **Risk: cache keys can become stale if processor behavior changes without version changes.** → Include processor names and a top-level cache version constant; bump the constant when processor semantics change.
- **Risk: strict encoding errors may reject files users expect to read.** → Keep errors clear and actionable; do not silently replace invalid bytes in the canonical text reader.
- **Risk: adding more tests around the real binary may require credentials accidentally.** → Ensure CLI/list/config tests do not instantiate providers or require OpenAI keys unless explicitly marked integration/manual.

## Migration Plan

1. Copy `ghcopilot/` to `{workspace}/txttattler`.
2. Verify the copied app builds and tests before modifications.
3. Apply structural fixes:
   - thin `main.rs`
   - no-op reporter
   - registry/use-case boundary
4. Apply config fixes:
   - `Option<f32>` speed
   - explicit-default precedence tests
   - `.env` manifest-dir fallback
5. Apply cache fixes:
   - shared atomic write helper
   - atomic cache writes
   - atomic output writes
   - cache hit output copy
6. Apply text/CLI polish:
   - richer voice descriptions
   - UTF-16 tests
   - cache-key sensitivity tests
7. Run verification:
   - `cargo fmt --check`
   - `cargo check`
   - `cargo test`
   - `cargo run -- --help`
   - `cargo run -- --list-voices`

Rollback strategy: keep source implementation directories untouched until the canonical app passes all verification. If consolidation fails, remove or reset only `{workspace}/txttattler` and recopy `ghcopilot/`.

## Open Questions

- Should the canonical cache directory use `directories::ProjectDirs` exactly as `ghcopilot/` does, or `dirs::cache_dir()` for consistency with other implementations?
- Should `--list-voices` show only supported OpenAI voices or include future/provider-specific voices once Azure deployments differ?
- Should markdown stripping remain config-only, or should it also have a CLI flag in this consolidation?
