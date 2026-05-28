## ADDED Requirements

### Requirement: instructions field is included in the cache key
The system SHALL append the `instructions` value (or an empty string when `None`) as a null-byte-separated field at the end of the cache key material before hashing.

#### Scenario: Cache key includes instructions when set
- **WHEN** two requests differ only in `instructions` value
- **THEN** their cache keys are different

#### Scenario: Cache key with instructions=None uses empty string contribution
- **WHEN** `instructions` is `None`
- **THEN** the cache key is computed as if an empty string was appended after the final null-byte separator

### Requirement: Different instructions always produce different cache keys
The system SHALL produce a distinct cache key for every distinct `instructions` value, including the distinction between `None` and `Some("")`.

#### Scenario: None vs Some produce different keys
- **WHEN** one request has `instructions = None` and another has `instructions = Some("Speak in Dutch.")`
- **THEN** the two cache keys are different

#### Scenario: Two different instruction strings produce different keys
- **WHEN** one request has `instructions = Some("Speak in Dutch.")` and another has `instructions = Some("Speak in English.")`
- **THEN** the two cache keys are different

#### Scenario: Same instructions produce the same key
- **WHEN** two requests are identical in all fields including `instructions`
- **THEN** their cache keys are equal

### Requirement: A second run with the same instructions is a cache hit
The system SHALL reuse a cached MP3 on the second run when all synthesis parameters, including `instructions`, are identical.

#### Scenario: Cache hit on repeated run with instructions
- **WHEN** the same command with `--instructions "Speak in Dutch."` is run twice
- **THEN** the second run is a cache hit and no API call is made

### Requirement: Changing instructions after a cached run produces a cache miss
The system SHALL treat a changed `instructions` value as a cache miss even when all other parameters are identical.

#### Scenario: Cache miss on changed instructions
- **WHEN** a run with `--instructions "Speak in Dutch."` is followed by a run with `--instructions "Speak in English."`
- **THEN** the second run is a cache miss and a new API call is made

### Requirement: Removing instructions after a cached run produces a cache miss
The system SHALL treat the removal of `--instructions` (i.e. transition from `Some` to `None`) as a cache miss.

#### Scenario: Cache miss when instructions removed
- **WHEN** a run with `--instructions "Speak in Dutch."` is followed by a run without `--instructions`
- **THEN** the second run is a cache miss

### Requirement: Instructions value is never truncated in the cache key
The system SHALL use the full, untruncated `instructions` string as input to the cache key hash, regardless of console display truncation.

#### Scenario: Long instructions string hashed in full
- **WHEN** `instructions` is a string longer than 80 characters
- **THEN** the complete string (not a truncated version) is used in the cache key computation
