## ADDED Requirements

### Requirement: --instructions CLI option accepted
The system SHALL provide a `--instructions <TEXT>` CLI option that accepts a free-text string.

#### Scenario: Instructions flag accepted at parse time
- **WHEN** the user runs `txttattler file.txt --model gpt-4o-mini-tts --instructions "Speak in Dutch."`
- **THEN** the command parses without error and `ResolvedConfig.instructions` is `Some("Speak in Dutch.")`

#### Scenario: Instructions absent resolves to None
- **WHEN** `--instructions` is not passed and no env or config value is set
- **THEN** `ResolvedConfig.instructions` is `None`

### Requirement: instructions resolution follows CLI → env → config → None precedence
The system SHALL resolve the `instructions` value using the precedence: CLI flag overrides `TXT_TATTLER_INSTRUCTIONS` env var, which overrides `instructions` in the TOML config file, which defaults to `None`.

#### Scenario: CLI overrides environment variable
- **WHEN** `TXT_TATTLER_INSTRUCTIONS=env-value` is set and `--instructions cli-value` is passed
- **THEN** `ResolvedConfig.instructions` is `Some("cli-value")`

#### Scenario: Environment variable overrides config file
- **WHEN** `TXT_TATTLER_INSTRUCTIONS=env-value` is set and `instructions = "config-value"` is in the TOML config
- **THEN** `ResolvedConfig.instructions` is `Some("env-value")`

#### Scenario: Config file value used when CLI and env absent
- **WHEN** `instructions = "config-value"` is in the TOML config and neither CLI nor env var is set
- **THEN** `ResolvedConfig.instructions` is `Some("config-value")`

### Requirement: instructions is threaded to TtsRequest
The system SHALL include `instructions: Option<String>` on `TtsRequest` and populate it from `ResolvedConfig.instructions` in the use-case.

#### Scenario: Instructions value reaches the TTS provider
- **WHEN** `ResolvedConfig.instructions` is `Some("Speak in Dutch.")` and synthesis is requested
- **THEN** the `TtsRequest` passed to the provider has `instructions = Some("Speak in Dutch.")`

#### Scenario: None instructions reaches the TTS provider as None
- **WHEN** `ResolvedConfig.instructions` is `None`
- **THEN** the `TtsRequest` passed to the provider has `instructions = None`

### Requirement: instructions is sent to the API when Some
The OpenAI adapter SHALL include the `instructions` field in the API request when `TtsRequest.instructions` is `Some(_)`, and omit it when `None`.

#### Scenario: Instructions included in API request
- **WHEN** `TtsRequest.instructions` is `Some("Speak in British English.")`
- **THEN** the serialised API request body contains `"instructions": "Speak in British English."`

#### Scenario: Instructions omitted from API request when None
- **WHEN** `TtsRequest.instructions` is `None`
- **THEN** the serialised API request body does not contain an `instructions` key

### Requirement: Warning emitted when instructions used with tts-1 or tts-1-hd
The system SHALL emit a warning via the `Reporter` port when `instructions` is `Some(_)` and the resolved model is `tts-1` or `tts-1-hd`. The command SHALL NOT abort.

#### Scenario: Warning on tts-1 with instructions
- **WHEN** `--model tts-1 --instructions "Speak in Dutch."` is used
- **THEN** a warning is reported and synthesis completes successfully

#### Scenario: Warning on tts-1-hd with instructions
- **WHEN** `--model tts-1-hd --instructions "Speak in Dutch."` is used
- **THEN** a warning is reported and synthesis completes successfully

#### Scenario: No warning on gpt-4o-mini-tts with instructions
- **WHEN** `--model gpt-4o-mini-tts --instructions "Speak in Dutch."` is used
- **THEN** no instructions-related warning is reported

### Requirement: Warning emitted when speed != 1.0 used with gpt-4o-mini-tts
The system SHALL emit a warning via the `Reporter` port when the resolved speed is not `1.0` and the resolved model is `gpt-4o-mini-tts`. The command SHALL NOT abort.

#### Scenario: Warning on gpt-4o-mini-tts with non-default speed
- **WHEN** `--model gpt-4o-mini-tts --speed 1.5` is used
- **THEN** a speed warning is reported and synthesis completes successfully

#### Scenario: No speed warning on tts-1 with non-default speed
- **WHEN** `--model tts-1 --speed 1.5` is used
- **THEN** no speed warning is reported

### Requirement: Instructions shown in console output when set
The system SHALL include an `Instructions:` line in the synthesis summary when `instructions` is `Some(_)`. In normal mode, values longer than 80 characters SHALL be truncated with an ellipsis. The full value SHALL always be used in the API request and cache key.

#### Scenario: Instructions appear in summary
- **WHEN** `--instructions "Speak in Dutch."` is set and synthesis completes
- **THEN** the console output includes a line containing `Instructions:` and `"Speak in Dutch."`

#### Scenario: Instructions line absent when not set
- **WHEN** `--instructions` is not set
- **THEN** the console output does not contain an `Instructions:` line

### Requirement: Warnings are emitted via Reporter, not directly to stderr
The system SHALL route all instructions and speed compatibility warnings through the `Reporter` port so they can be suppressed or captured in tests.

#### Scenario: SilentReporter suppresses warnings
- **WHEN** a `SilentReporter` is used and a warning condition is triggered
- **THEN** no output is written to stderr or stdout
