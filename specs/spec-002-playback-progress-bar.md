# Spec: Playback Progress Bar for TxtTattler

## 1. Purpose

Improve the playback phase of the consolidated `txttattler/` application by replacing the current minimal “playing audio” console feedback with a live playback progress display.

The goal is to make playback feel like a first-class phase of the CLI experience, comparable in quality to file reading, text processing, caching, and TTS generation.

This change does not affect speech generation, caching, OpenAI/Azure integration, text chunking, or MP3 output creation. It only improves console feedback while audio is being played.

## 2. Scope

### In scope

The application shall display live playback progress when audio is played through the CLI.

The progress display shall show:

| Element           | Description                                                  |
| ----------------- | ------------------------------------------------------------ |
| Playback status   | Indicates that the generated audio is currently being played |
| Elapsed time      | Time already played                                          |
| Total duration    | Full duration of the generated MP3                           |
| Progress bar      | Visual indication of playback progress                       |
| Completion status | Clear message when playback finishes                         |

Example output:

```text
Playing audio  ███████████░░░░░░░░░░░░░░░░  00:23 / 01:47
```

or, depending on existing `indicatif` style:

```text
🎧 Playing  [████████████░░░░░░░░░░░░░]  00:23 / 01:47
```

### Out of scope

The following are explicitly excluded from this change:

| Excluded feature             | Reason                                                                      |
| ---------------------------- | --------------------------------------------------------------------------- |
| Audio waveform visualization | Requires decoding/analyzing audio samples                                   |
| Sequencer animation          | Deferred to a later UX enhancement                                          |
| Real-time amplitude meter    | Requires access to audio samples or playback stream                         |
| Pause/resume controls        | Would change TxtTattler from a CLI playback tool into an interactive player |
| Seeking/skipping             | Same as above                                                               |
| Terminal full-screen UI      | Not needed for this phase                                                   |

## 3. Functional Requirements

### 3.1 Playback duration detection

The application shall determine the total duration of the MP3 before playback starts.

Acceptable approaches:

1. Use the existing audio playback stack if it can expose decoded duration.
2. Decode metadata or audio frames using the current `rodio`/decoder path.
3. If exact duration cannot be determined, fall back gracefully to the current minimal playback message.

The implementation must not fail the whole command only because duration detection fails. Playback should still occur.

Required behavior:

```text
If duration is known:
  show live progress bar

If duration is unknown:
  show simple playback status
```

Example fallback:

```text
🎧 Playing audio...
```

### 3.2 Elapsed-time tracking

During playback, the CLI shall update elapsed time based on wall-clock time from playback start.

Elapsed time shall be formatted as:

```text
MM:SS
```

For audio longer than one hour:

```text
HH:MM:SS
```

The elapsed time shall not exceed the known total duration in the display.

### 3.3 Progress bar behavior

The progress bar shall update periodically while playback is active.

Recommended update interval:

```text
100 ms to 250 ms
```

The progress bar shall complete when playback ends.

The progress bar must not print a new line for every update. It must update in place using the existing terminal progress mechanism, preferably `indicatif`.

### 3.4 Completion behavior

When playback finishes successfully, the application shall show a clean completion message.

Example:

```text
✓ Playback complete  01:47
```

If playback fails, the error shall include context:

```text
Failed to play generated audio: <reason>
```

If playback is skipped due to `--no-play`, no playback progress bar shall be shown.

### 3.5 Non-interactive terminal behavior

If the application detects that stdout is not an interactive terminal, it should avoid dynamic progress output.

In non-interactive mode, it may print a simple static message:

```text
Playing audio: 01:47
Playback complete.
```

This avoids broken output in logs, CI, redirected output, or scripted execution.

## 4. CLI Behavior

No new CLI flag is required for the first implementation.

The feature should be enabled automatically when:

```text
--no-play is not set
stdout/stderr supports interactive progress display
audio duration can be determined
```

Optional future flag, not required now:

```text
--no-playback-progress
```

This is not part of the current scope unless implementation complexity requires it.

## 5. Architectural Requirements

The playback progress feature must preserve the existing architecture.

The use-case layer must not directly manage terminal animation, timers, or progress bars.

Preferred structure:

```rust
pub trait AudioPlayer {
    fn play(&self, audio_path: &Path) -> Result<()>;
}
```

may evolve into something like:

```rust
pub trait AudioPlayer {
    fn play(&self, audio_path: &Path, reporter: &dyn PlaybackReporter) -> Result<()>;
}
```

or:

```rust
pub trait PlaybackReporter {
    fn playback_started(&self, duration: Option<Duration>);
    fn playback_progress(&self, elapsed: Duration, duration: Duration);
    fn playback_finished(&self, duration: Option<Duration>);
}
```

The exact shape can depend on the existing consolidated codebase, but the principles are mandatory:

| Requirement                               | Rule                                                          |
| ----------------------------------------- | ------------------------------------------------------------- |
| No direct console logic in use case       | Progress rendering belongs in reporter/adapter layer          |
| Audio playback remains an adapter concern | `rodio` or decoder-specific logic must stay in infrastructure |
| Testability preserved                     | A silent/no-op reporter must remain possible                  |
| Graceful fallback                         | Unknown duration must not break playback                      |

## 6. Suggested Implementation Design

### 6.1 Add playback progress abstraction

Introduce or extend a reporter interface with playback-specific methods.

Example:

```rust
pub trait PlaybackReporter: Send + Sync {
    fn playback_started(&self, duration: Option<Duration>);
    fn playback_tick(&self, elapsed: Duration, duration: Duration);
    fn playback_finished(&self, duration: Option<Duration>);
}
```

If the existing `Reporter` abstraction is already broad enough, these methods may be added there instead.

### 6.2 Implement console playback reporter

The console implementation should use `indicatif::ProgressBar`.

Expected behavior:

```rust
ProgressBar::new(duration_ms)
```

Position should be updated in milliseconds.

Template should include:

```text
{msg} {bar} {elapsed} / {duration}
```

Approximate style:

```text
🎧 Playing  [{bar:30}]  {elapsed_precise} / {duration_precise}
```

### 6.3 Duration extraction

Duration extraction should live in the audio infrastructure layer.

Example helper:

```rust
fn detect_audio_duration(path: &Path) -> Result<Option<Duration>>
```

Rules:

| Case                  | Behavior                                            |
| --------------------- | --------------------------------------------------- |
| Duration detected     | Use progress bar                                    |
| Duration not detected | Return `Ok(None)`                                   |
| File unreadable       | Return error only if playback itself cannot proceed |
| Unsupported metadata  | Fall back to simple playback                        |

### 6.4 Playback loop

The audio player should start playback and update the reporter while the sink is active.

Conceptual flow:

```text
1. Detect duration
2. Notify reporter: playback_started(duration)
3. Start rodio playback
4. While sink is not empty:
     calculate elapsed
     update reporter
     sleep 100–250 ms
5. Notify reporter: playback_finished(duration)
```

The loop must not busy-wait.

## 7. Acceptance Criteria

The implementation is complete when the following are true:

| ID    | Acceptance criterion                                                                                            |
| ----- | --------------------------------------------------------------------------------------------------------------- |
| AC-1  | When audio playback starts and duration is known, the CLI displays a live progress bar                          |
| AC-2  | The display includes elapsed time and total duration                                                            |
| AC-3  | The progress bar updates in place, not by printing repeated lines                                               |
| AC-4  | When playback completes, the progress bar reaches 100% and a completion message is shown                        |
| AC-5  | `--no-play` suppresses playback and therefore suppresses playback progress                                      |
| AC-6  | If duration cannot be detected, playback still works with a simple static message                               |
| AC-7  | The use-case layer does not import `indicatif`, `rodio`, terminal-specific logic, or timer-loop rendering logic |
| AC-8  | Existing tests continue to pass                                                                                 |
| AC-9  | At least one test verifies that playback progress reporting can be disabled or replaced by a silent reporter    |
| AC-10 | Non-interactive output does not produce broken progress-bar artifacts                                           |

## 8. Testing Requirements

### Unit tests

Add tests for:

| Test                             | Expected result                       |
| -------------------------------- | ------------------------------------- |
| Duration formatting under 1 hour | `00:03`, `01:25`, `59:59`             |
| Duration formatting over 1 hour  | `01:02:03`                            |
| Progress calculation             | elapsed cannot exceed total           |
| Unknown duration fallback        | reporter uses simple playback message |
| Silent reporter                  | no panic, no terminal output required |

### Integration-style test

Use a fake `AudioPlayer` or fake `PlaybackReporter` to simulate playback.

The test should verify:

```text
playback_started called once
playback_tick called at least once
playback_finished called once
```

The test must not require real speakers or an audio device.

## 9. Non-Functional Requirements

The feature must remain:

| Property       | Requirement                                              |
| -------------- | -------------------------------------------------------- |
| Cross-platform | Linux, macOS, Windows                                    |
| Lightweight    | No heavy terminal UI framework unless strictly necessary |
| Non-invasive   | No changes to TTS generation or caching semantics        |
| Robust         | Unknown duration must degrade gracefully                 |
| Testable       | No real audio device required for core tests             |
| Clean          | No direct console rendering in the application use-case  |


