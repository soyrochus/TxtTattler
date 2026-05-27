## 1. Reporter Contract

- [x] 1.1 Add playback progress reporting types to `txttattler/src/domain/ports.rs`, including a handle that can start, tick, finish, and support unknown-duration fallback.
- [x] 1.2 Implement playback reporting for `SilentReporter` as no-op behavior suitable for tests.
- [x] 1.3 Implement playback reporting for `ConsoleReporter` using `indicatif` for interactive known-duration playback and static messages for unknown or non-interactive playback.
- [x] 1.4 Add focused tests for playback time formatting and elapsed-time clamping.

## 2. Audio Playback Integration

- [x] 2.1 Update the `AudioPlayer` port so playback can report progress without exposing terminal or `rodio` details to the use-case layer.
- [x] 2.2 Update `TextToSpeechService` to pass the reporter into playback while keeping `--no-play` suppression unchanged.
- [x] 2.3 Update fake audio players and existing tests for the revised `AudioPlayer` contract.

## 3. Rodio Implementation

- [x] 3.1 Detect MP3 duration in `RodioAudioPlayer` using the existing decode path where possible.
- [x] 3.2 Fall back to unknown-duration reporting when duration cannot be determined without failing playback.
- [x] 3.3 Replace the blocking-only playback path with a playback loop that updates progress every 100-250 ms and avoids busy-waiting.
- [x] 3.4 Add playback error context so failures mention generated audio playback.

## 4. Verification

- [x] 4.1 Add tests proving silent or fake playback reporting can be used without terminal output or an audio device.
- [x] 4.2 Add tests for unknown-duration fallback behavior.
- [x] 4.3 Verify `--no-play` continues to suppress playback and playback progress.
- [x] 4.4 Run `cargo test --manifest-path txttattler/Cargo.toml --locked`.
