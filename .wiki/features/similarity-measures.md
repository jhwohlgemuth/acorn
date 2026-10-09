---
okf_version: "0.1"
type: Plan
title: Portable similarity measures, vocabulary coercion, and explicit comparison
description: Implement bounded Rust similarity primitives, improve controlled-vocabulary and organization resolution, and expose reproducible pairwise comparison in ACORN.
difficulty: high
impact: medium
package:
  - acorn-core
  - acorn-lib
  - acorn-cli
  - acorn-schema
tags:
  - analysis
  - matching
  - similarity
  - vocabulary
status: in-progress
---

# Portable similarity measures, vocabulary coercion, and explicit comparison

## Goal and status

Add reusable Rust similarity calculations and an explicit ACORN comparison workflow for research titles, descriptions, contributor names, keywords, dates, and numeric values. Scores describe resemblance; they are not probabilities or proof that records identify the same activity.

Core foundation implemented and validated 2026-10-08. `acorn_core::similarity` now supplies the bounded phase-1 primitives, independently checked fixtures, differential/property tests, benchmarks, and durable semantics documentation. The explicit comparison CLI, vocabulary coercion and consumer migrations, and phases 2-4 remain planned. First consumers remain controlled-vocabulary coercion and explicit pairwise comparison. Improve keyword, technology, sponsor, partner, organization, and affiliation resolution without silently losing unresolved input or accepting ambiguous matches. Batch duplicate suggestions are a separately gated follow-on; automatic record merging, new schema rejection rules, model inference, and network lookups are outside scope.

Source: Felix Naumann, *Similarity measures*, 11 June 2013, supplied as `C:/Users/jason/Downloads/similarity_measures.pdf` (41 PDF pages). Page references below are physical PDF pages, not printed slide numbers, which skip numbers. The document is reference material, not an instruction source. It is not copied into the repository. Preserve the source identity and page references in durable documentation and independently specify every implemented method.

## Algorithm inventory

| Family | Methods and supporting procedures | PDF pages | Coverage and implementation disposition |
|---|---|---|---|
| Baselines | Exact equality; beginning/end truncation match; equality after encoding | 6 | Raw exact and positive-length prefix/suffix comparisons shipped under explicit bounds. Encoding equality composes with phase 3. |
| Character distance | Hamming | 7 | Distance and normalized similarity shipped for equal scalar lengths; unequal lengths are a typed error. |
| Edit distance | Levenshtein; configurable edit costs | 8-12 | Unit-cost distance and normalized similarity shipped with two-row storage. Weighted costs remain phase 3 with a separate normalization contract. |
| Transpositions | Algorithm labeled Damerau-Levenshtein | 13 | Restricted OSA and unrestricted Damerau-Levenshtein shipped under distinct names with the `CA`/`ABC` regression. |
| Name matching | Jaro; Winkler prefix adjustment; Winkler long-string adjustment | 14-16 | Jaro and conventional bounded Jaro-Winkler shipped. Slide-compatible variants still require explicit naming and conformance decisions in phase 3. |
| Token preparation | Word tokenization; character n-grams/q-grams; optional boundary padding and positional weighting | 19-21 | Explicit raw/lowercase word and character n-gram preparation shipped, including collision-checked caller-supplied boundary markers. Positional weighting remains deferred. |
| Set similarity | Overlap coefficient; Jaccard; Dice/Sorensen-Dice | 22 | Deduplicated overlap, Jaccard, and Sorensen-Dice shipped with explicit empty-set semantics. Multiset variants require distinct names. |
| Phonetic encoding | Soundex; Cologne phonetics (Kolner Phonetik); original Metaphone | 24, 26, 28 | Rules supplied. Phase 3, with language and normalization contracts and authoritative conformance fixtures. |
| Phonetic extensions | Double Metaphone; Metaphone 3 | 27 | Named and characterized, not specified. Double Metaphone requires an external specification before phase 3. Metaphone 3 is deferred pending specification, license, and demonstrated need. Accuracy claims in slides are not ACORN acceptance targets. |
| Hybrid token matching | Monge-Elkan | 30-31 | Mean of each left token's best right-token score. Phase 2; directional by default, with separately named symmetric mean if needed. |
| Fuzzy set matching | Extended Jaccard | 32 | Thresholded token-pair construction shown. Phase 2 only after ambiguity is resolved; do not silently substitute one-to-one assignment. |
| Vector similarity | Cosine; term frequency and inverse document frequency; TF-IDF weighting | 33-35 | Cosine and weighting described. Phase 2 with an explicit corpus snapshot and weighting formula. TF-IDF is a representation, not an independent pairwise similarity. |
| Weighted hybrid | Soft TF-IDF | 36 | Sketch given; formula requires clarification. Phase 2 uses a cited reference implementation/specification and reports the variant. |
| Numeric | Absolute-difference similarity; percentage-difference similarity | 38 | Finite percentage difference plus cutoff-based absolute and percentage similarities shipped with checked arithmetic results. |
| Temporal | Day-distance similarity; age-percentage similarity; swapped-day/month and month-error rules | 39 | Date/age methods and heuristic examples. Phase 3; heuristics opt-in, with explicit calendar/reference date. |
| Sequence alignment | Smith-Waterman; Smith-Waterman-Gotoh; Needleman-Wunsch | 3 and repeated overview, 41 | Named only; Needleman-Wunsch appears in the package list. Phase 4 after primary algorithm specifications and scoring conventions are selected. |
| Spatial/domain rules | Geographic distance through a projection; domain-specific rules | 39, 41 | No geographic algorithm or general rule engine specified. Defer a concrete geographic method until coordinate system, units, and research use case exist. |

SecondString, SimMetrics, and GeographicLib on page 41 are implementation packages, not additional similarity algorithms. Clustering, partitioning, filtering, and learning on page 2 are research categories, not specified algorithms to implement.

## Current repository evidence and boundaries

- [acorn-core](../../crates/acorn-core/src/similarity/mod.rs) now exposes pure similarity primitives behind its existing `alloc` feature. Production code adds no dependency; `strsim` and Criterion are dev-only oracle/benchmark dependencies.
- [acorn-lib analyzer](../../crates/acorn-lib/src/analyzer/mod.rs) exposes supplied-content analysis separately from host orchestration. [Its manifest](../../crates/acorn-lib/Cargo.toml) defines `analyzer = ["alloc"]`, while `analysis` brings host capabilities. Put research-field policies and corpus preparation here, with file loading in host/CLI adapters.
- The new core module exposes validated `SimilarityScore`, typed errors, configurable limits, raw distances, normalized scores, explicit preprocessing, and distinct OSA/unrestricted Damerau variants. No CLI, analyzer, schema, or existing matching behavior changed in this slice.
- [Check results](../../crates/acorn-lib/src/analyzer/check.rs) have established categories and summaries. Pair comparison should initially return a separate typed result, avoiding changes to every check consumer. Advisory batch checking needs a later explicit analyzer-contract integration slice.
- [Makefile](../../Makefile) supplies `check-portable` and `check-wasm`; [WASM bindings](../../crates/acorn-wasm/src/lib.rs) already wrap portable supplied-content analysis. Browser bindings for similarity are optional follow-on work, not a prerequisite for portable Rust.
- The core slice started from a clean tree and used the workspace-graph, library-roots, and documentation-generation surfaces serially. Recheck current source and locks before later slices.

## Controlled-vocabulary coercion and related consumers

### Verified behavior and gaps

| Current surface | Evidence and implication |
|---|---|
| [ControlledVocabulary::normalize and list resolver](../../crates/acorn-schema/src/types.rs) | Normalization calls the CSV resolver for each value, drops unresolved results with `filter_map`, and sorts/deduplicates accepted canonical strings through `BTreeSet`. The result has no per-input explanation or alternatives. |
| Native versus portable resolution | The `std` list resolver uses Nucleo patterns, path-oriented configuration, any positive match, and maximum-score selection. The non-`std` implementation uses its own exact/acronym/prefix/suffix/subsequence rules. Equal-score selection differs (last versus first encountered candidate), so ordering and feature flags can affect coercion. Both inspect only the first four row values. |
| [Vocabulary assets and parser](../../crates/acorn-schema/src/vocabulary.rs) | Assets have different row meanings. Technology rows include canonical value, a namespaced classification, display alias, URL, license, and a flag; sponsors include names, acronyms, and identifiers. Matching the first four columns conflates aliases with metadata. The parser splits on commas and maps missing assets to an empty collection. |
| [Shared text normalization](../../crates/acorn-core/src/util/alloc.rs) | `MarkdownSupport::normalize` removes selected punctuation and expands `&`. Vocabulary-specific policy should not be hidden in a general Markdown helper or globally changed for unrelated callers. |
| [ResearchActivity formatting](../../crates/acorn-schema/src/research_activity/mod.rs) | `Vocabulary::resolve` feeds keyword, technology, partner, and sponsor formatting. Coercion changes affect existing formatting behavior, not just a new comparison command. |
| Organization/contact formatting in `types.rs` | `resolve_from_organization_json` searches the root and two member levels. `ContactPoint::format` replaces an unresolved organization with an empty string and an unresolved supplied affiliation with a default. Preserve supplied evidence and distinguish missing input from failed resolution in the new path. |
| Existing tests | [Schema tests](../../crates/acorn-schema/src/tests/mod.rs) cover aliases, truncated names, and native-only `^alpha`/`alpha$` pattern semantics. [Portable tests](../../crates/acorn-schema/tests/portable_semantics.rs) cover ORNL contact normalization. Pattern interpretation is an existing tested compatibility contract, not something to remove unnoticed. |
| Existing downstream consumers | [Discovery](../../crates/acorn-lib/src/analyzer/host/discovery/mod.rs), [GitLab intake](../../crates/acorn-lib/src/io/api/gitlab/intake.rs), and [repository language enrichment](../../crates/acorn-lib/src/lib.rs) call `ControlledVocabulary::normalize` for keywords/technology. They should share one resolution policy rather than inventing provider-specific similarity thresholds. |

### Resolution contract

Keep scoring primitives in `acorn-core`; keep vocabulary assets, column semantics, and canonicalization policy in `acorn-schema`, which already depends on core. Host enrichment and presentation consume structured resolution results through `acorn-lib`. Do not introduce a schema-to-lib dependency.

1. Introduce an additive, pure `resolve_with_policy`-style API returning an ordered outcome per original input: exact, alias, fuzzy-accepted, ambiguous, unmatched, or invalid/resource-limited. Include original value, vocabulary identity/revision, canonical value when accepted, matched alias, method/variant, score, runner-up margin, and bounded candidate explanations. These are proposed API names, to freeze in phase 0. Batch errors must distinguish unknown/malformed vocabulary from valid vocabulary with no match.
2. Parse each asset into explicit entries with canonical key/label, approved aliases, and separate metadata. Exact canonical match precedes exact declared alias, then unique policy-normalized equality, then fuzzy ranking. Duplicate aliases mapping to different canonical entries are ambiguous; deterministic ordering is not permission to pick a winner. Deduplicate aliases by canonical identity before computing the runner-up margin. Do not treat `---`, URLs, ROR identifiers, enum paths, license text, or booleans as fuzzy aliases. If identifier lookup is useful, expose exact lookup under an explicitly typed field.
3. Freeze normalization per vocabulary. Preserve meaningful technology punctuation (`c`, `c++`, `c#`, `.NET`) and test collisions caused by punctuation removal. Acronyms such as AI, HPC, ORNL, NREL, and IU should resolve through declared aliases; short unknown inputs should not acquire automatic acceptance merely because they form a subsequence. Derived acronyms can be suggestions, subject to collision checks.
4. Evaluate normalized Levenshtein/OSA for typos and transpositions, and token Jaccard/Dice or token-aware comparison for multiword labels. Jaro-Winkler is a candidate for organization-name ranking, not a universal default. Calibrate methods separately for each vocabulary and input-length band. Long organization suffixes, shared prefixes, department names, and program versions need hard-negative fixtures. Do not average unrelated scores without validating the combined policy. Phonetic and corpus-based methods remain optional experiments; they must demonstrate added value before participating in coercion.
5. Accept a fuzzy result only when its vocabulary-specific threshold and separation from the best distinct canonical alternative both pass. Exact alias resolution is independent of a fuzzy threshold. An ambiguity or budget exhaustion returns candidates/diagnostics rather than changing the value. Determine thresholds from held-out labeled inputs; record the false-coercion rate, coverage, and abstention rate. Similarity scores are not confidence probabilities.
6. Preserve every original input and its result even when several inputs resolve to the same canonical value. A separate accepted-values projection may retain the existing sorted/deduplicated output. Formatting/enrichment must retain unresolved or ambiguous source values in the field when allowed, otherwise in an explicit loss/provenance report; never silently discard them. Keep missing affiliation defaulting separate from a supplied affiliation that failed to resolve. Organization lookup should traverse all supported hierarchy depths with path/type evidence and distinguish identical labels in different branches.
7. New coercion treats input as literal text on both native and alloc/WASM builds. Keep Nucleo pattern search as a separately named legacy/search behavior where compatibility requires it; ordinary metadata must not accidentally execute `^`, `$`, `!`, or other pattern syntax. Do not silently reinterpret the existing tested API. Characterize old/new output on committed fixtures, publish differences, and migrate call sites explicitly. Retain a compatibility wrapper while consumers move to the report-bearing API; document any later deprecation.
8. Prepare a vocabulary once per batch, indexing exact canonical values and aliases before bounded fuzzy scoring. Key reusable prepared data by vocabulary content/revision and policy version, without a host-only global cache. Bound entries, aliases, per-input work, total batch work, and returned candidates. Candidate pruning must preserve measured recall and report incomplete searches; never declare a unique automatic match from a truncated candidate search.

### Related integration scope

| Consumer | Planned behavior and boundary |
|---|---|
| RAD keyword/technology/partner/sponsor formatting | Use the shared resolver and return accepted changes plus unresolved/ambiguous diagnostics. Preserve canonical spelling and idempotence. A report-bearing format API can be additive while the existing return shape remains available. |
| Contact organization and affiliation | Resolve against typed hierarchy entries and explicit aliases; preserve supplied unknown values, and derive a parent affiliation only from an accepted organization and documented hierarchy rule. No fuzzy correction of names, email, telephone, or person identifiers is implied. |
| Gather, CFF/RAiD/cite-as enrichment, GitLab intake, repository languages | Reuse vocabulary policy, retain raw provider evidence and field locators, and expose unresolved values through existing candidate/provenance reports. Coordinate destination/merge changes with the enrichment plan; no fuzzy overwrite of an already accepted conflicting assertion. |
| Validation and correction suggestions | Add deterministic vocabulary alternatives through existing diagnostics/fix presentation where it fits, without making similarity an additional validity rule or requiring inference. Writing a suggested replacement follows the caller's existing explicit formatting/fix workflow. Registering new diagnostic shapes requires analyzer/check coordination. |
| Vocabulary maintenance | Offline audit for duplicate canonical keys, aliases shared across entries, normalization collisions, malformed rows, placeholder leakage, and near-duplicate terms. Near-duplicate results are advisory; do not merge vocabulary concepts or edit assets automatically. Validate actual CSV schema/quoting requirements rather than changing the shared CSV helper indiscriminately. |
| Other search/identity paths | Audit reuse opportunities when encountered, but keep interactive fuzzy search distinct from canonicalization. PID/checksum/URI parsing, license identifiers, contributor identity, and crosswalk field mappings retain exact/domain-specific rules; lexical resemblance may only supply explicitly requested suggestions. |

## Mathematical contracts to freeze first

1. **Input units:** use Unicode scalar values for character algorithms, never UTF-8 byte offsets. Raw matching is case-sensitive and does not normalize implicitly. Offer explicitly named preprocessing (initially raw or lowercase plus whitespace tokenization); evaluate Unicode normalization/case-folding dependencies separately. Do not equate lowercase with full Unicode case folding. Preserve punctuation and diacritics unless the chosen policy says otherwise.
2. **Scores versus distances:** expose integer edit distances separately from normalized similarities. Unit Levenshtein similarity is `1 - d / max(lengths)` with empty/empty defined as 1. Never apply the slide's generic `1/d` at zero or assume `1-d` is normalized. Only methods with proven metric properties may drive metric-tree pruning.
3. **Empty/missing data:** primitive exact/string/set matching uses empty/empty = 1 and one-empty = 0 where mathematically applicable. Missing research fields are unavailable evidence, not empty strings or perfect matches. Zero-norm cosine and zero-weight TF-IDF return an explicit undefined-score error. Empty corpus, invalid thresholds, zero n-gram width, unequal vector dimensions, and non-finite numbers are typed errors.
4. **Sets and token sequences:** set methods deduplicate; Monge-Elkan keeps a documented token sequence policy. N-grams operate on scalars with explicit padding; an unpadded input shorter than n yields no grams. Boundary markers must not collide with literal input. Prefix/suffix comparison uses a positive requested scalar count and rejects inputs shorter than it.
5. **OSA versus Damerau-Levenshtein:** implement and test independently. Include `CA` versus `ABC` (OSA 3, unrestricted distance 2). The slide recurrence must not be advertised as full Damerau-Levenshtein. [strsim documentation](https://docs.rs/strsim/0.11.1/strsim/) distinguishes these variants and can supply a differential test oracle after compatibility review.
6. **Jaro/Winkler:** clamp the search window at zero; handle zero matches; count transpositions as half the mismatches between matched subsequences. For the conventional Winkler variant, freeze a maximum prefix of four, scale 0.1, and an explicit boost threshold (proposed 0.7, boost only above it). Page 16 instead shows an uncapped prefix and a long-string boost. Do not reproduce scores above 1 or silently clamp them; keep slide compatibility gated until a bounded definition is specified.
7. **Extended Jaccard:** page 32 counts all qualifying token pairs, allowing multiple matches per token. Its unmatched-token notation is ambiguous. Specify unmatched tokens as having no qualifying edge and test many-to-many cases. If a one-to-one matching variant is desired, name it separately and specify maximum-cardinality versus maximum-weight matching and deterministic tie-breaking.
8. **TF-IDF/Soft TF-IDF:** fix corpus document boundaries, count a term once per document for document frequency, freeze TF/IDF/log/smoothing choices, L2 normalization, unseen-token policy, threshold inclusivity, and corpus identity. Page 36 writes the right weight using the left token, even when a different right token is the best match. Resolve against [Cohen, Ravikumar, and Fienberg](https://www.cs.cmu.edu/~wcohen/postscript/kdd-2003-match-ws.pdf) and reference code before implementation. Best-match token reuse can defeat naive boundedness assumptions; do not silently clamp or label an unproven score as normalized. Preserve directional results or use a separately documented symmetric construction.
9. **Numeric/time:** absolute similarity is `max(0, 1 - abs(a-b)/cutoff)` for a finite positive cutoff. Percentage comparison uses `100 * abs(a-b)/max(abs(a),abs(b))`, with both zero treated as equal; guard overflow. Dates use calendar-day differences, and ages require a caller-supplied reference date. Special date-error scores are policies, not inferred truth.
10. **Phonetics/alignment:** choose Soundex vowel/H/W handling explicitly; the slide is a simplified rule order. Cologne requires umlaut/eszett and initial-vowel decisions. Double Metaphone returns primary and alternate encodings. Alignment methods need match/mismatch scores, gap-open/extension costs, local versus global behavior, and raw-score/normalization decisions before they have a stable API.

## Rust design and resource limits

Prefer small pure functions over a generic matching framework. Borrow inputs; use explicit configuration only for methods that require it. Proposed types are `SimilarityError`, bounded `SimilarityScore` for proven [0,1] methods, and separate raw/directional scores for other methods. Keep distance, score, algorithm/variant, preprocessing, and corpus metadata distinguishable in results. Never serialize NaN or infinity.

Use two-row Levenshtein with O(mn) time and O(min(m,n)) auxiliary DP space; Unicode scalar conversion may add O(m+n) storage. OSA needs sufficient previous-row state; unrestricted Damerau requires its own last-occurrence bookkeeping. Token-set operations should use stable sorted sets or deterministic output ordering. Cosine uses stable finite arithmetic; do not let square roots/logarithms accidentally introduce a host-only dependency in no_std builds. Evaluate existing portable math support before adding a dependency.

Set caller-configurable, enforced limits before allocating: proposed defaults are 4,096 scalars per string, 4,000,000 DP cells/work units, 256 tokens per hybrid input, 65,536 token pairs, and a shared cumulative budget for nested token comparisons. Limits are provisional until benchmarked. Reject over-budget work explicitly; never truncate inputs silently. Full-matrix algorithms need a separate byte budget and checked dimension multiplication. Corpus construction must bound document count, token count, vocabulary size, and retained bytes, with deterministic errors.

Evaluate `strsim` as a test oracle or implementation dependency only after checking Unicode semantics, variant fidelity, license, MSRV, dependency cost, and alloc/WASM support. It is not selected as a production dependency by this plan. Basic methods should not require external services or model downloads.

## Delivery sequence and validation

### Phase 0: specification and fixtures

Freeze the contracts above, record which methods are symmetric/metric/bounded, and create independently checked fixtures. Recalculate slide examples rather than copying rounded or inconsistent values. Include Hamming `peter/pedro = 3`, Levenshtein `Jones/Johnson = 4`, `Paul/Pual = 2`, OSA `Paul/Pual = 1`, Jaro `PAUL/PUAL = 11/12`, and Soundex `PAUL = P400`. Resolve inconsistent token examples from explicit token lists. Select primary specifications for named-only methods before scheduling their implementation.

### Phase 1: useful portable primitives and first consumer

Implement exact/truncated comparisons, Hamming, Levenshtein, OSA, full Damerau-Levenshtein, Jaro, conventional Jaro-Winkler, word/n-gram preparation, overlap/Jaccard/Dice, and absolute/percentage numeric similarity. Expose an explicit proposed `acorn compare` command with a required method, two supplied values (or explicit file operands), and text/JSON output. Confirm that command name is free in phase 0 and follow the current command registry rather than an assumed main.rs match layout.

The portable primitive portion is complete. The `acorn compare` command and its result envelope remain the next independent phase-1 slice and require the CLI/documentation locks.

The command result reports variant, preprocessing, optional raw distance, score semantics, and relevant parameters. Invalid input and budget failures are errors; a low similarity is a successful comparison. File operands use existing host I/O and work offline. No automatic interpretation of arbitrary arguments as paths or URLs. Use existing terminal labels and command error conventions. Add command discovery/dispatch/help tests and update generated help.

### Phase 1A: vocabulary resolution and consumer migration

Schedule this immediately after the required phase-1 edit/token primitives, independently of the comparison CLI and phases 2-4. First characterize all embedded vocabularies and existing native/portable outputs, then freeze typed entries and the report-bearing resolver. Implement exact/alias precedence, bounded ranking, ambiguity/abstention, literal-input semantics, and a prepared vocabulary shared across a batch. Produce an offline vocabulary audit before changing asset interpretations.

Migrate RAD formatting and contact organization/affiliation through additive report-bearing APIs, then discovery/intake/repository-language enrichment. Record compatibility changes individually, including values previously dropped, ambiguous winners, punctuation collisions, and Nucleo patterns. Preserve existing legacy behavior behind an explicit compatibility boundary until each caller has a tested migration. Add deterministic suggestions to existing diagnostics only after their contract and exact write set are claimed; a new check category is not required by this plan.

Evaluation fixtures must include every canonical entry and declared alias, known current examples (`AI`, `HPC`, `ORNL`, `NREL`, `IU`, `Kitware, Inc.`), synthetic typos/transpositions, reordered multiword names, ambiguous acronyms, short unknowns, empty input, punctuation-sensitive technologies, Unicode, placeholders/URLs/IDs, equal-score candidates, hierarchy collisions, and vocabulary rows with more than four fields. Keep accepted aliases separate from held-out fuzzy evaluation. Measure false acceptance, ambiguity/abstention, top-k suggestion recall, and runtime by vocabulary; freeze explicit release targets from the baseline before switching production callers. Native and portable tests must produce identical structured outcomes for the new policy, independent of input asset ordering.

### Phase 2: corpus and hybrid methods

Add cosine, TF-IDF corpus preparation, Monge-Elkan, Extended Jaccard, and Soft TF-IDF after their contracts pass phase 0. Corpus inputs must be explicit; never scan a database implicitly during pair comparison. Reuse prepared weights across comparisons. Validate asymmetric examples, repeated-token collisions, all-common-term zero weights, out-of-vocabulary terms, deterministic ties, and threshold boundaries. Compare a bounded randomized corpus against an independent implementation with the same variant.

### Phase 3: phonetic and field policies

Add Soundex, Cologne, original Metaphone, specified Double Metaphone, weighted edit costs, and date/age policies. Resolve the slide-specific Winkler variants before implementing them. Expose encoding outputs distinctly from their equality scores. Research-field adapters select methods explicitly; identifiers retain canonical exact matching. Document language limitations without inferring contributor identity from name resemblance.

### Phase 4: externally specified alignment methods

Implement Needleman-Wunsch, Smith-Waterman, and Smith-Waterman-Gotoh using vetted primary specifications and independent fixtures. Cover affine-gap conventions, empty inputs, negative scores, local restart, and resource limits. Return scores without traceback initially; adding alignment transcripts requires an explicit memory contract. Metaphone 3, positional n-gram weighting, geographic methods, and a general rule engine remain deferred unless a concrete contract and consumer justify them.

### Follow-on gate: research duplicate suggestions

Evaluate titles with token similarity, names with Jaro-Winkler/phonetics, descriptions with TF-IDF, and dates with explicit cutoffs on a labeled research-record corpus. Measure precision, recall, false positives, and candidate coverage by field and language; select thresholds from held-out data, not the lecture. Missing fields stay missing. Freeze any combined-score policy and blocking strategy separately; avoid unbounded all-pairs comparison. Only after this evidence should a new atomic plan add opt-in advisory findings to `check`. No automatic merge or change to default validation behavior is authorized by this plan.

## Core foundation validation

Validated on 2026-10-08:

- `cargo test -p acorn-core`: 38 unit, 15 similarity integration, and 12 passing doc tests (one ignored); the similarity suite also passes with `--no-default-features --features alloc --test similarity`.
- `cargo clippy -p acorn-core --all-targets --all-features -- -D warnings`; featureless, `core`, and `alloc` checks; `core` and `alloc` WASM checks; Rust documentation; and the mdBook all pass.
- Repository gates `make lint`, `make test`, `make check-portable`, `make doc`, and `make book` pass. The full Nextest run executed 2,284 tests successfully with 17 skipped.
- Randomized differential tests agree with `strsim 0.11.1` for Hamming, Levenshtein, OSA, unrestricted Damerau-Levenshtein, Jaro, and conventional Jaro-Winkler. Property tests cover symmetry and the claimed Levenshtein/Damerau metric triangle inequality.
- Criterion's 10-sample local baseline measured 1,024-scalar Levenshtein at 4.91-5.03 ms, OSA at 8.24-8.42 ms, unrestricted Damerau-Levenshtein at 11.28-11.39 ms, and Jaro-Winkler at 208-213 µs. Unicode Levenshtein measured 436-452 µs; character n-grams 25.0-25.3 µs; word tokens 10.4-10.6 µs; and 256-by-256 Jaccard 19.5-19.7 µs. These are development-host observations, not portable performance guarantees.
- macOS `xcrun` emitted sandbox cache warnings during linked test/benchmark builds, but every command completed successfully.

## Implementation write boundaries and locks

This planning change owns only this file and its entries in `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, and `.wiki/parallel-execution.md`.

Before each implementation assignment, replace the proposed boundaries below with a complete exact create/update/delete list verified against the then-current tree:

The completed core foundation slice created `crates/acorn-core/src/similarity/{mod,error,edit,numeric,token}.rs`, `crates/acorn-core/tests/similarity.rs`, `crates/acorn-core/benches/similarity.rs`, `tests/fixtures/similarity.json`, and `docs/src/concepts/similarity.md`; it updated `crates/acorn-core/src/lib.rs`, `crates/acorn-core/Cargo.toml`, `Cargo.lock`, `docs/src/SUMMARY.md`, this plan, and `.wiki/index.md`. It created or deleted no other tracked paths.

- Remaining portable leaves: proposed `crates/acorn-core/src/similarity/{vector,hybrid,phonetic,alignment}.rs`. Assign exact leaf files per phase; any further root registration requires the library-roots lock.
- Domain adapter: proposed `crates/acorn-lib/src/analyzer/similarity.rs` and its registration in `crates/acorn-lib/src/analyzer/mod.rs`; library-roots lock. No existing check categories change in phases 0-4.
- Vocabulary implementation: proposed `crates/acorn-schema/src/vocabulary/resolution.rs`; update `crates/acorn-schema/src/vocabulary.rs`, `crates/acorn-schema/src/types.rs`, and `crates/acorn-schema/src/research_activity/mod.rs` under schema-roots/library-roots locks. Tests: `crates/acorn-schema/src/tests/mod.rs` and `crates/acorn-schema/tests/portable_semantics.rs`; proposed dedicated fixtures `tests/fixtures/vocabulary-resolution.json`. Claim specific asset files only if the audit requires corrections; do not broadly rewrite embedded vocabularies.
- Vocabulary consumers: `crates/acorn-lib/src/analyzer/host/discovery/mod.rs`, `crates/acorn-lib/src/io/api/gitlab/intake.rs`, and `crates/acorn-lib/src/lib.rs`, with focused existing tests claimed by exact filename before migration. Coordinate discovery/intake/provenance edits with enrichment and gather owners. Diagnostics/fix integration requires a separately enumerated write set and analyzer/check lock within phase 1A; it does not authorize wholesale fix-service changes.
- CLI: proposed `crates/acorn-cli/src/commands/compare/mod.rs` and adjacent tests; resolve exact registry/parser/dispatch files during phase 0 and claim the CLI-contract lock before editing them. Generated help and book integration take the documentation-generation lock.
- Fixtures/documentation: extend `tests/fixtures/similarity.json` and `docs/src/concepts/similarity.md` only for the owning algorithm slice. CLI work must separately identify its exact command page and generated help output. Keep examples synthetic or explicitly approved for redistribution.
- Any root/crate manifest or lockfile change takes the workspace-graph lock and follows the dependency-capabilities prerequisite. CI/Makefile changes require CI/build ownership. Benchmarks must name their file and registration explicitly before work starts.

Dependencies: pure specification/fixture work can proceed independently. Manifest additions follow [dependency capabilities](../refactors/portability/dependency-features-and-macos-size.md). CLI integration coordinates with [CLI composition](../refactors/core/cli-composition-boundary.md). Any later check integration serializes with CEL and enrichment under the analyzer/check contract. WASM exposure coordinates with the browser tools plan only if separately scheduled.

Vocabulary phase 1A depends only on its selected phase-1 primitives, not TF-IDF, phonetics, alignment, or a new CLI. It owns resolution semantics; [structured enrichment](../refactors/core/research-activity-enrichment-integration.md) continues to own assertion provenance, destination fields, and merge policy. Freeze the resolver report before those plans consume it. Schema roots serialize with dPID/PID work; discovery paths serialize with [gather extraction](gather-extraction.md). Existing source-asset loaders remain offline.

## Acceptance criteria

- [ ] Vocabulary resolution has typed column roles, canonical/alias precedence, distinct-canonical ambiguity handling, and calibrated acceptance/margin rules. No metadata column or placeholder accidentally becomes a fuzzy alias.
- [ ] New resolver outcomes agree across native and portable feature builds and remain stable under row/alias permutations. Legacy Nucleo pattern behavior has an explicit migration boundary and regression coverage.
- [ ] Every supplied vocabulary value has an outcome. Ambiguous/unmatched inputs survive in fields or explicit provenance/loss reports; contact formatting no longer silently erases supplied unknown organizations or replaces supplied unknown affiliations in the migrated path.
- [ ] RAD formatting, contact resolution, discovery, GitLab intake, and repository-language enrichment use the shared policy with fixture-backed compatibility evidence, idempotence, accepted-value deduplication, and preserved per-input provenance.
- [ ] The vocabulary audit detects alias/normalization collisions and malformed assets; threshold quality targets and search-budget behavior pass before fuzzy coercion is enabled for each vocabulary. Optional diagnostics use existing correction workflows and never automatically merge concepts or records.

- [x] Every inventory row is mapped to a shipped, explicitly specified method or an explicit deferral; named-only methods are never presented as fully specified by the slides.
- [x] Phase-1 core API contracts, normalization rules, variant names, errors, and resource budgets are documented and tested before public exposure. Later phases retain their own contract gates.
- [x] Phase-1 core unit and property tests cover identity, bounds, promised symmetry, valid metric properties, empty inputs, Unicode, repeated tokens, and invalid parameters.
- [ ] Differential tests use matching variants; OSA/full-Damerau separation, directional Monge-Elkan, fuzzy-token collisions, and Soft TF-IDF normalization have dedicated regression cases.
- [ ] Benchmarks show bounded behavior for long strings, repeated tokens, nested comparisons, and growing corpora; resource-limit failures are deterministic.
- [ ] CLI text and JSON results agree with library calls; malformed parameters and file errors fail cleanly; offline execution requires no network or model.
- [ ] Focused tests pass for each slice. After all implementations are stable, one integration owner runs `make lint`, `make test`, and `make check-portable` (including the repository WASM checks). Run `make help` for CLI changes. Add focused no-default-feature/alloc checks for the new core module.
- [ ] Durable API/user documentation explains suitable research fields, unsupported variants, score semantics, corpus reproducibility, and why scores do not establish identity.
- [ ] Final report records actual validation results. When all scheduled phases close, move durable inventory/decisions into documentation and delete this plan and its index/network/lane entries; create separate plans for any accepted follow-ons.
