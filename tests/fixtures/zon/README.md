# ZON reference fixtures

`reference-aliases.zonf` exercises syntax accepted by the TypeScript reference implementation at commit [`1b5917dbbf254ba11fb217a378d8ea05a17598d2`](https://github.com/ZON-Format/zon-TS/commit/1b5917dbbf254ba11fb217a378d8ea05a17598d2): the `@N` root-table alias, single-quoted cells, exponent numbers, case-insensitive booleans, and null aliases.

`reference-aliases.json` is the equivalent JSON data model used to assert compatible decoding. ACORN's encoder intentionally normalizes the decoded value to its stricter canonical profile rather than reproducing alias spelling.
