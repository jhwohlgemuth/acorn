---
type: Plan
title: "Creusot verification for PID checksum calculations"
description: "Formally verify panic/overflow-freedom and functional correctness of persistent-identifier check-digit calculations in acorn-lib's PID module using Creusot (Coma/Why3/SMT)."
difficulty: high
impact: medium
package: [acorn-lib]
tags: [testing, verification, creusot, pid, checksum, why3]
status: planned
priority: next
---

# Goal

Prove that every PID checksum / check-digit routine in `crates/acorn-lib/src/schema/pid/mod.rs` is free of panics, integer overflows, and logic mismatches against its published specification, using the Creusot deductive verifier. Initial scope is the six pure checksum helpers and their public wrappers; network, regex, and SHA-256 paths are explicitly excluded.

# Current evidence

## In-scope checksum surface (verified 2026-08-20 against `acorn-lib` `0.1.72`)

| Function | Location | Normative spec | Current implementation notes |
|---|---|---|---|
| `isbn_check_digit` | `crates/acorn-lib/src/schema/pid/mod.rs:1911` | ISBN-10 MOD 11 ( `X` for 10 ) and ISBN-13 MOD 10 with alternating weights 1/3 | `try_fold` over 9/12 chars; `#[allow(clippy::arithmetic_side_effects)]` at `mod.rs:1910`; integration test vectors at `crates/acorn-lib/src/schema/pid/tests/mod.rs:335` |
| `isni_check_digit` | `mod.rs:1949` | ISO 7064 MOD 11-2 | delegates to `mod_11_2_check_digit` |
| `orcid_check_digit` | `mod.rs:2014` | ISO 7064 MOD 11-2 | delegates to `mod_11_2_check_digit`; vectors at `tests/mod.rs:511` |
| `mod_11_2_check_digit` (private core) | `mod.rs:1959` | ISO 7064 MOD 11-2, modulus 11 radix 2 | `fold(0,(acc+digit)*2)`, `(MODULUS+1-remainder)%MODULUS`; uses `unwrap_or_default` at `mod.rs:1967` |
| `noid_check_digit` | `mod.rs:1990` | NOID Check Digit Algorithm (NCDA), radix 29 | `sum(position*ordinal)%29` via `Betanumeric::to_betanumeric_ordinal` `mod.rs:46,749`; `BETANUMERIC_DIGITS` `mod.rs:41` length 29; vector at `tests/mod.rs:422` |
| `ror_check_digit` | `mod.rs:2035` | ISO 7064 MOD 97-10, ROR + Crockford Base32 | `take(6)` + `base32_crockford_decode` `crates/acorn-lib/src/util/mod.rs:873` + `(value*100)%97`; `98 - remainder` zero-padded; vectors at `tests/mod.rs:708` |
| `base32_crockford_encode` / `decode` / `crockford_digit` | `crates/acorn-lib/src/util/mod.rs:845,873,904` | Crockford Base32 (excludes I,L,O,U) | `checked_mul`/`checked_add` loop with `HashSet` dedup; used only by ROR |
| `to_betanumeric` | `mod.rs:2063` | helper for NOID | linear search in `BETANUMERIC_DIGITS` |

Out of scope for phase 1: `Identifier::identifier_hash` `mod.rs:571` (SHA-256 via `ring`), `RE_*` regex parsing, and all `PersistentIdentifierParse` dispatch; they remain tested by existing `is_valid`/`find_all` unit tests.

## Tooling readiness

- No Creusot/Why3 integration exists in the repo (grep `creusot|why3|coma` across workspace returns zero hits).
- Toolchain pinned to `rust-version = "1.95.0"` `Cargo.toml:7`; Creusot requires nightly + `why3` + SMT solvers (`z3`, `cvc5`, `alt-ergo`) — see [Creusot](https://creusot.rs/) and [creusot-rs/creusot](https://github.com/creusot-rs/creusot).
- Workspace lints `unwrap_used = "warn"` `Cargo.toml:39` and `arithmetic_side_effects = "warn"` `Cargo.toml:22` already forbid patterns Creusot will prove away; `clippy.toml` allows unwraps in tests only.
- Checksum helpers are `alloc`-only (except pure `core` arithmetic) and can be verified under `--no-default-features --features alloc`, avoiding `std` dependencies (`ring`, `fancy-regex`, `reqwest`) gated in `crates/acorn-lib/Cargo.toml:144`.

## Related work

- [Informational Miri quality checks](miri-quality.md) establishes the pattern for isolated, informational verification jobs over `acorn-lib` `alloc` surface — reuse its curated-suite and `make miri` conventions.
- [CLI test coverage initiative](index.md) tracks complementary dynamic test coverage; Creusot is deductive and does not overlap.

# Recommendation

## Phase 0 — Tool install & harness (1 day)

1. Install Creusot nightly toolchain, `why3` platform, and at least two SMT solvers. Document in `CONTRIBUTING.md` alongside Miri install notes.
2. Add `make creusot` mirroring the existing `make wasm` pattern (`make wasm` runs `cargo check --no-default-features` for portable surface):
   ```make
   creusot:
       cargo creusot --only=coma -p acorn-lib --no-default-features --features alloc
   creusot-prove:
       cargo creusot -p acorn-lib --no-default-features --features alloc
   ```
   Default `cargo creusot` now runs compile + prove (`cargo creusot 0.12.0 prove-by-default` — see [Creusot 0.12.0 devlog](https://devlog.creusot.rs/v0-12-0/)).
3. Create a feature-gated shim `#[cfg(creusot)]` that re-exports checksum helpers with contracts, so normal `cargo check/test` is unaffected. Pin `creusot-contracts` as an optional dev dependency; do not add to `default` features.

## Phase 1 — Isolate pure cores (2 days)

1. Keep public signatures (`Option<Vec<char>>`) for backward compat but extract inner `fn mod_11_2_on_digits(digits: &[u8]) -> u32` etc for spec-level proof. Where `String` construction is required, provide `#[logic]` spec functions.
2. Replace proof-hostile patterns with invariant-friendly forms:
   - `unwrap_or_default` at `mod.rs:1967` (`to_digit` fallback) → explicit `requires` that input contains digits.
   - `BETANUMERIC_DIGITS.chars().position` at `mod.rs:750` → `#[logic]` lookup with bounded index `< 29`.
3. Annotate `BETANUMERIC_DIGITS` length, `CROCKFORD_BASE32_ALPHABET` length, and `checked_mul/add` preconditions for overflow-freedom.

## Phase 2 — Contracts (3–5 days)

Attach `creusot_contracts::{requires, ensures, invariant}` to each helper:

- **Safety:** `#[ensures(!panics)]` +溢出 freedom: `checked_mul`/`checked_add` succeed, indexing within `BETANUMERIC_DIGITS`, `CROCKFORD_BASE32_ALPHABET`.
- **Functional:**

  ```rust
  #[requires(value.chars().filter(|c| c.is_ascii_digit()).count() >= 15)]
  #[ensures(result.is_some() ==> result.unwrap().len()==1 && (result.unwrap()[0].is_ascii_digit() || result.unwrap()[0]=='X'))]
  pub fn mod_11_2_check_digit<S: AsRef<str>>(value: S) -> Option<Vec<char>>
  ```

  - `isbn_check_digit`: 10-char branch `check = (11 - sum%11)%11` → `X` iff 10 else digit; 13-char branch `check = (10 - sum%10)%10` with weight `1 + 2*(i%2)`.
  - `noid_check_digit`: `sum = Σ (i+1)*ordinal(c_i)` `% 29` maps via `BETANUMERIC_DIGITS[sum%29]`.
  - `ror_check_digit`: `take(6)` truncation `mod.rs:2045`, `base32_crockford_decode` inverts `encode` (lemma `decode(encode(n))==Some(n)` in `util/mod.rs`), then `checksum = (98 - (decoded*100)%97)%97`.
  - `base32_crockford_decode`: loop `#[invariant(acc < 32u128.pow(k))]` and `checked_*` postconditions.

Prefer `pearlite` `seq` for sum specs; keep `util::base32_crockford_decode` loop as explicit `for` for Creusot invariant inference.

## Phase 3 — Prove & curate

1. Fast feedback: `cargo creusot --only=coma` (type-checks Coma translation). Fix unsupported constructs (`fancy_regex`, `HashSet` in `merge`) by keeping them out of verified path.
2. Full prove: `cargo creusot` → Why3 VCs → SMT. Expect manual lemmas for Crockford modulo and ROR padding; place in `crates/acorn-lib/src/util/mod.rs` logic module.
3. Cross-check against existing vectors (`tests/mod.rs:335,422,511,708`) — encode as `#[ensures]` examples so proofs subsume tests.

## Phase 4 — CI & sustainability

- Add informational GitLab job `creusot` (test stage, `allow_failure: true` initially, mirroring `miri-quality.md`), gating on `acorn-lib/**`, `Cargo.*`, `Makefile`.
- Document maintenance: re-pin nightly with fuzz/Miri nightly (`nightly-2026-08-01` in `miri-quality.md`), keep `make creusot` as canonical local entry, file upstream issues for unsupported `alloc` patterns rather than disabling checks globally.

# Validation

1. `cargo creusot --only=coma -p acorn-lib --no-default-features --features alloc` type-checks with zero errors.
2. `cargo creusot -p acorn-lib --no-default-features --features alloc` discharges all checksum VCs with `z3` + `cvc5` (or `alt-ergo`) without `admit`/`assume`.
3. `make test` and `make test-ci` still pass; no new `unsafe` introduced (`unsafe_code = "forbid"` `Cargo.toml:48`).
4. `cargo clippy --workspace --no-deps --all-targets` clean (existing `#[allow(clippy::arithmetic_side_effects)]` at `mod.rs:1910,1958,1989,2014,2034` remains justified and proven).
5. A merge request touching `crates/acorn-lib/src/schema/pid/**` or `crates/acorn-lib/src/util/mod.rs` triggers the `creusot` job, logs are exposed, pipeline not blocked.

# Acceptance criteria

1. `make creusot` and `make creusot-prove` are documented and reproduce locally.
2. All five checksum families (ISBN-10/13, ISNI/ORCID MOD 11-2, NOID NCDA, ROR MOD 97-10) plus `base32_crockford_*`/`Betanumeric` helpers carry `requires/ensures` linking to normative references in their doc comments (`mod.rs:1978,2003,2021`).
3. Proofs cover overflow-freedom and functional equivalence to spec, not just smoke tests.
4. Existing unit vectors in `crates/acorn-lib/src/schema/pid/tests/mod.rs` are encoded as proof examples and continue to pass natively.
5. No blanket `cfg_attr(creusot, ignore)` or solver-disable flags; only narrow, documented lemmas.

# Risks & tradeoffs

- **Unsupported Rust features:** `fancy_regex`, `serde` derives, and `HashMap`/`HashSet` in non-checksum code are outside Creusot's supported fragment — mitigated by scoping to pure `alloc`/`core` helpers.
- **String allocation model:** Creusot's `std`/`alloc` model is less mature than `core` — fallback is to verify `&[u8]` cores and lift via wrapper lemmas.
- **Nightly pin drift:** reuse the existing `nightly-2026-08-01`/`1.95.0` coordination pattern from `miri-quality.md`; update lockstep.
- **Alternative tooling:** `prusti`/`kani` provide cheaper bounded checking but weaker deductive guarantees — Creusot chosen for ISO-spec functional proofs; `proptest` (`crates/acorn-lib/Cargo.toml:114`) remains for fuzz complement.

# References

- Creusot homepage & user guide — <https://creusot.rs/>
- Creusot repository — <https://github.com/creusot-rs/creusot>
- Creusot 0.12.0 prove-by-default — <https://devlog.creusot.rs/v0-12-0/>
- Why3 platform — <https://why3.org/>
- ISO 7064 MOD 11-2 / MOD 97-10 — citations already in `mod.rs:2003,2021`
- NOID Check Digit Algorithm — <https://metacpan.org/dist/Noid/view/noid#NOID-CHECK-DIGIT-ALGORITHM> and `mod.rs:1978`
- Crockford Base32 — <https://www.crockford.com/base32.html> and `util/mod.rs:845`
