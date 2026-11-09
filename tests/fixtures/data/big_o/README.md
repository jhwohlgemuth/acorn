# Big-O research activity fixtures

These complete Research Activity Data fixtures share one valid envelope and exercise nested computation-complexity validation.

- `valid/index.json` contains time and space bounds across three dependent stages.
- `invalid-syntax/index.json` replaces the first time bound with unsupported subtraction.
- `invalid-reference/index.json` leaves `n` undeclared in the first time bound while keeping the space bound valid.

The canonical comparison pair is `O(n^2)` and `O(n log n)`. Tests preserve every expression byte-for-byte and assert validation paths relative to `aspect.computation`.
