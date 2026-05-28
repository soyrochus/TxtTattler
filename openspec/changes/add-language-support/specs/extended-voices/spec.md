## ADDED Requirements

### Requirement: Seven additional voices accepted by --voice
The system SHALL accept the following voices as valid values for `--voice`, the `TXT_TATTLER_VOICE` environment variable, and the `voice` key in the TOML config file: `ash`, `ballad`, `coral`, `sage`, `verse`, `marin`, `cedar`.

#### Scenario: ash accepted at parse time
- **WHEN** `--voice ash` is passed on the CLI
- **THEN** the command proceeds without a parse error

#### Scenario: cedar accepted at parse time
- **WHEN** `--voice cedar` is passed on the CLI
- **THEN** the command proceeds without a parse error

#### Scenario: All seven voices accepted
- **WHEN** each of `ash`, `ballad`, `coral`, `sage`, `verse`, `marin`, `cedar` is passed as `--voice`
- **THEN** each parses without error

### Requirement: Extended voices parse case-insensitively
The system SHALL parse the extended voice names in a case-insensitive manner from all input sources.

#### Scenario: Mixed-case voice string parses correctly
- **WHEN** `VoiceName::from_str("CORAL")` is called
- **THEN** the result is `Ok(VoiceName::Coral)`

### Requirement: VoiceName::ALL contains all thirteen voices
The constant `VoiceName::ALL` SHALL contain exactly thirteen entries: the existing six (`alloy`, `echo`, `fable`, `onyx`, `nova`, `shimmer`) plus the seven extended voices.

#### Scenario: ALL array length
- **WHEN** `VoiceName::ALL.len()` is evaluated
- **THEN** the result is `13`

### Requirement: Each extended voice has a description
The `VoiceName::description()` method SHALL return a non-empty description string for each of the seven extended voices.

#### Scenario: ash has description
- **WHEN** `VoiceName::Ash.description()` is called
- **THEN** a non-empty string is returned

#### Scenario: All extended voices have descriptions
- **WHEN** `description()` is called on each of the seven extended voices
- **THEN** each returns a non-empty string

### Requirement: Extended voices are mapped to async-openai Voice variants
The OpenAI adapter SHALL map each extended `VoiceName` variant to the corresponding `async_openai::types::audio::Voice` variant.

#### Scenario: ash maps to Voice::Ash
- **WHEN** a `TtsRequest` with `voice = VoiceName::Ash` is synthesised
- **THEN** the API request contains the voice identifier `"ash"`

#### Scenario: cedar maps to Voice::Cedar
- **WHEN** a `TtsRequest` with `voice = VoiceName::Cedar` is synthesised
- **THEN** the API request contains the voice identifier `"cedar"`

### Requirement: --list-voices shows all thirteen voices
The `--list-voices` command SHALL display all thirteen voices, including descriptions, and exit with code `0` without requiring a FILE argument or an API call.

#### Scenario: All thirteen voices listed
- **WHEN** `txttattler --list-voices` is run
- **THEN** the output contains all thirteen voice names and the exit code is `0`

#### Scenario: Extended voices appear in --list-voices
- **WHEN** `txttattler --list-voices` is run
- **THEN** the output contains `ash` and `cedar`

### Requirement: Extended voices usable with any model
The system SHALL NOT reject extended voices at parse time based on the selected model. Any incompatibility is surfaced by the API at runtime.

#### Scenario: ash accepted with tts-1
- **WHEN** `--voice ash --model tts-1` is passed
- **THEN** the command proceeds to synthesis without a parse or config error
