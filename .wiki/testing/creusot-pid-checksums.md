---
type: Plan
title: "Creusot verification for PID checksum calculations"
description: "Formally verify the four shared PID check-digit algorithms and their PID-specific callers using Creusot, Why3, and SMT solvers."
difficulty: high
impact: medium
package: [acorn-lib]
tags: [testing, verification, creusot, pid, checksum, why3]
status: planned
priority: next
---

# Goal

Prove panic- and overflow-freedom plus functional correctness for the shared check-digit algorithms in `crates/acorn-lib/src/schema/pid/mod.rs` and connect those proofs to the checksum-bearing PID modules that call them. Preserve the refactored folder layout and existing visibility: proofs must not make private helpers public or move PID-specific parsing back into the base module.

Network access, regex matching, serialization, SHA-256 identifier hashing, and PID types without checksum arithmetic are outside the initial proof boundary.

# Current evidence

## Refactored PID layout (verified 2026-08-23 against `acorn-lib` `0.1.75`)

- Shared traits, `PID`, `Identifier`, the beta-numeric alphabet, and shared free functions live in `crates/acorn-lib/src/schema/pid/mod.rs`.
- Every concrete PID has its own `pid/<kind>/mod.rs` implementation and adjacent `pid/<kind>/tests.rs` suite. Checksum-bearing callers are `ark`, `isbn`, `isni`, `orcid`, and `ror`.
- Direct vectors for every shared check-digit function are centralized in `crates/acorn-lib/src/schema/pid/tests.rs:12-96`; PID-specific parsing, validation, and trait contracts remain in the adjacent test modules.
- `cargo check -p acorn-lib --no-default-features --features alloc` passes. The proof target can therefore stay on the portable `alloc` surface.

## In-scope proof surface

| Shared function or helper | Current location | PID callers | Current arithmetic and test evidence |
|---|---|---|---|
| `mod_10_or_11_check_digit` (private) | `pid/mod.rs:778` | `ISBN::check_digit` and `ISBN::is_valid` in `pid/isbn/mod.rs` | ISBN-10 MOD 11 and ISBN-13 MOD 10; all multiplication, addition, subtraction, and remainder operations are checked; direct vectors at `pid/tests.rs:12,74` and wrapper cases in `pid/isbn/tests.rs` |
| `mod_11_2_check_digit` (private) | `pid/mod.rs:847` | `ISNI` and `ORCID` check/validation paths in `pid/isni/mod.rs` and `pid/orcid/mod.rs` | ISO 7064 MOD 11-2; checked fold; non-decimal payload characters currently contribute zero; direct vectors at `pid/tests.rs:20,80` plus both adjacent suites |
| `mod_97_10_check_digit` (private) | `pid/mod.rs:887` | `ROR::check_digit` and `ROR::is_valid` in `pid/ror/mod.rs` | first six compact Crockford characters, `base32_crockford_decode`, checked multiply/remainder/complement, two-digit formatting; direct vectors at `pid/tests.rs:26,86` and ROR cases in `pid/ror/tests.rs` |
| `noid_check_digit` (public) | `pid/mod.rs:924` | `ARK::check_digit` in `pid/ark/mod.rs` | NCDA radix 29; position, ordinal, accumulation, and reduction use checked operations; direct vectors at `pid/tests.rs:33,92` and ARK cases in `pid/ark/tests.rs` |
| `BETANUMERIC_DIGITS`, `Betanumeric for char`, `to_betanumeric` | `pid/mod.rs:47,556,943` | NCDA implementation | 29-character alphabet, forward lookup by `position`, reverse lookup by bounded enumeration; direct boundary checks at `pid/tests.rs:34` |
| `base32_crockford_encode`, `base32_crockford_decode`, `crockford_digit` | `crates/acorn-lib/src/util/mod.rs:913,941,972` | ROR checksum dependency | checked decoder arithmetic and Crockford alphabet mapping; utility tests remain the executable oracle |

The old `isbn_check_digit`, `isni_check_digit`, `orcid_check_digit`, and `ror_check_digit` free-function names no longer exist. ISBN, ISNI, ORCID, and ROR expose checksum behavior through their `PersistentIdentifier::check_digit` implementations, while only `noid_check_digit` remains a public free function.

Out of scope for phase 1:

- `Identifier::identifier_hash` in `pid/mod.rs` because it is a `std`-gated SHA-256 path through `ring`.
- Regex-backed `PersistentIdentifierParse::{find_all,from_string,is_valid}` internals except for narrow wrapper lemmas connecting validated ASCII payloads to the shared arithmetic.
- ARK, ISBN, ISNI, ORCID, and ROR formatting rules unrelated to check digits.
- arXiv, DOI, Handle, Patent, RAiD, and SWHID modules, which do not implement checksum arithmetic in this scope.

## Tooling readiness

- No Creusot, Why3, or Coma integration currently exists in the repository.
- Stable Rust remains pinned to `1.95.0`; the existing CI nightly is `nightly-2026-08-01`. Creusot requires its own compatible nightly, Why3, and SMT solver installation; pin the verified combination rather than assuming the fuzz/Miri nightly is compatible. See [Creusot](https://creusot.rs/) and [creusot-rs/creusot](https://github.com/creusot-rs/creusot).
- Workspace Clippy warns on arithmetic side effects, unwraps, indexing, and unchecked conversions. The refactored shared functions now use checked arithmetic and do not carry production `clippy::arithmetic_side_effects` allowances.
- The shared helpers allocate `String`/`Vec<char>` wrappers, but their arithmetic is otherwise `core`-compatible. Private slice- or digit-based proof cores can be extracted if Creusot cannot model the wrappers directly.

## Related work

- [Informational Miri quality checks](miri-quality.md) covers dynamic undefined-behavior checks through public PID APIs; Creusot supplies deductive arithmetic guarantees for the private shared cores.
- [CLI test coverage initiative](index.md) tracks complementary executable tests and does not replace formal contracts.

# Recommendation

## Phase 0 — Pin tools and create the proof target

1. Pin a Creusot-supported nightly, Creusot release, Why3 release, and at least two supported SMT solvers. Document installation in `CONTRIBUTING.md` separately from the Miri nightly.
2. Add canonical `make creusot` and `make creusot-prove` targets for `acorn-lib --no-default-features --features alloc`. Keep normal stable builds and default features unchanged.
3. Add the narrowest conditional dependency/configuration supported by the pinned Creusot release. Contracts belong on private shared helpers or private extracted cores; do not re-export them from `schema::pid` solely for verification.
4. Record a baseline showing that native `cargo check -p acorn-lib --no-default-features --features alloc`, `make lint`, and PID tests pass before proof annotations are added.

## Phase 1 — Define invariant-friendly private cores

1. Attempt contracts on the four current shared helpers first. Extract private arithmetic cores only where `String`, `Vec<char>`, iterator, or formatting support blocks verification.
2. If extraction is required, keep the existing `Option<Vec<char>>` wrappers behaviorally unchanged and place the cores in `pid/mod.rs` beside their wrappers:
   - ISBN-10/13 over validated ASCII digits and a declared algorithm variant.
   - MOD 11-2 over at most the first 15 normalized decimal digits.
   - MOD 97-10 over the decoded six-character Crockford payload.
   - NCDA over beta-numeric ordinals with position computed modulo 29.
3. Model the 29-character beta-numeric alphabet and Crockford alphabet as finite sequences with bounded forward/reverse lookup lemmas.
4. Preserve current invalid-input semantics at wrapper boundaries. In particular, `mod_11_2_check_digit` currently maps non-decimal payload characters to zero; PID-specific validation supplies the stronger character-set precondition.

## Phase 2 — Add contracts and wrapper lemmas

Prove both safety and functional behavior:

- `mod_10_or_11_check_digit`: valid compact ISBN-10 input uses weights 10 through 2 and maps result 10 to `X`; valid ISBN-13 input uses alternating weights 1/3 and produces one decimal digit.
- `mod_11_2_check_digit`: each step equals `(remainder + digit) * 2 mod 11`; the final complement is decimal or `X`; the remainder remains below 11.
- `mod_97_10_check_digit`: Crockford decoding succeeds for a valid six-character ROR payload, checked multiplication by 100 cannot overflow for that bound, and the result is the zero-padded MOD 97-10 complement.
- `noid_check_digit`: the result is the alphabet entry at `Σ((index + 1) * ordinal) mod 29`; checked operations succeed for all finite Rust strings because the accumulator is reduced below 29 on every step.
- Beta-numeric/Crockford lookup: accepted characters map to bounded ordinals and reverse lookup returns the original canonical character.

Add narrow lemmas connecting each private core to the public trait boundary in `pid/ark/mod.rs`, `pid/isbn/mod.rs`, `pid/isni/mod.rs`, `pid/orcid/mod.rs`, and `pid/ror/mod.rs`. Do not pull regex engines or complete parsing implementations into the proof target.

## Phase 3 — Prove and cross-check executable vectors

1. Generate/type-check the Creusot/Coma translation, then discharge all verification conditions with two solvers.
2. Treat `crates/acorn-lib/src/schema/pid/tests.rs` as the shared executable oracle. Encode its beta-numeric and four check-digit case tables as proof examples or theorem checks without deleting the native tests.
3. Retain the adjacent PID suites as wrapper-level regression coverage. Formal proof should complement, not replace, parsing, formatting, trait-order, and validation tests.
4. Run native PID tests, `make lint`, `make test`, and the portable `cargo check` after proof-oriented extraction.

## Phase 4 — CI and maintenance

- Add an informational GitLab `creusot` job with `allow_failure: true` initially.
- Trigger it for `crates/acorn-lib/src/schema/pid/**`, `crates/acorn-lib/src/util/mod.rs`, Creusot configuration, Cargo manifests/lockfile, `Makefile`, and CI changes.
- Keep the Creusot toolchain pin independent but review it alongside the Miri/fuzz nightly update.
- Require proof updates when a shared checksum helper, checksum-bearing PID caller, beta-numeric alphabet, or Crockford decoder changes.

# Validation

1. `cargo check -p acorn-lib --no-default-features --features alloc` remains clean.
2. The pinned Creusot translation command succeeds for the isolated PID proof target with zero unsupported items in the proof boundary.
3. Two configured SMT solvers discharge every checksum verification condition without `admit`, `assume`, blanket ignore attributes, or solver-disable flags.
4. `cargo test -p acorn-lib schema::pid`, `make lint`, and `make test` pass after proof annotations or private-core extraction.
5. A qualifying change under `schema/pid/**` or to the Crockford helpers triggers the informational CI job and exposes its logs without blocking the pipeline.

# Acceptance criteria

1. `make creusot` and `make creusot-prove` are documented, reproducible entry points using pinned tools.
2. All four shared algorithms, beta-numeric lookup, and the Crockford decoder dependency have contracts covering overflow-freedom, bounds, and functional equivalence to their specifications.
3. Wrapper lemmas cover ARK/NOID, ISBN-10/13, ISNI/ORCID MOD 11-2, and ROR MOD 97-10 without making private helpers public.
4. Every case table in `crates/acorn-lib/src/schema/pid/tests.rs` remains a passing native test and has a corresponding proof example or theorem.
5. The refactored `pid/<kind>/mod.rs` plus `pid/<kind>/tests.rs` organization remains intact, with no checksum implementation duplicated into PID-specific modules.

# Risks and tradeoffs

- **Iterator and allocation support:** Creusot may not model the current `String`/`Vec<char>` wrappers or iterator chains directly. Mitigate with small private digit/slice cores, not public API expansion.
- **Parser-to-core gap:** Regex validation is excluded, so wrapper lemmas must state the exact normalized ASCII and length assumptions supplied by each PID module.
- **Formatting proof:** ROR zero-padding creates an allocation boundary. Prove the numeric checksum first and keep a narrow formatting lemma or executable assertion if string formatting is unsupported.
- **Nightly pin drift:** Creusot compatibility may differ from Miri compatibility. Pin and update each toolchain deliberately.
- **Proof maintenance:** Shared helpers reduce duplicate proof work, but changes affect several PID modules. CI path rules and wrapper lemmas make that blast radius explicit.

# References

- Creusot homepage and user guide — <https://creusot.rs/>
- Creusot repository — <https://github.com/creusot-rs/creusot>
- Why3 platform — <https://why3.org/>
- ISO 7064 MOD 11-2 and MOD 97-10 — links in the ORCID/ROR implementation documentation
- NOID Check Digit Algorithm — <https://metacpan.org/dist/Noid/view/noid#NOID-CHECK-DIGIT-ALGORITHM>
- Crockford Base32 — <https://www.crockford.com/base32.html>
