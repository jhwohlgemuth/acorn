# Browser examples

These are illustrative teaching records, not claims that the named institutions,
people or identifiers belong to one real project. Contact and project links use
example domains. The browser checks syntax locally without resolving identifiers.

| Example | What it exercises |
| --- | --- |
| `summary.json` | Valid multi-site research, methods, outcomes, contributors, dates and datasets. |
| `publications.json` | Valid multi-institution RAD with project and output DOIs, authors, affiliations, funding, languages, rights, geography and logbook milestones. Also the Crosswalk and FAIR input. |
| `computing.json` | Valid heterogeneous CPU/GPU/TPU/FPGA resources, multiple compute environments, software provenance, real and synthetic data. |
| `validation-review.json` | Intentionally invalid RAD for Research Activity and Metadata standard. A focused set of issues covers PIDs, required text, contact details, dates, chronology, languages, rights, geography, hardware/vendor consistency, duplicate IDs, data licenses and logbook ownership. Other records contain corrected values for comparison. It parses successfully so schema errors and local FAIR suggestions and information appear together. |
| `discovery.txt` | Multiple distinct examples of all 12 supported PID kinds, varied forms, duplicate aliases, and malformed lookalikes. |
| `diff-before.json` / `diff-after.json` | Nested metadata changes, added and removed fields, modified arrays, new datasets and outputs, and changed licensing. **Use example** selects JSON Patch for semantic diff; the same pair also works as unified text diff. Key order and whitespace alone do not create JSON Patch operations. |
| `readability.txt` | Several paragraphs mixing accessible explanations with technical methods and dense prose. |

The inspection example uses a qualified Software Heritage content identifier with
origin, revision anchor, path and line range. Big-O compares composite growth
expressions. Home loads the three valid RAD files; validation tool examples are
deliberately flawed and labeled in the button tooltip and document title.

Vocabulary matching defaults to Off. Select Strict or Relaxed to enable vocabulary advisories. Local FAIR findings
are included in every successfully parsed RAD, including records with schema errors.
The real-WASM tests exercise the same requests loaded by the UI, verify multiple
identifiers per PID kind, and check representative diagnostics across nested categories.
