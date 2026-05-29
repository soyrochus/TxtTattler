# Proposal: Measuring Whether Granular Specification Reduces Architecture Drift

## Research Question

Does granular, testable specification reduce architecture drift compared with one-shot AI code generation?

The primary hypothesis is:

> For an architecture-sensitive Rust CLI application, a staged OpenSpec-driven workflow will produce less architecture drift than a one-shot prompt, even when using the same model.

The experiment should evaluate process effects first and model effects second. The main question is not whether one model is globally better than another. The main question is whether a structured specification process improves architectural fidelity within each model.

## Experimental Design

Use a lightweight 2 by 2 experiment.

| Dimension | Option A | Option B |
| --- | --- | --- |
| Generation method | One-shot prompt | OpenSpec / granular specification |
| Model | Codex | Claude Opus |

This creates four implementations:

| Variant | Model | Method |
| --- | --- | --- |
| A1 | Codex | One-shot |
| A2 | Codex | OpenSpec-driven |
| B1 | Claude Opus | One-shot |P
| B2 | Claude Opus | OpenSpec-driven |

The primary comparisons are:

```text
Codex one-shot       vs Codex OpenSpec
Claude one-shot      vs Claude OpenSpec
```

These comparisons estimate whether the specification process improves architecture fidelity within the same model.

The secondary comparisons are:

```text
Codex OpenSpec       vs Claude OpenSpec
Codex one-shot       vs Claude one-shot
One-shot average     vs OpenSpec average
```

These comparisons estimate whether model choice matters once the workflow is controlled, and whether the structured workflow narrows the quality gap between models.

## Target Scope

The experiment should not rebuild the full TxtTattler application. It should use a reduced but architecture-sensitive slice called:

```text
TxtTattler Core
```

The slice should include the specific failure modes the experiment is intended to measure.

| Included area | Reason |
| --- | --- |
| CLI contract | Provides an externally visible control surface. |
| Config precedence | Exposes hidden logic drift and default-handling mistakes. |
| Ports/adapters boundary | Directly tests architectural separation. |
| Text processor pipeline | Tests whether middleware is implemented as architecture rather than incidental code. |
| Chunking | Tests algorithmic compliance and edge-case handling. |
| Cache key and atomic writes | Tests cross-cutting correctness and persistence boundaries. |
| Reporter abstraction | Tests whether console I/O leaks into the use case. |
| Tests | Measures whether critical behavior is verified, not merely implemented. |

The slice should exclude features that add environment or integration complexity without improving the architecture-drift signal.

| Excluded area | Reason |
| --- | --- |
| Real OpenAI integration | Adds API and network complexity without materially improving the drift measurement. |
| Real audio playback | Environment-sensitive and orthogonal to the architecture question. |
| Full PDF/DOCX support | Placeholder behavior is sufficient for this experiment. |
| Release binaries and GitHub workflow | Useful for productization, not for this experiment. |
| Full README polish | Consumes time without improving the dependent variable. |

The implementation should include a fake deterministic `TtsProvider` that writes stable bytes. The port for a real provider should still exist, but no real API calls are required.

## Canonical Requirements Packet

Before running any variant, create a single source-of-truth requirements packet. This packet should define the complete expected behavior and architecture for TxtTattler Core.

The packet should cover:

| Requirement area | Expected content |
| --- | --- |
| CLI contract | Commands, flags, validation behavior, `--list-voices`, input/output handling. |
| Config resolution | Exact precedence: CLI > environment > config file > defaults. |
| Domain model | Core types, use case responsibilities, and error boundaries. |
| Ports | TTS provider, cache store, reporter, file/text input, and any other external effects. |
| Adapters | Filesystem cache, fake TTS provider, CLI reporter, and config loader. |
| Text processing | Named composable processors and a stable pipeline version. |
| Chunking | Paragraph, sentence, word, then character fallback behavior with boundary tests. |
| Cache behavior | Null-byte-delimited SHA-256 cache key, pipeline version inclusion, reuse, refresh, no-cache, and atomic writes. |
| Test expectations | Unit, integration, and architecture checks required for acceptance. |

The one-shot prompt and the OpenSpec sequence should be derived from this same packet. The OpenSpec path may provide more structure, but it must not receive additional requirements that the one-shot path does not receive.

## OpenSpec Change Sequence

Use 6 to 8 changes. More than that will increase experiment cost and make the comparison harder to interpret.

Recommended sequence:

| Step | Change | Acceptance focus |
| ---: | --- | --- |
| 1 | `init-cli-and-bootstrap` | Thin `main.rs`, parse-time validation, `--list-voices`. |
| 2 | `define-domain-ports` | Ports in the domain layer; no infrastructure imports in the use case. |
| 3 | `implement-config-resolution` | CLI > environment > config file > defaults; explicit default tests. |
| 4 | `implement-text-processing` | Named composable processors; stable pipeline version. |
| 5 | `implement-chunking` | Paragraph -> sentence -> word -> character fallback; limit tests. |
| 6 | `implement-cache` | Null-byte SHA-256 key, pipeline version, atomic writes, refresh/no-cache behavior. |
| 7 | `implement-use-case` | Orchestration through ports only. |
| 8 | `add-architecture-tests` | Static checks for imports, no direct console or filesystem access in the use case. |

Each change should include:

- A short motivation.
- Concrete requirements.
- Acceptance criteria.
- Required tests.
- Explicit out-of-scope notes.

## Fairness Controls

The main validity risk is accidentally making the OpenSpec variant easier by giving it more precise requirements than the one-shot variant. To reduce that risk:

1. Use one canonical requirements packet for all variants.
2. Derive both the one-shot prompt and the OpenSpec changes from that packet.
3. Keep the same starting repository state for all variants.
4. Use the same time budget or interaction budget for all variants.
5. Avoid manual repairs before scoring.
6. Record every prompt, correction, command, and model response needed to reach the final implementation.
7. Score implementations only after the run is complete.

The OpenSpec workflow is allowed to be more structured. It should not be allowed to contain hidden extra behavior, hidden extra examples, or hidden evaluator hints.

## Evaluation Model

Use three scores:

| Score | Measures | Weight |
| --- | --- | ---: |
| Architecture fidelity | Whether the implementation preserves intended boundaries and structure. | 40% |
| Behavioral correctness | Whether specified edge cases and user-visible behavior work. | 35% |
| Verification quality | Whether critical behavior is covered by meaningful tests. | 25% |

Also preserve the raw subscores. The weighted total is useful for comparison, but the raw scores will be more persuasive and easier to inspect.

```text
Total Score =
  Architecture Fidelity * 0.40 +
  Behavioral Correctness * 0.35 +
  Verification Quality * 0.25
```

## Architecture Drift Metric

Architecture drift should be scored against explicit indicators. Each indicator should be marked as pass, fail, or partial. Partial credit should be used sparingly and documented.

Recommended architecture indicators:

| # | Indicator | Severity |
| ---: | --- | --- |
| 1 | Use case imports no infrastructure modules. | High |
| 2 | Use case does not import `std::fs`, filesystem adapters, or path-specific persistence code. | High |
| 3 | Use case does not import terminal rendering, progress bars, or console libraries. | High |
| 4 | External effects are behind ports. | High |
| 5 | Config resolution is centralized and not duplicated across CLI and use case layers. | Medium |
| 6 | CLI defaults preserve precedence by using `Option<T>` where needed. | Medium |
| 7 | Text processors are named, composable units rather than one monolithic string function. | Medium |
| 8 | The processor pipeline has a stable version included in the cache key. | High |
| 9 | Cache writes are atomic. | High |
| 10 | Reporter output is abstracted behind a port and does not leak into core orchestration. | High |
| 11 | Tests cover config precedence. | Medium |
| 12 | Tests cover cache reuse, refresh, no-cache, and key invalidation behavior. | Medium |
| 13 | Tests cover chunking fallbacks and size limits. | Medium |
| 14 | Architecture tests detect forbidden imports or dependency direction violations. | High |

Calculate drift as:

```text
Architecture Drift Score = failed indicators / total indicators
```

If partial credit is used:

```text
pass = 0 drift points
partial = 0.5 drift points
fail = 1 drift point

Architecture Drift Score = drift points / total indicators
```

Example interpretation:

| Drift score | Interpretation |
| ---: | --- |
| 0.00-0.10 | Minimal detected drift. |
| 0.11-0.30 | Low to moderate drift. |
| 0.31-0.60 | Significant drift. |
| 0.61-1.00 | Severe drift. |

## Behavioral Correctness Metric

Behavioral correctness should be evaluated with a fixed checklist and, where possible, a shared test suite.

Recommended behavioral checks:

| # | Behavior | Severity |
| ---: | --- | --- |
| 1 | CLI rejects invalid option combinations at parse time or before execution. | Medium |
| 2 | `--list-voices` works without requiring an input file. | Medium |
| 3 | Config precedence is exactly CLI > environment > config file > defaults. | High |
| 4 | Explicit defaults are tested separately from absent values. | Medium |
| 5 | Text processors run in the specified order. | High |
| 6 | Pipeline version changes invalidate the cache. | High |
| 7 | Chunking prefers paragraphs, then sentences, then words, then characters. | High |
| 8 | Chunks never exceed the configured limit unless the specification explicitly allows it. | High |
| 9 | Cache reuse avoids calling the fake TTS provider. | Medium |
| 10 | Refresh bypasses existing cache entries and rewrites them. | Medium |
| 11 | No-cache bypasses reading and writing cache entries. | Medium |
| 12 | Atomic write failures do not leave a corrupted final cache file. | High |

## Verification Quality Metric

Verification quality should measure whether the model produced useful evidence, not just whether tests exist.

Recommended verification checks:

| # | Verification indicator | Severity |
| ---: | --- | --- |
| 1 | Unit tests cover config resolution precedence. | High |
| 2 | Unit tests cover text processor ordering. | Medium |
| 3 | Unit tests cover chunking edge cases. | High |
| 4 | Unit or integration tests cover cache reuse and invalidation. | High |
| 5 | Tests verify atomic write behavior or failure safety. | Medium |
| 6 | Tests verify use case orchestration through fake ports. | High |
| 7 | Architecture tests check forbidden imports or dependency direction. | High |
| 8 | Tests are deterministic and do not require network access. | High |
| 9 | Tests do not rely on real audio playback or real OpenAI credentials. | High |
| 10 | Test names communicate the behavior being protected. | Low |

## Intervention Cost Metric

Track intervention cost as a fourth non-weighted metric. This helps distinguish quality gains from process cost.

Record:

| Metric | Meaning |
| --- | --- |
| Prompt count | Number of user prompts required to complete the run. |
| Correction count | Number of prompts that explicitly fix or redirect the model. |
| Wall-clock time | Approximate elapsed time from first prompt to completed implementation. |
| Tool/runtime failures | Number of failed commands or blocked tool actions. |
| Generated churn | Approximate number of large rewrites or abandoned implementation paths. |

This metric should not replace architecture or behavior scoring, but it should appear in the final analysis. A process that reduces drift but requires much more steering may still be valuable, but the tradeoff should be visible.

## Run Protocol

For each variant:

1. Create a clean working copy from the same baseline.
2. Record the model, model settings, date, and generation method.
3. Provide the appropriate prompt or OpenSpec change sequence.
4. Allow the model to implement within the agreed interaction budget.
5. Do not manually repair the result before scoring.
6. Run the same formatter, test suite, and evaluator checks.
7. Record failures, generated tests, architectural violations, and intervention cost.
8. Score the implementation using the rubric.
9. Archive the implementation, transcript, scorecard, and run notes.

## Required Experiment Artifacts

The experiment should produce these files or equivalent records:

| Artifact | Purpose |
| --- | --- |
| `canonical-requirements.md` | Single source of truth for all variants. |
| `one-shot-prompt.md` | Prompt used for A1 and B1. |
| `openspec/changes/...` | Staged changes used for A2 and B2. |
| `scorecard-template.md` | Rubric used for all variants. |
| `run-log-template.md` | Prompt count, corrections, failures, and timing. |
| `results.md` | Final comparison and interpretation. |
| Variant repositories or branches | Immutable implementation outputs for review. |

## Analysis Plan

Analyze results in this order:

1. Compare one-shot vs OpenSpec within Codex.
2. Compare one-shot vs OpenSpec within Claude Opus.
3. Compare average one-shot score vs average OpenSpec score.
4. Compare Codex OpenSpec vs Claude Opus OpenSpec.
5. Compare intervention cost across all four variants.
6. Review qualitative drift patterns that recur across models.

The most important signal is whether both models show lower architecture drift under the OpenSpec workflow. If only one model improves, the result is still useful but should be framed as model-dependent.

## Expected Outcome

If the hypothesis is supported, the expected result is:

```text
Architecture drift:
OpenSpec variants < one-shot variants

Behavioral correctness:
OpenSpec variants >= one-shot variants

Verification quality:
OpenSpec variants > one-shot variants
```

It is acceptable if OpenSpec has a higher intervention cost. The conclusion should report that tradeoff directly.

## Claims Supported by This Experiment

If the expected result occurs, the strongest defensible claim is:

> In this reduced Rust CLI experiment, granular executable specification reduced specific architecture drift patterns compared with one-shot generation. The effect appeared within model families, suggesting that process quality had an independent effect from model choice.

A more cautious version is:

> For this architecture-sensitive TxtTattler Core slice, staged specification improved architectural fidelity and verification quality relative to one-shot prompting under the tested conditions.

## Claims Not Supported by This Experiment

This experiment should not claim:

```text
OpenSpec makes AI-generated code production-ready.
```

That is too broad.

It should not claim:

```text
Model X is better than model Y.
```

The setup is too contextual for a general model ranking.

It should not claim:

```text
Granular specification always improves AI-generated software.
```

The result applies to this specific architecture-sensitive Rust CLI case unless replicated elsewhere.

## Recommendation

Proceed with the 2 by 2 experiment using TxtTattler Core as the target slice. Before running the four implementations, first prepare the canonical requirements packet, one-shot prompt, OpenSpec change sequence, scorecard, and run log template.

The proposal is most valuable if it remains narrow, repeatable, and explicit about tradeoffs. The final result should emphasize architecture drift as the main dependent variable, with behavioral correctness, verification quality, and intervention cost used to explain the outcome.
