# Architecture Drift Experiment

This directory contains the materials for testing whether granular, testable specification reduces architecture drift compared with one-shot AI code generation.

The reference application remains at:

```text
../txttattler/
```

Historical generated implementations are archived at:

```text
../implementations-archive/
```

The original one-shot generation prompt is:

```text
../original-txttattler-creation-prompt.md
```

## Structure

| Path | Purpose |
| --- | --- |
| `metrics-proposal.md` | Formal experiment proposal and scoring approach. |
| `canonical-requirements.md` | Source-of-truth requirements shared by all variants. |
| `one-shot-prompt.md` | Prompt used for one-shot implementation variants. |
| `scorecard-template.md` | Scoring template for architecture, behavior, verification, and intervention cost. |
| `run-log-template.md` | Per-run log template for prompts, corrections, timing, and failures. |
| `openspec/changes/` | Staged OpenSpec changes for the granular-spec variants. |
| `runs/` | Output directories for the four experiment variants. |

## Variants

| Variant | Directory | Method |
| --- | --- | --- |
| Codex one-shot | `runs/codex-one-shot/` | Single prompt. |
| Codex OpenSpec | `runs/codex-openspec/` | Staged OpenSpec changes. |
| Claude one-shot | `runs/claude-one-shot/` | Single prompt. |
| Claude OpenSpec | `runs/claude-openspec/` | Staged OpenSpec changes. |
