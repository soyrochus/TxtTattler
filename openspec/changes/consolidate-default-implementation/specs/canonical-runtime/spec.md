## ADDED Requirements

### Requirement: Canonical App Location
The canonical TxtTattler implementation SHALL live at `{workspace}/txttattler` and SHALL start from a copy of the `ghcopilot/` implementation.

#### Scenario: Canonical root exists
- **WHEN** the consolidation is complete
- **THEN** `{workspace}/txttattler/Cargo.toml` and `{workspace}/txttattler/src/main.rs` exist

#### Scenario: Candidate implementations remain separate
- **WHEN** the canonical app is created
- **THEN** source implementation directories such as `codex/`, `grok/`, `ghcopilot/`, `claude/`, `codex-gpt55-high/`, and `claude-opus/` are not deleted without explicit approval

### Requirement: Normal Runtime Flow
The canonical app SHALL execute a normal text-to-speech run in the order: load environment, resolve config, read file, process text, compute cache key, reuse or generate MP3, write cache/output, play audio if enabled, and report completion.

#### Scenario: Normal run with cache miss
- **WHEN** the user runs `txttattler notes.txt` and no valid cache entry exists
- **THEN** the app reads and processes the file, synthesizes speech, writes a cache entry atomically, plays audio, and reports completion

#### Scenario: Normal run with cache hit
- **WHEN** the user runs `txttattler notes.txt` and a valid matching cache entry exists
- **THEN** the app reuses cached audio without making an OpenAI request

### Requirement: No-Op Run Guard
The canonical app MUST fail early with a helpful error when playback is disabled, output is absent, and caching is disabled.

#### Scenario: Nothing to do
- **WHEN** the user runs `txttattler notes.txt --no-play --no-cache` without `--output`
- **THEN** the app fails before reading credentials or calling OpenAI with a message explaining that playback, output, or caching must be enabled

### Requirement: Structured Speech Report
The canonical app SHALL return a structured run report from the app/use-case layer.

#### Scenario: Report includes run metadata
- **WHEN** a run completes
- **THEN** the report includes input path, optional output path, optional cache path, cache hit status, chunk count, character count, voice, model, speed, and whether playback occurred
