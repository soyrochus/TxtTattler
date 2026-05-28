## ADDED Requirements

### Requirement: gpt-4o-mini-tts is accepted as a model value
The system SHALL accept `gpt-4o-mini-tts` as a valid value for the `--model` CLI option, the `TXT_TATTLER_MODEL` environment variable, and the `model` key in the TOML config file.

#### Scenario: CLI accepts gpt-4o-mini-tts at parse time
- **WHEN** the user runs `txttattler file.txt --model gpt-4o-mini-tts`
- **THEN** the command proceeds without a parse error

#### Scenario: Config file accepts gpt-4o-mini-tts
- **WHEN** `model = "gpt-4o-mini-tts"` is set in `txttattler.toml` and no `--model` flag is passed
- **THEN** `ResolvedConfig.model` is `SpeechModelName::Gpt4oMiniTts`

#### Scenario: Environment variable accepts gpt-4o-mini-tts
- **WHEN** `TXT_TATTLER_MODEL=gpt-4o-mini-tts` is set and no `--model` flag is passed
- **THEN** `ResolvedConfig.model` is `SpeechModelName::Gpt4oMiniTts`

### Requirement: gpt-4o-mini-tts model name parses case-insensitively
The system SHALL parse `gpt-4o-mini-tts` in a case-insensitive manner from all input sources.

#### Scenario: Mixed-case model string parses correctly
- **WHEN** `SpeechModelName::from_str("GPT-4O-MINI-TTS")` is called
- **THEN** the result is `Ok(SpeechModelName::Gpt4oMiniTts)`

### Requirement: gpt-4o-mini-tts displays correctly
The system SHALL render `SpeechModelName::Gpt4oMiniTts` as the string `"gpt-4o-mini-tts"` in all console output and structured reports.

#### Scenario: Display round-trip
- **WHEN** `SpeechModelName::Gpt4oMiniTts.to_string()` is called
- **THEN** the result is `"gpt-4o-mini-tts"`

### Requirement: SpeechModelName::ALL includes all three models
The constant `SpeechModelName::ALL` SHALL contain exactly three entries: `Tts1`, `Tts1Hd`, and `Gpt4oMiniTts`.

#### Scenario: ALL array length
- **WHEN** `SpeechModelName::ALL.len()` is evaluated
- **THEN** the result is `3`

### Requirement: gpt-4o-mini-tts is mapped to the correct async-openai type
The OpenAI adapter SHALL map `SpeechModelName::Gpt4oMiniTts` to `async_openai::types::audio::SpeechModel::Gpt4oMiniTts` when constructing the API request.

#### Scenario: Adapter sends correct model identifier
- **WHEN** a `TtsRequest` with `model = Gpt4oMiniTts` is synthesised
- **THEN** the API request contains the model identifier `"gpt-4o-mini-tts"`

### Requirement: Existing models continue to work
The addition of `gpt-4o-mini-tts` SHALL NOT change the behaviour of `tts-1` or `tts-1-hd`.

#### Scenario: tts-1 still accepted
- **WHEN** the user runs `txttattler file.txt --model tts-1`
- **THEN** the command proceeds and the API request uses model `"tts-1"`

#### Scenario: tts-1-hd still accepted
- **WHEN** the user runs `txttattler file.txt --model tts-1-hd`
- **THEN** the command proceeds and the API request uses model `"tts-1-hd"`
