## ADDED Requirements

### Requirement: Standard OpenAI Provider
The canonical app SHALL support standard OpenAI TTS using `async-openai`.

#### Scenario: Standard provider uses OpenAI credentials
- **WHEN** standard OpenAI mode is selected
- **THEN** the provider uses `OPENAI_API_KEY` or configured OpenAI credentials and builds speech requests with selected voice, model, and speed

### Requirement: Azure OpenAI Provider
The canonical app SHALL support Azure OpenAI TTS using `async-openai`.

#### Scenario: Azure provider uses Azure credentials
- **WHEN** Azure mode is selected
- **THEN** the provider uses Azure endpoint, API key, deployment, and API version from resolved configuration

#### Scenario: Azure API version is configurable
- **WHEN** `AZURE_OPENAI_API_VERSION` or config API version is set
- **THEN** the Azure provider uses that API version

### Requirement: Azure Auto-Detection
The canonical app SHALL auto-select Azure mode when OpenAI credentials are absent and Azure endpoint configuration is present.

#### Scenario: Azure endpoint without OpenAI key selects Azure
- **WHEN** `OPENAI_API_KEY` is absent and `AZURE_OPENAI_ENDPOINT` is present
- **THEN** provider resolution selects Azure mode without requiring `--azure`

#### Scenario: OpenAI key prevents accidental Azure selection
- **WHEN** `OPENAI_API_KEY` and `AZURE_OPENAI_ENDPOINT` are both present and `--azure` is absent
- **THEN** provider resolution selects standard OpenAI mode unless config explicitly forces Azure

### Requirement: No Secret Output
The canonical app MUST NOT print API keys or secret-bearing environment values.

#### Scenario: Verbose output hides secrets
- **WHEN** verbose mode is enabled
- **THEN** output may report whether credentials are present but does not print credential values
