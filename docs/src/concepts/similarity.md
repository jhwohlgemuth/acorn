# Similarity and distance

ACORN exposes bounded similarity primitives through `acorn_core::similarity`. They describe resemblance between supplied values; they are not probabilities, confidence estimates, or evidence that two research records identify the same activity.

Character algorithms operate on Unicode scalar values rather than UTF-8 bytes. Raw comparison is case-sensitive and performs no hidden Unicode normalization, punctuation removal, or case folding. Callers may explicitly request Rust Unicode lowercase mapping, full locale-independent Unicode case folding, NFC canonical normalization, or NFKC compatibility normalization before comparison and token preparation.

Raw remains the default because this is a general-purpose API: normalization can change scalar-level distances, and compatibility normalization can collapse distinctions that matter in identifiers, technical notation, and source text. For human-language matching where canonically equivalent spellings should compare alike, opt in to NFC. Prefer NFC over NFKC unless the consumer intentionally treats compatibility characters such as circled digits and ordinary digits as equivalent. Case folding is separate from normalization and should be selected only when case distinctions are irrelevant.

The initial portable API includes exact prefix and suffix comparisons, Hamming distance, Levenshtein distance, restricted optimal string alignment (OSA), unrestricted Damerau-Levenshtein distance, Jaro, conventional Jaro-Winkler, word and character n-gram preparation, overlap/Jaccard/Sorensen-Dice set similarity, and cutoff-based numeric similarity. OSA and unrestricted Damerau-Levenshtein are separate variants: for `CA` and `ABC`, their respective distances are 3 and 2.

Normalized edit similarity is `1 - distance / max(lengths)`, with empty/empty defined as 1. Set methods deduplicate their inputs; both-empty sets score 1 and one-empty sets score 0. Conventional Jaro-Winkler uses at most four common prefix scalars, a scale of 0.1, and applies its boost only when Jaro similarity is above 0.7.

Every allocating or nested operation accepts `SimilarityLimits`. The defaults allow 4,096 scalars per input, 4,000,000 comparison work units, and 4,096 collection items. Exceeding a limit is an explicit error; inputs are never truncated silently. Scores use a validated finite type bounded to `[0,1]`, so NaN and infinity cannot be serialized as similarity scores.

The algorithm inventory and examples were independently specified with reference to Felix Naumann, *Similarity measures*, 11 June 2013. Page references in the project plan use physical PDF pages. The lecture is reference material rather than executable specification: notably, its local transposition recurrence describes OSA rather than unrestricted Damerau-Levenshtein. Corpus methods, phonetic encodings, temporal policies, sequence alignment, and duplicate suggestions remain separate later work.

## Controlled-vocabulary resolution

`acorn_schema::vocabulary::VocabularyResolver` applies these primitives to ACORN's embedded keyword, technology, partner, sponsor, and organization vocabularies. It prepares each vocabulary once per batch, treats input as literal text, and checks canonical values, declared aliases, and typed identifiers before bounded fuzzy ranking. Distinct canonical entries that share an alias or cannot clear the runner-up margin are reported as ambiguous rather than selected by row order.

Every input produces a `VocabularyResolution` with its original value, optional source locator, outcome, accepted canonical value, matched term, method, score, runner-up margin, and bounded alternatives. Reports also identify the vocabulary and policy revisions. The lossless projection substitutes accepted values and retains unmatched, ambiguous, invalid, or resource-limited input; the accepted projection is stable and deduplicated.

Three profiles are available:

- `strict` is the default high-precision policy.
- `relaxed` lowers the vocabulary-specific fuzzy score and separation gates but keeps ambiguity and resource safeguards.
- `off` disables fuzzy scoring while retaining exact canonical, alias, identifier, and normalized matches.

`acorn format --fuzzy <MODE>` applies the selected policy to RAD fields and contact organization data. `acorn gather --fuzzy <MODE>` applies it when provider evidence becomes a persisted candidate. Unknown evidence is preserved in formatted fields or provider provenance; similarity never establishes organizational or record identity.

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
