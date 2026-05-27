## ADDED Requirements

### Requirement: Content-Addressed MP3 Cache
The canonical app SHALL cache generated MP3 audio by a content-based key.

#### Scenario: Cache key uses output-affecting fields
- **WHEN** the cache key is computed
- **THEN** it includes processed text, voice, model, speed, and text-processing pipeline version

#### Scenario: Cache key uses null separators
- **WHEN** fields are hashed into the cache key
- **THEN** fields are separated by null bytes rather than printable delimiters

### Requirement: Cache Hit Reuse
The canonical app SHALL reuse a valid cached MP3 for repeated runs with identical processed text and options.

#### Scenario: Second identical run avoids TTS
- **WHEN** the use-case runs twice with identical input and options
- **THEN** the second run is a cache hit and the TTS provider is not called again

### Requirement: Refresh Regenerates Cache
The canonical app SHALL regenerate audio and replace the cache entry when `--refresh` is supplied.

#### Scenario: Refresh ignores existing cache
- **WHEN** a matching cache entry exists and the user passes `--refresh`
- **THEN** the app calls the TTS provider and atomically replaces the cache entry

### Requirement: No Cache Skips Reads And Writes
The canonical app SHALL skip both cache reads and cache writes when `--no-cache` is supplied.

#### Scenario: No-cache one-off generation
- **WHEN** the user passes `--no-cache`
- **THEN** the app does not read an existing cache entry and does not write a new cache entry

### Requirement: Atomic Cache Writes
The canonical app SHALL write cache files atomically.

#### Scenario: Cache write succeeds
- **WHEN** generated audio is written to cache
- **THEN** data is written to a temporary file in the target directory and renamed into place

#### Scenario: Failed cache write preserves old file
- **WHEN** replacing an existing cache entry fails before rename
- **THEN** the existing cache file remains valid

### Requirement: Output Writing From Cache
The canonical app SHALL write `--output` from cached audio when a cache hit occurs.

#### Scenario: Output on cache hit
- **WHEN** the user passes `--output out.mp3` and a valid cache entry exists
- **THEN** the app writes `out.mp3` from the cached MP3 without making an OpenAI request

### Requirement: Atomic Output Writes
The canonical app SHALL write user-visible `--output` files atomically.

#### Scenario: Output write succeeds atomically
- **WHEN** the app writes an output MP3
- **THEN** it writes to a temporary file and renames into the requested output path
