## ADDED Requirements

### Requirement: Reporter Port
The canonical app SHALL route user-facing phase output through a reporter port.

#### Scenario: Use-case has no direct console output
- **WHEN** reviewing the text-to-speech use-case
- **THEN** it does not use direct `println!` or `eprintln!` calls for phase reporting

#### Scenario: Normal output includes phases
- **WHEN** the console reporter is used during a normal run
- **THEN** it reports file read, text cleanup, cache check, synthesis, write or copy, playback, and completion phases

### Requirement: Progress Handle Port
The canonical app SHALL route multi-chunk progress through a progress handle abstraction.

#### Scenario: Multi-chunk synthesis reports progress
- **WHEN** synthesis requires more than one chunk
- **THEN** the reporter creates a progress handle and updates per chunk

### Requirement: Silent Reporter
The canonical app SHALL provide a silent reporter for tests.

#### Scenario: Use-case test runs silently
- **WHEN** a test runs the use-case with the silent reporter
- **THEN** the test does not need to capture stdout or stderr

### Requirement: Fakeable Use-Case Dependencies
The canonical use-case SHALL accept fake TTS, audio, reader, cache, and reporter dependencies where practical.

#### Scenario: Cache hit test avoids OpenAI
- **WHEN** a test injects a counting fake TTS provider
- **THEN** it can assert cache reuse without calling OpenAI
