# Scorecard Template

Variant:

Model:

Method:

Date:

Evaluator:

## Summary Scores

| Score | Raw | Weight | Weighted |
| --- | ---: | ---: | ---: |
| Architecture fidelity | TODO | 40% | TODO |
| Behavioral correctness | TODO | 35% | TODO |
| Verification quality | TODO | 25% | TODO |
| Total | TODO | 100% | TODO |

## Architecture Drift

| # | Indicator | Result | Notes |
| ---: | --- | --- | --- |
| 1 | Use case imports no infrastructure modules. | TODO | TODO |
| 2 | Use case does not import filesystem persistence code. | TODO | TODO |
| 3 | Use case does not import terminal rendering or progress libraries. | TODO | TODO |
| 4 | External effects are behind ports. | TODO | TODO |
| 5 | Config resolution is centralized. | TODO | TODO |
| 6 | CLI defaults preserve precedence with optional values where needed. | TODO | TODO |
| 7 | Text processors are named and composable. | TODO | TODO |
| 8 | Pipeline version is included in the cache key. | TODO | TODO |
| 9 | Cache writes are atomic. | TODO | TODO |
| 10 | Reporter output is abstracted behind a port. | TODO | TODO |
| 11 | Tests cover config precedence. | TODO | TODO |
| 12 | Tests cover cache reuse, refresh, no-cache, and invalidation. | TODO | TODO |
| 13 | Tests cover chunking fallbacks and size limits. | TODO | TODO |
| 14 | Architecture tests detect forbidden imports or dependency direction violations. | TODO | TODO |

## Behavioral Correctness

| # | Behavior | Result | Notes |
| ---: | --- | --- | --- |
| 1 | CLI rejects invalid option combinations. | TODO | TODO |
| 2 | `--list-voices` works without requiring input. | TODO | TODO |
| 3 | Config precedence is exact. | TODO | TODO |
| 4 | Explicit defaults are tested separately from absent values. | TODO | TODO |
| 5 | Text processors run in the specified order. | TODO | TODO |
| 6 | Pipeline version changes invalidate cache. | TODO | TODO |
| 7 | Chunking follows paragraph, sentence, word, character fallback. | TODO | TODO |
| 8 | Chunks respect the configured limit. | TODO | TODO |
| 9 | Cache reuse avoids fake TTS calls. | TODO | TODO |
| 10 | Refresh bypasses existing cache entries. | TODO | TODO |
| 11 | No-cache bypasses reading and writing cache entries. | TODO | TODO |
| 12 | Atomic write failures do not corrupt final cache files. | TODO | TODO |

## Verification Quality

| # | Indicator | Result | Notes |
| ---: | --- | --- | --- |
| 1 | Unit tests cover config precedence. | TODO | TODO |
| 2 | Unit tests cover processor ordering. | TODO | TODO |
| 3 | Unit tests cover chunking edge cases. | TODO | TODO |
| 4 | Tests cover cache reuse and invalidation. | TODO | TODO |
| 5 | Tests verify atomic write behavior or failure safety. | TODO | TODO |
| 6 | Tests verify orchestration through fake ports. | TODO | TODO |
| 7 | Architecture tests check forbidden imports or dependency direction. | TODO | TODO |
| 8 | Tests are deterministic and offline. | TODO | TODO |
| 9 | Tests do not require real audio playback or OpenAI credentials. | TODO | TODO |
| 10 | Test names communicate protected behavior. | TODO | TODO |

## Intervention Cost

| Metric | Value | Notes |
| --- | ---: | --- |
| Prompt count | TODO | TODO |
| Correction count | TODO | TODO |
| Wall-clock time | TODO | TODO |
| Tool/runtime failures | TODO | TODO |
| Generated churn | TODO | TODO |
