## ADDED Requirements

### Requirement: Parse-Safe Pretty CLI
The canonical app SHALL provide a `clap` v4 derive-based CLI with pretty help output, emoji or ASCII banner, and parse-time validation for constrained options.

#### Scenario: Help is pretty
- **WHEN** the user runs `txttattler --help`
- **THEN** the help output includes a branded TxtTattler banner and lists all supported options

#### Scenario: Invalid constrained values fail at parse time
- **WHEN** the user passes an invalid voice, model, or speed
- **THEN** argument parsing fails before file I/O, config resolution, or OpenAI provider construction

### Requirement: Required CLI Options
The canonical app SHALL support the required flags `-v/--voice`, `-m/--model`, `-s/--speed`, `-o/--output`, `--no-play`, `--no-cache`, `--refresh`, `--cache-dir`, `--azure`, `--config`, `--list-voices`, and `-V/--verbose`.

#### Scenario: Verbose short flag is accepted
- **WHEN** the user runs `txttattler notes.txt -V`
- **THEN** the CLI accepts `-V` as verbose mode rather than treating it as version

### Requirement: List Voices Without File
The canonical app SHALL allow `--list-voices` to exit successfully without requiring a `FILE` argument or provider credentials.

#### Scenario: List voices exits cleanly
- **WHEN** the user runs `txttattler --list-voices`
- **THEN** the app prints every supported voice with a friendly description and exits successfully

### Requirement: Configuration Precedence
The canonical app SHALL resolve configurable values with precedence: CLI argument, then environment variable where supported, then config file, then hard default.

#### Scenario: Config overrides hard default
- **WHEN** config sets `speed = 1.5` and the CLI omits `--speed`
- **THEN** final speed is `1.5`

#### Scenario: Explicit CLI default overrides config
- **WHEN** config sets `speed = 1.5` and the CLI passes `--speed 1.0`
- **THEN** final speed is `1.0`

#### Scenario: Environment overrides config credentials
- **WHEN** config contains an OpenAI API key and `OPENAI_API_KEY` is set in the environment
- **THEN** the environment value is used for provider configuration

### Requirement: Dotenv Loading
The canonical app SHALL load environment variables from `.env` before provider configuration.

#### Scenario: CWD dotenv is loaded
- **WHEN** `{cwd}/.env` exists
- **THEN** values from that file are available during config resolution

#### Scenario: Manifest dotenv fallback is loaded
- **WHEN** `{cwd}/.env` is absent and `{CARGO_MANIFEST_DIR}/.env` exists
- **THEN** values from `{CARGO_MANIFEST_DIR}/.env` are available during config resolution

#### Scenario: Missing dotenv is not an error
- **WHEN** no `.env` file exists
- **THEN** the app continues using existing environment variables, config file values, and defaults
