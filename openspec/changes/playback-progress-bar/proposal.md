## Why

TxtTattler already reports reading, processing, caching, and speech generation clearly, but playback currently has only minimal console feedback. Adding live playback progress makes the final CLI phase feel complete without changing generation, caching, output, or provider behavior.

## What Changes

- Add live playback progress when generated audio is played through the CLI and total duration is known.
- Show playback status, elapsed time, total duration, a progress bar, and a clean completion message.
- Fall back gracefully to simple static playback messages when duration cannot be detected or output is non-interactive.
- Preserve `--no-play` behavior by suppressing playback and playback progress entirely.
- Keep terminal rendering outside the use-case layer and audio decoding/playback details inside infrastructure.

## Capabilities

### New Capabilities

- `playback-progress`: Console feedback for audio playback progress, duration display, fallback behavior, and completion reporting.

### Modified Capabilities

- None.

## Impact

- Affected target: canonical `txttattler/` Rust crate.
- Likely affected code: audio player adapter, reporter/progress abstractions, application wiring, and focused tests.
- Possible dependency impact: none expected beyond existing `rodio` and `indicatif`; duration detection should prefer the existing audio stack.
- No CLI flags, TTS provider behavior, cache semantics, MP3 output generation, or text processing behavior should change.
