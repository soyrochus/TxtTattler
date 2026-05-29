# Canonical Requirements

This file is the single source of truth for all experiment variants.

Use it to define the full TxtTattler Core behavior before deriving either:

- `one-shot-prompt.md`
- staged OpenSpec changes under `openspec/changes/`

The one-shot and OpenSpec paths should receive the same requirements. The OpenSpec path may be more structured, but it should not contain additional hidden behavior or evaluator hints.

## Scope

Target application:

```text
TxtTattler Core
```

Reference application:

```text
../txttattler/
```

Original generation prompt:

```text
../original-txttattler-creation-prompt.md
```

## Requirements To Define

| Area | Required detail |
| --- | --- |
| CLI contract | Commands, flags, validation behavior, `--list-voices`, input/output handling. |
| Config resolution | Exact precedence: CLI > environment > config file > defaults. |
| Domain model | Core types, use case responsibilities, and error boundaries. |
| Ports | TTS provider, cache store, reporter, file/text input, and other external effects. |
| Adapters | Filesystem cache, fake TTS provider, CLI reporter, and config loader. |
| Text processing | Named composable processors and a stable pipeline version. |
| Chunking | Paragraph, sentence, word, then character fallback behavior with boundary tests. |
| Cache behavior | Null-byte-delimited SHA-256 cache key, pipeline version inclusion, reuse, refresh, no-cache, and atomic writes. |
| Test expectations | Unit, integration, and architecture checks required for acceptance. |
