# Diff and pairwise similarity

`acorn diff` has two explicit modes. Without `--method`, it compares two files through their typed metadata representation and emits a unified diff by default. With `--method`, it calculates a bounded pairwise similarity score.

## Semantic file comparison

```console
acorn diff before.json after.json
acorn diff before.yaml after.yaml --format json-patch
acorn diff before.json after.json --standard rads
acorn diff before.ttl after.jsonld --standard dcmi
```

Semantic mode accepts `diff` (`unified` is an alias) and `json-patch`. A standard override is available only in this mode.

For Dublin Core, semantic mode decodes native graph JSON, JSON-LD, RDF/XML, and Turtle into the same normalized graph before comparison. Equivalent RDF in different syntaxes produces an empty diff. Unified output uses the first file's syntax; JSON Patch operates on ACORN's canonical graph JSON representation.

## Similarity comparison

Files remain the default operands:

```console
acorn diff first.txt second.txt --method levenshtein
acorn diff first.txt second.txt --method jaccard --ngram-width 3 --format json
```

Use `--values` when the operands themselves are the values:

```console
acorn diff Paul Pual --method optimal-string-alignment --values
acorn diff 10 12 --method absolute-difference --cutoff 4 --values
```

Available methods are:

- `exact`, `prefix`, and `suffix`
- `hamming`, `levenshtein`, `weighted-levenshtein`, `optimal-string-alignment` (`osa`), and `damerau-levenshtein`
- `jaro` and `jaro-winkler`
- `overlap`, `jaccard`, and `sorensen-dice` (`dice`)
- `monge-elkan`, `extended-jaccard`, `tfidf-cosine`, and `soft-tfidf`
- `soundex`, `cologne-phonetic`, and `metaphone`
- `cosine` for comma- or whitespace-delimited finite vectors
- `absolute-difference` and `percentage-difference`

Prefix and suffix require `--length`. Numeric methods require a finite positive `--cutoff`. Extended Jaccard and Soft TF-IDF require an inclusive `--threshold`; tied or reused fuzzy-token matches fail instead of selecting by input order. Set and hybrid methods split on Unicode whitespace by default; `--ngram-width N` switches to unpadded character n-grams. Weighted Levenshtein uses insertion 1, deletion 1, and substitution 2 in the CLI. Preprocessing is explicit through `--preprocess`: choose `raw` (the default), `lowercase`, full locale-independent `case-fold`, canonical `nfc`, or compatibility `nfkc`. The modes are not combined. NFC is the recommended opt-in for human-language text that should ignore canonically equivalent encodings; NFKC is appropriate only when compatibility distinctions may also be erased.

The TF-IDF CLI methods use the two compared token sequences as an explicit pair-local corpus and report its deterministic identity. For reusable research corpora, use `acorn_core::similarity::TfidfCorpus` directly so document boundaries and corpus identity remain fixed across comparisons. Monge-Elkan is directional: swapping operands can change its score. Phonetic methods compare their language-specific encodings for equality; they rank spelling resemblance and do not establish contributor or organization identity.

Similarity mode accepts `text` and `json`, defaulting to text. A low score is a successful result. Invalid input, incompatible parameters, unreadable files, and exceeded resource limits fail the command.

Use `--max-input-scalars`, `--max-work-units`, and `--max-collection-items` to replace the documented [similarity defaults](../../concepts/similarity.md). Inputs are rejected rather than truncated when a limit is exceeded.
