# Similarity and distance

ACORN exposes bounded similarity primitives through `acorn_core::similarity`. They describe resemblance between supplied values; they are not probabilities, confidence estimates, or evidence that two research records identify the same activity.

Character algorithms operate on Unicode scalar values rather than UTF-8 bytes. Raw comparison is case-sensitive and performs no hidden Unicode normalization, punctuation removal, or case folding. Callers may explicitly request Rust Unicode lowercase mapping, full locale-independent Unicode case folding, NFC canonical normalization, or NFKC compatibility normalization before comparison and token preparation.

Raw remains the default because this is a general-purpose API: normalization can change scalar-level distances, and compatibility normalization can collapse distinctions that matter in identifiers, technical notation, and source text. For human-language matching where canonically equivalent spellings should compare alike, opt in to NFC. Prefer NFC over NFKC unless the consumer intentionally treats compatibility characters such as circled digits and ordinary digits as equivalent. Case folding is separate from normalization and should be selected only when case distinctions are irrelevant.

The portable API includes exact prefix and suffix comparisons; Hamming, Levenshtein, weighted Levenshtein, restricted optimal string alignment (OSA), and unrestricted Damerau-Levenshtein distance; Jaro and conventional Jaro-Winkler; word and character n-gram preparation; overlap, Jaccard, and Sorensen-Dice set similarity; cosine and explicit-corpus TF-IDF; Monge-Elkan, collision-checked Extended Jaccard, and Soft TF-IDF; Soundex, Cologne phonetics, and original Metaphone; cutoff-based numeric and calendar-date policies; and bounded global or local sequence-alignment scores. OSA and unrestricted Damerau-Levenshtein are separate variants: for `CA` and `ABC`, their respective distances are 3 and 2.

Normalized edit similarity is `1 - distance / max(lengths)`, with empty/empty defined as 1. Set methods deduplicate their inputs; both-empty sets score 1 and one-empty sets score 0. Conventional Jaro-Winkler uses at most four common prefix scalars, a scale of 0.1, and applies its boost only when Jaro similarity is above 0.7.

Every allocating or nested operation accepts `SimilarityLimits`. The defaults allow 4,096 scalars per input, 4,000,000 comparison work units, and 4,096 collection items. Exceeding a limit is an explicit error; inputs are never truncated silently. Scores use a validated finite type bounded to `[0,1]`, so NaN and infinity cannot be serialized as similarity scores.

The algorithm inventory and examples were independently specified with reference to Felix Naumann, *Similarity measures*, 11 June 2013, a 41-page lecture supplied from `C:/Users/jason/Downloads/similarity_measures.pdf`. Page numbers below are physical PDF pages. The lecture is reference material rather than executable specification: notably, its local transposition recurrence describes OSA rather than unrestricted Damerau-Levenshtein, and its named-only methods require independent specifications.

| Source inventory | Pages | ACORN disposition |
|---|---:|---|
| Exact and truncation baselines | 6 | Shipped as exact, prefix, and suffix comparisons. |
| Hamming, Levenshtein, weighted edit costs, and transpositions | 7–13 | Shipped as Hamming, unit and weighted Levenshtein, restricted OSA, and unrestricted Damerau-Levenshtein. |
| Jaro and Winkler adjustments | 14–16 | Jaro and conventional bounded Jaro-Winkler shipped; the slide-specific uncapped and long-string adjustments remain unsupported. |
| Word and character n-grams | 19–21 | Shipped with explicit preprocessing and optional boundary padding; positional weighting is deferred. |
| Overlap, Jaccard, and Sørensen–Dice | 22 | Shipped with deduplicated-set and empty-set contracts. |
| Soundex, Cologne phonetics, and original Metaphone | 24, 26, 28 | Shipped as standalone encoders and equality comparisons with documented language limits. |
| Double Metaphone and Metaphone 3 | 27 | Deferred because the lecture does not provide an implementable specification; Metaphone 3 also needs compatible licensing and a demonstrated consumer. |
| Monge-Elkan | 30–31 | Shipped as directional and explicitly symmetric variants. |
| Extended Jaccard | 32 | Shipped with deterministic threshold and collision handling rather than implicit many-to-many reuse. |
| Cosine and TF-IDF | 33–35 | Shipped with an explicit, reusable, identity-bearing corpus and versioned weighting formula. |
| Soft TF-IDF | 36 | Shipped with L2-normalized TF-IDF weights and collision-checked unique best matches. |
| Numeric difference policies | 38 | Shipped as finite percentage difference plus cutoff-based absolute and percentage similarities. |
| Calendar date and age policies | 39 | Day distance, optional valid day/month swapping, and explicit-reference age percentage shipped; the underspecified month-error heuristic is deferred. |
| Needleman-Wunsch, Smith-Waterman, and Smith-Waterman-Gotoh | 3, 41 | Shipped from independent primary algorithm specifications with explicit linear and affine gap conventions. |
| Geographic and domain-specific rules | 39, 41 | Deferred until a coordinate system, units, algorithm, and concrete consumer are specified. |

SecondString, SimMetrics, and GeographicLib on page 41 are software packages rather than additional algorithms. Clustering, partitioning, filtering, and learning on page 2 are research categories, not implementable method specifications.

## Method contracts and appropriate use

Choose a method for the evidence represented by a field, not because one score happens to be high:

- Titles and short descriptions can use token sets or directional Monge-Elkan when word overlap is meaningful. Long prose can use an explicit, frozen TF-IDF corpus. Missing fields remain missing rather than becoming empty strings.
- Contributor or organization names can use Jaro-Winkler or a language-appropriate phonetic encoder to rank suggestions. A phonetic match never establishes a person's or organization's identity. Soundex and original Metaphone are English-oriented; Cologne phonetics is German-oriented.
- Keywords and other controlled values should use the controlled-vocabulary resolver below. Identifiers, checksums, PIDs, URIs, emails, and telephone numbers retain exact or domain-specific validation.
- Numeric and date methods require a caller-chosen cutoff. Age comparison also requires an explicit reference date. Optional day/month swapping considers only valid proleptic-Gregorian interpretations and must not be treated as a correction of source truth.
- Needleman-Wunsch returns a global raw alignment score. Smith-Waterman and Smith-Waterman-Gotoh return local raw scores. The affine variant charges `gap_open` for the first gap scalar and `gap_extend` for each continuation. These scores depend on the supplied scoring scheme and are not normalized similarities.

`TfidfCorpus` is constructed from caller-supplied tokenized documents and never scans an ACORN database. Document frequency counts a token once per document. Term frequency is `count / document token count`; the versioned `tfidf-ratio-v1` inverse-document weight is `(document_count + 1) / (document_frequency + 1)`, followed by L2 normalization. The corpus identity hashes ordered document boundaries and tokens, so reports can reproduce the exact snapshot and weighting version. Tokens absent from the corpus use document frequency zero. Empty token lists and zero norms are explicit errors.

Soft TF-IDF uses those L2-normalized weights, an inclusive token threshold, and unique best matches. Extended Jaccard uses the same collision rule. A tied best candidate or reuse of one right token is reported as `FuzzyTokenCollision`; ACORN does not silently choose a row-order winner or claim the many-to-many slide formula. Directional Monge-Elkan averages each left token's best right-token score. A separately named symmetric function averages both directions.

Weighted Levenshtein accepts positive insertion, deletion, and substitution costs and normalizes by the cost of deleting the complete left input and inserting the complete right input. Cosine accepts equal-length finite vectors with nonzero L2 norms. All bounded methods return finite scores in `[0,1]`; alignment functions and raw distances use their natural units.

Soundex retains the first ASCII letter, does not let `H` or `W` reset adjacent-code suppression, and pads to four characters. Cologne phonetics folds German umlauts to their base vowels and `ß` to `S`, retains an initial vowel code, removes later vowel codes, and suppresses repeated codes. Original Metaphone follows Lawrence Philips' 1990 English rules and returns one encoding. The encoding functions are public independently of the binary equality comparisons in `acorn diff`.

The slide-specific uncapped/long-string Winkler variants, Double Metaphone, Metaphone 3, positional n-gram weighting, the month-error heuristic, geographic distance, and a general domain-rule engine are unsupported. Double Metaphone and Metaphone 3 remain deferred because the supplied source names them without a complete implementable specification; Metaphone 3 also requires a compatible license and demonstrated need. Similarity-based duplicate suggestions and automatic record merging are likewise outside this API.

Release-mode Criterion observations on the 2026-10-10 development host were 1.66–1.67 µs for 16-scalar Needleman-Wunsch and 332–333 µs for 256 scalars; 81.3–81.9 µs for a 16-document TF-IDF corpus and 1.077–1.084 ms for 256 documents; 1.186–1.194 ms for the bounded nested Monge-Elkan case; and 183–442 ns for the three phonetic encoders. These measurements are regression observations, not portable performance guarantees. Long edit, repeated-token, nested-comparison, and growing-corpus cases all remained within their explicit work limits.

## Controlled-vocabulary resolution

`acorn_schema::vocabulary::VocabularyResolver` applies these primitives to ACORN's embedded keyword, technology, partner, sponsor, and organization vocabularies. It prepares each vocabulary once per batch, treats input as literal text, and checks canonical values, declared aliases, and typed identifiers before bounded fuzzy ranking. Distinct canonical entries that share an alias or cannot clear the runner-up margin are reported as ambiguous rather than selected by row order.

Every input produces a `VocabularyResolution` with its original value, optional source locator, outcome, accepted canonical value, matched term, method, score, runner-up margin, and bounded alternatives. Reports also identify the vocabulary and policy revisions. The lossless projection substitutes accepted values and retains unmatched, ambiguous, invalid, or resource-limited input; the accepted projection is stable and deduplicated.

Three profiles are available:

- `relaxed` is the default advisory policy across `acorn check`, the shared service, gRPC, TUI, and existing WASM validation binding. It lowers the vocabulary-specific fuzzy score and separation gates but keeps ambiguity and resource safeguards.
- `strict` is the higher-precision policy used by formatting unless its caller selects another mode.
- `off` disables fuzzy scoring while retaining exact canonical, alias, identifier, and normalized matches.

Vocabulary policy revision `vocabulary-v2` uses NFC plus full Unicode case folding for organization, partner, and sponsor names. Keywords and technologies use NFKC plus full folding. Organization punctuation is normalized as part of name matching; technology punctuation is preserved unless an asset declares an alias, so values such as `C++`, `C#`, and `node.js` remain distinct. Keyword spaces and underscores normalize to hyphens.

The resolver preflights total fuzzy work for a batch. If the complete fuzzy stage would exceed its configured ceiling, deterministic exact, alias, identifier, and normalized lookups still run, but every otherwise fuzzy-eligible unresolved input is marked resource-limited. It never accepts a prefix of an over-budget batch, and input reordering cannot change accepted canonical results.

The post-migration release corpus contains 1,465 provenance-labeled compatibility and evaluation cases generated against the historical `nucleo-matcher 0.3.1` behavior at source revision `b4007f43^`. Nucleo is not a production or development dependency. Held-out relaxed results are:

| Vocabulary | Accepted | Precision | Recall | 95% Wilson precision interval | Ambiguous | Protected-negative failures |
|---|---:|---:|---:|---:|---:|---:|
| Keywords | 221 | 100% | 100% | 98.29–100% | 0 | 0 |
| Organizations | 198 | 100% | 100% | 98.10–100% | 0 | 0 |
| Partners | 234 | 100% | 100% | 98.38–100% | 1 | 0 |
| Sponsors | 170 | 100% | 97.70% | 97.79–100% | 0 | 0 |
| Technology | 226 | 100% | 100% | 98.33–100% | 0 | 0 |

These measurements describe the checked-in corpus, not a universal probability of correctness. The tests recompute the table and enforce zero known strict false canonicalizations, at least 99% relaxed observed precision, and no protected-hard-negative failures.

`acorn format --fuzzy <MODE>` applies the selected policy to RAD fields and contact organization data. `acorn gather --fuzzy <MODE>` applies it when provider evidence becomes a persisted candidate. Unknown evidence is preserved in formatted fields or provider provenance; similarity never establishes organizational or record identity.

The native resolver may reuse a versioned JSON cache keyed by vocabulary content, vocabulary revision, policy revision, cache schema, and Unicode-data version. Cache records contain prepared embedded vocabulary data only, never user input or reports. A missing, stale, or corrupt record is rebuilt without changing resolver semantics; global `--clear-cache` removes this ACORN-managed namespace with the other application caches. Alloc-only and WASM callers use the same embedded resolver contract without filesystem state.

Release-mode Criterion observations on the same development host were 11.83–11.91 µs to construct the keyword vocabulary and 327–331 µs for organizations. A validated warm organization cache loaded in 338–343 µs versus 355–357 µs for uncached preparation. One six-field RAD advisory pass took 20.90–21.17 ms in relaxed mode and 10.48–10.61 ms in strict mode. Maximum 256-input batches took 92.2–92.7 ms relaxed and 96.2–97.3 ms strict; the unmatched worst case took 125.7–126.3 ms.

Accepted Info and Warning advisories can produce deterministic safe-automatic fixes. Application rechecks the finding fingerprint, original located value, effective mode, complete and unambiguous outcome, selected canonical value, vocabulary revision, and policy revision against the current resolver. Scalar keyword, partner, and technology lists are stably deduplicated after replacement; structured funder records remain distinct. Ambiguous and resource-limited results have no automatic replacement, and inference cannot choose or modify one.

Every applied vocabulary formatting or fix operation writes one combined audit report in ACORN's application-data namespace. Reports contain redacted source URIs, before/after content fingerprints, locators, original and replacement values, outcomes, bounded candidates and evidence, effective modes, completeness, resolver revisions, and scalar-deduplication effects rather than a full document snapshot. URL credentials, queries, and fragments are removed. Files use owner-only permissions on Unix and retention keeps the latest 1,000 reports. Preview, unchanged, and rolled-back operations do not leave reports. Global `--clear-logs` removes only this managed log namespace.

## Pairwise comparison

Rust callers can use `acorn_core::similarity::compare` to select a method, validate its options, and receive a typed result containing the score, optional raw distance, effective parameters, and stable algorithm variant. The facade requires `acorn-core`'s `alloc` feature and performs no file or network I/O.

Use `acorn diff --method` to calculate one similarity score from two supplied values. Operands are UTF-8 files by default; `--values` treats both operands as literal text. The command works offline and does not interpret operands as URLs.

```console
acorn diff Paul Pual --method optimal-string-alignment --values
acorn diff before.txt after.txt --method jaccard --format json
acorn diff 10 12 --method absolute-difference --cutoff 4 --values
```

Text methods compare complete file contents, including trailing whitespace. Numeric methods trim surrounding whitespace before parsing finite numbers. Raw preprocessing is the default. `--preprocess lowercase` applies Rust Unicode lowercase mapping; `case-fold` applies full locale-independent Unicode case folding; `nfc` applies canonical normalization; and `nfkc` applies compatibility normalization. Each is an explicit, non-composable mode. Overlap, Jaccard, and Sørensen–Dice use Unicode-whitespace word sets unless `--ngram-width` selects unpadded character n-grams.

Prefix and suffix comparisons require `--length`. Absolute- and percentage-difference comparisons require a positive `--cutoff`. The three `--max-*` resource options override the core defaults for textual work. Invalid or irrelevant options are errors, while a score of zero is a successful comparison.

Similarity mode defaults to human-readable text and accepts `--format text` or `--format json`. JSON reports the method and stable variant, optional preprocessing and raw distance, the normalized score and its semantics, and the effective parameters. It does not repeat operand contents. Without `--method`, `acorn diff` retains its semantic file comparison behavior and accepts `diff` or `json-patch` output.
