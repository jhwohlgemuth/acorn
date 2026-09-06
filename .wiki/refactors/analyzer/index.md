# Analyzer Refactor Plans

These plans improve the in-tree analysis boundary defined by the [crate boundary architecture](../crate-boundary-architecture.md). They do not by themselves authorize crate extraction.

## Recommended next

- [Standards traversal](standards-traversal.md) - Deduplicate near-isomorphic standard analysis flows after check batching stabilizes; portable validation is already flat.

## Additional plans

The summary and Vale normalization plans apply the small, law-driven composition patterns described in [the category-inspired composition concept](../../../docs/src/concepts/category-theory.md); they do not introduce a category-theory framework.

- [Website and resolution checks](link-resolution-checks.md) - Route URL reachability and PID resolution through one link-analysis contract.
- [Vale configuration normalization](vale-config-normalization.md) - Compose configuration transformations.
- [Result boundary normalization](result-boundary-normalization.md) - Standardize conversion and recovery.
