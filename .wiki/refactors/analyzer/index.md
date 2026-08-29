# Analyzer Refactor Plans

These plans prepare the pure analysis boundary defined by the [crate boundary architecture](../crate-boundary-architecture.md).

## Recommended first

- [Check batch monoid](check-batch-monoid.md) - Reduce repeated asynchronous collection assembly.
- [Standards traversal](standards-traversal.md) - Deduplicate near-isomorphic standard analysis flows after check batching stabilizes; portable validation is already flat.

## Additional plans

- [Website and resolution checks](link-resolution-checks.md) - Route URL reachability and PID resolution through one link-analysis contract.
- [Summary monoid](summary-monoid.md) - Fold checks into one additive summary.
- [Vale configuration normalization](vale-config-normalization.md) - Compose configuration transformations.
- [Applicative validation](applicative-validation.md) - Accumulate independent failures.
- [Check options context](check-options-context.md) - Stop repeatedly threading derived options.
- [Result boundary normalization](result-boundary-normalization.md) - Standardize conversion and recovery.
- [Typed scoring model](typed-scoring-model.md) - Make score combination and severity interpretation explicit.
