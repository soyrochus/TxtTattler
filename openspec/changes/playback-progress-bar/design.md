## Context

The canonical implementation lives in `txttattler/`. Playback currently flows through `TextToSpeechService`, which reports a static playback-start status and then calls the `AudioPlayer` port. The concrete `RodioAudioPlayer` decodes MP3 bytes with `rodio`, appends the source to a player, and blocks with `sleep_until_end()`.

The project already has a domain-level `Reporter` abstraction, `ProgressHandle`, `ConsoleReporter` using `indicatif`, and `SilentReporter` for tests. The design should extend that pattern so the use-case layer can request playback while infrastructure owns decoding, timing, and audio-device behavior.

## Goals / Non-Goals

**Goals:**

- Display a live playback progress bar for known-duration MP3 playback in the CLI.
- Show elapsed time, total duration, and a clean completion message.
- Preserve graceful playback when duration cannot be detected.
- Avoid dynamic progress artifacts in non-interactive output.
- Keep `rodio`, `indicatif`, terminal rendering, and timer-loop behavior outside the use-case layer.
- Preserve testability with silent or fake reporters and no required real audio device for core tests.

**Non-Goals:**

- No new CLI flag in this change.
- No waveform, amplitude meter, sequencer animation, seeking, pause/resume, or full-screen terminal UI.
- No change to speech generation, caching, text chunking, provider selection, or MP3 output semantics.
- No requirement to make exact duration detection fatal.

## Decisions

1. Extend the existing reporter abstraction for playback progress.

   Add playback-specific reporting methods or a playback handle to the domain reporter layer, reusing `ConsoleReporter` and `SilentReporter`. A likely shape is a `PlaybackProgressHandle` with methods for `tick(elapsed, total)` and `finish(duration)`, returned by a `Reporter::playback(duration: Option<Duration>)` method.

   Rationale: The codebase already routes terminal feedback through `Reporter`, so extending that abstraction preserves the established architecture. A separate global terminal helper would duplicate this role and make test replacement harder.

2. Let the audio adapter own duration detection and the playback loop.

   Update the `AudioPlayer` port so playback can receive a reporter reference, for example `play_mp3(&self, audio: Vec<u8>, reporter: &dyn Reporter) -> Result<()>`, or a narrower playback reporter trait if that reads cleaner during implementation. The use case will still only decide whether playback is enabled.

   Rationale: Duration detection, `rodio` decoding, sink state, and sleep intervals are infrastructure concerns. The use case must not import `rodio`, `indicatif`, or timing-loop code.

3. Prefer existing `rodio` duration capabilities before adding dependencies.

   Attempt to obtain duration from the decoded source or equivalent existing audio-stack metadata. If duration is unavailable, notify the reporter with `None` and proceed with static playback feedback.

   Rationale: The change should stay lightweight and cross-platform. A new MP3 metadata parser is only justified if the existing stack cannot support the acceptance criteria.

4. Use elapsed wall-clock time for progress position.

   Once playback starts, the audio adapter records `Instant::now()`, updates progress every 100-250 ms while playback is active, clamps elapsed display to the known duration, and finishes at 100%.

   Rationale: Wall-clock elapsed time is simple, avoids sample-level inspection, and matches the requested scope.

5. Keep non-interactive output simple.

   `ConsoleReporter` should detect whether dynamic progress is appropriate, or otherwise use simple static messages. The silent reporter must remain no-op.

   Rationale: `indicatif` progress bars are useful in terminals but noisy in logs, CI, and redirected output.

## Risks / Trade-offs

- Duration detection may be unavailable for some decoded audio -> fall back to static playback status and still play audio.
- Clocks can drift slightly from actual audio-device playback -> clamp elapsed to duration and finish when the player reports completion.
- Passing a reporter into the audio player changes the `AudioPlayer` trait -> update test fakes and wiring in one focused pass.
- Interactive-terminal detection can vary by platform -> keep the default fallback readable and add tests around reporter behavior rather than platform-specific terminal internals.

## Migration Plan

Implement behind the existing playback path. Existing commands keep the same flags and behavior, with richer feedback only when playback occurs. Rollback is limited to restoring the previous `AudioPlayer::play_mp3(audio)` call and reporter methods if needed.

## Open Questions

- Whether to model playback reporting as new methods on `Reporter` or as a narrower `PlaybackReporter` trait implemented by the same concrete reporters.
- Whether `rodio`'s decoded source duration is reliable enough for generated OpenAI MP3 bytes across all supported platforms.
