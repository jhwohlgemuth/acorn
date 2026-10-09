# Similarity and distance

ACORN exposes bounded similarity primitives through `acorn_core::similarity`. They describe resemblance between supplied values; they are not probabilities, confidence estimates, or evidence that two research records identify the same activity.

Character algorithms operate on Unicode scalar values rather than UTF-8 bytes. Raw comparison is case-sensitive and performs no hidden Unicode normalization, punctuation removal, or case folding. Callers may explicitly request Rust Unicode lowercase mapping before word or character n-gram preparation.

The initial portable API includes exact prefix and suffix comparisons, Hamming distance, Levenshtein distance, restricted optimal string alignment (OSA), unrestricted Damerau-Levenshtein distance, Jaro, conventional Jaro-Winkler, word and character n-gram preparation, overlap/Jaccard/Sorensen-Dice set similarity, and cutoff-based numeric similarity. OSA and unrestricted Damerau-Levenshtein are separate variants: for `CA` and `ABC`, their respective distances are 3 and 2.

Normalized edit similarity is `1 - distance / max(lengths)`, with empty/empty defined as 1. Set methods deduplicate their inputs; both-empty sets score 1 and one-empty sets score 0. Conventional Jaro-Winkler uses at most four common prefix scalars, a scale of 0.1, and applies its boost only when Jaro similarity is above 0.7.

Every allocating or nested operation accepts `SimilarityLimits`. The defaults allow 4,096 scalars per input, 4,000,000 comparison work units, and 4,096 collection items. Exceeding a limit is an explicit error; inputs are never truncated silently. Scores use a validated finite type bounded to `[0,1]`, so NaN and infinity cannot be serialized as similarity scores.

The algorithm inventory and examples were independently specified with reference to Felix Naumann, *Similarity measures*, 11 June 2013. Page references in the project plan use physical PDF pages. The lecture is reference material rather than executable specification: notably, its local transposition recurrence describes OSA rather than unrestricted Damerau-Levenshtein. Corpus methods, phonetic encodings, temporal policies, sequence alignment, vocabulary coercion, and duplicate suggestions remain separate later work.
