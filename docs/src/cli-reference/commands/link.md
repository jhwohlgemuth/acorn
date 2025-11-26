# 🕸️ Link
> Add linked data context to research activity data and create `JSON-LD` documents

The `link` command augments research activity data (RAD), as provided by a human user, with linked data context and outputs `JSON-LD` documents. This process involves mapping the input data to established ontologies and vocabularies, thereby enhancing its interoperability and semantic richness.

ACORN enables ingesting RAD in a sort of "pre-compacted" format. The `link` command makes the input RAD machine-readable, ready for programmatic expansion.

> [!TIP]
> [Linked Data](https://json-ld.org/) empowers people that publish and use information on the Web. It is a way to create a network of standards-based, machine-readable data across Web sites.

## Example Usage
```sh
# Link a specific research activity index
acorn link path/to/project/index.json

# Link all research activity data in a directory
acorn link path/to/project/
```

> [!TIP]
> The `link` command is almost identical in usage to the [`format` command](./format.md), complete with support for the `--dry-run` flag to preview changes without creating files.

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