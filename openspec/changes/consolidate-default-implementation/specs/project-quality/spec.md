## ADDED Requirements

### Requirement: Thin Entry Point
The canonical app SHALL keep `src/main.rs` under 50 lines and delegate bootstrap to library/application code.

#### Scenario: Main contains no raw provider wiring
- **WHEN** reviewing `src/main.rs`
- **THEN** provider construction, file reader registry construction, and config merging are delegated outside `main.rs`

### Requirement: Architecture Boundaries
The canonical app SHALL keep application use-cases independent from concrete infrastructure adapters.

#### Scenario: Use-case imports domain ports
- **WHEN** reviewing `application/text_to_speech.rs`
- **THEN** it imports domain ports/entities rather than concrete OpenAI, rodio, or infrastructure file-reader types

### Requirement: Dependency Hygiene
The canonical app SHALL keep dependencies current, purposeful, and correctly categorized.

#### Scenario: Dev dependencies are separated
- **WHEN** reviewing `Cargo.toml`
- **THEN** test-only crates such as `assert_cmd`, `predicates`, and `tempfile` are in `[dev-dependencies]`

#### Scenario: Package metadata is complete
- **WHEN** reviewing `Cargo.toml`
- **THEN** package metadata includes edition, description, license, authors, repository, readme, keywords, and categories

### Requirement: Verification Commands
The canonical app SHALL pass standard verification commands before consolidation is considered complete.

#### Scenario: Verification passes
- **WHEN** implementation is complete
- **THEN** `cargo fmt --check`, `cargo check`, `cargo test`, `cargo run -- --help`, and `cargo run -- --list-voices` all succeed

### Requirement: Minimum Test Coverage
The canonical app SHALL include tests covering CLI validation, config precedence, decoding, chunking, processors, cache behavior, reporter behavior, and provider mode resolution.

#### Scenario: Required tests exist
- **WHEN** the test suite is inspected
- **THEN** it contains tests for all minimum coverage areas defined by the consolidation spec
