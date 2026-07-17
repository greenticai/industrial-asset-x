# PR 3 — Add Industrial-X MVP contracts

## Goal

Define the minimum predictive-maintenance and industrial asset-health resource model.

## Scope

Add generic Industrial-X contracts for data profiling, normal behaviour, features, predictions, recommendations, feedback, and model evaluation.

## Adds contracts

```text
contracts/industrialx/signal-profile/
contracts/industrialx/data-quality-report/
contracts/industrialx/normal-behaviour-profile/
contracts/industrialx/feature-set/
contracts/industrialx/prediction/
contracts/industrialx/recommendation/
contracts/industrialx/feedback-event/
contracts/industrialx/model-evaluation/
```

## Key resource lifecycle examples

```text
signal-profile: draft -> validated -> superseded
data-quality-report: generated -> reviewed -> accepted -> superseded
normal-behaviour-profile: generated -> validated -> active -> superseded
prediction: generated -> explained -> reviewed -> closed
recommendation: proposed -> approved -> executed -> verified
feedback-event: recorded -> reviewed -> learned
```

## Acceptance criteria

- Resources can be linked to Asset-X resources.
- Schemas are suitable for deterministic Rust/WASM analytics output.
- Prediction and recommendation resources capture evidence and confidence.
- Feedback events support false positive, useful alert, missed issue, bad explanation, wrong asset, wrong failure mode, maintenance confirmed, and no fault found.

## Out of scope

- Synthetic data contracts.
- Training datasets and model tournament contracts.
- Customer-specific demo schemas.
