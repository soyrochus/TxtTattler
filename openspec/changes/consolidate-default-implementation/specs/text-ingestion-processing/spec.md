## ADDED Requirements

### Requirement: Text File Decoding
The canonical app SHALL decode supported plain text inputs without silent replacement.

#### Scenario: Plain UTF-8 reads successfully
- **WHEN** the input file contains valid UTF-8 without a BOM
- **THEN** the reader returns the expected text

#### Scenario: UTF-8 BOM is stripped
- **WHEN** the input file starts with a UTF-8 BOM
- **THEN** the BOM is removed from the returned text

#### Scenario: UTF-16LE BOM is decoded
- **WHEN** the input file starts with a UTF-16LE BOM
- **THEN** the reader returns decoded Unicode text

#### Scenario: UTF-16BE BOM is decoded
- **WHEN** the input file starts with a UTF-16BE BOM
- **THEN** the reader returns decoded Unicode text

#### Scenario: Invalid encoding errors visibly
- **WHEN** the input contains invalid data for the detected encoding
- **THEN** the reader returns a user-visible error rather than silently replacing bytes

### Requirement: Text Processor Middleware
The canonical app SHALL process text through a composable chain of named processors.

#### Scenario: Processor names form pipeline version
- **WHEN** the app computes the cache pipeline version
- **THEN** it includes the stable `name()` of every active text processor

#### Scenario: Markdown processor is externally activated
- **WHEN** markdown stripping is enabled by configuration
- **THEN** the markdown stripping processor is included in the processor chain

### Requirement: Text Chunking Hierarchy
The canonical app SHALL chunk processed text using paragraph, sentence, word, then character fallback boundaries.

#### Scenario: Long text stays under limit
- **WHEN** processed text exceeds the maximum TTS request size
- **THEN** every generated chunk has a character count less than or equal to the configured limit

#### Scenario: Sentence punctuation is preserved
- **WHEN** text is split at sentence boundaries
- **THEN** sentence-ending punctuation remains in the emitted chunks

#### Scenario: Long word falls back to character chunks
- **WHEN** a single word exceeds the maximum chunk size
- **THEN** the word is split into character chunks that do not exceed the limit
