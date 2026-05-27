## ADDED Requirements

### Requirement: Playback progress is shown for known-duration audio
When CLI playback is enabled and the generated MP3 duration is known, the system SHALL display live playback progress during audio playback.

#### Scenario: Playback starts with known duration
- **WHEN** `txttattler` plays generated audio and the total duration is known
- **THEN** the CLI displays playback status, elapsed time, total duration, and a progress bar

#### Scenario: Playback progress updates in place
- **WHEN** audio playback is active with known duration
- **THEN** the progress display updates in place without printing a new line for each update

### Requirement: Playback time display is formatted and bounded
The system SHALL format elapsed and total playback time as `MM:SS` for durations under one hour and `HH:MM:SS` for durations of one hour or longer.

#### Scenario: Playback duration is under one hour
- **WHEN** playback time is less than one hour
- **THEN** the displayed time uses `MM:SS` formatting

#### Scenario: Playback duration is at least one hour
- **WHEN** playback time is one hour or longer
- **THEN** the displayed time uses `HH:MM:SS` formatting

#### Scenario: Elapsed display reaches total duration
- **WHEN** wall-clock elapsed playback time exceeds the known total duration
- **THEN** the displayed elapsed time is clamped to the total duration

### Requirement: Playback completion is reported clearly
When playback finishes successfully, the system SHALL complete the progress display and show a clear playback completion message.

#### Scenario: Known-duration playback completes
- **WHEN** known-duration playback finishes successfully
- **THEN** the progress bar reaches 100% and the CLI reports playback completion with the final duration

#### Scenario: Playback fails
- **WHEN** audio playback fails
- **THEN** the error includes context that playback of generated audio failed

### Requirement: Unknown duration falls back gracefully
If the MP3 duration cannot be determined before playback, the system SHALL still play audio and SHALL use simple playback feedback instead of live progress.

#### Scenario: Duration detection returns unknown
- **WHEN** playback starts and the total duration is unknown
- **THEN** the CLI shows a simple static playback status and audio playback continues

#### Scenario: Duration detection cannot parse metadata
- **WHEN** duration metadata is unsupported or unavailable
- **THEN** duration detection does not fail the command by itself

### Requirement: Playback progress respects disabled and non-interactive modes
The system SHALL suppress dynamic playback progress when playback is disabled or when output is non-interactive.

#### Scenario: Playback is disabled
- **WHEN** the user runs with `--no-play`
- **THEN** no playback progress or playback status is shown

#### Scenario: Output is non-interactive
- **WHEN** playback occurs while CLI output is redirected, logged, or otherwise non-interactive
- **THEN** the system avoids dynamic progress-bar output and uses simple readable messages

### Requirement: Playback progress preserves architecture boundaries
The playback progress implementation MUST keep terminal rendering and audio-stack details out of the use-case layer.

#### Scenario: Use-case invokes playback
- **WHEN** the use-case layer requests audio playback
- **THEN** it does not import `indicatif`, `rodio`, terminal-specific logic, or timer-loop rendering logic

#### Scenario: Reporter is replaced in tests
- **WHEN** playback is exercised with a silent or fake reporter
- **THEN** playback progress reporting can be disabled or observed without requiring terminal output
