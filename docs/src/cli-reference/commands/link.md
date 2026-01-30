# 🕸️ Link
> Add linked data context to research activity data (RAD) and create `JSON-LD` documents

The `link` command augments user-input RAD with linked data context and outputs `JSON-LD` documents. This process involves mapping the input data to established ontologies and vocabularies, thereby enhancing its interoperability and depth of meaning.

ACORN helps prepare RAD for programmatic expansion. The `link` command makes input RAD machine-readable.

> [!TIP]
> [Linked Data](https://json-ld.org/) creates a connected network of standards-based, machine-readable data across Web sites.

## Example Usage
```sh
# Link a specific research activity index
acorn link path/to/project/index.json

# Link all research activity data in a directory
acorn link path/to/project/
```

> [!TIP]
> The `link` command is almost identical to the [`format` command](./format.md), complete with support for the `--dry-run` flag to preview changes without creating files.

## Example Output
```diff
    "contact": {
+     "@context": {
+       "jobTitle": "https://schema.org/jobTitle",
+       "givenName": "https://schema.org/givenName",
+       "familyName": "https://schema.org/familyName",
+       "identifier": "https://orcid.org",
+       "email": "https://schema.org/email",
+       "telephone": "https://schema.org/telephone",
+       "url": "https://schema.org/url",
+       "organization": "https://schema.org/worksFor",
+       "affiliation": "https://schema.org/affiliation"
+     },
+     "@type": "https://schema.org/person",
      "jobTitle": "Primary Investigator",
      "givenName": "Audson",
      "familyName": "Cargohlmuth",
      "email": "wohlgemuthjh@ornl.gov",
      "telephone": "865.576.7658",
      "url": "https://www.ornl.gov/staff-profile/jason-h-wohlgemuth",
      "organization": "Geospatial Science and Human Security Division",
      "affiliation": "National Security Sciences Directorate"
    }
```