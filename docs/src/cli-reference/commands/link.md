# 🕸️ Link
<blockquote class="blockquote-tag acorn-nutshell">
<p class="blockquote-tag-title acorn-nutshell-title">In a nutshell</p>
<p>Add linked-data context and create JSON-LD with <code>acorn link &lt;PATH&gt;</code>.</p>
</blockquote>

`acorn link` maps RAD fields to defined ontologies and vocabularies, then writes the result as JSON-LD for programmatic use.

> [!TIP]
> [Linked Data](https://json-ld.org/) creates a connected network of standards-based, machine-readable data across Web sites.

## Example usage
```sh
# Link a specific research activity index
acorn link path/to/project/index.json

# Create JSON-LD from Markdown RAD
acorn link path/to/project/index.md

# Link all research activity data in a directory
acorn link path/to/project/
```

> [!TIP]
> The `link` command is almost identical to the [`format` command](./format.md), complete with support for the `--dry-run` flag to preview changes without creating files.

## Example output
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

<p class="acorn-next"><strong>Next stop:</strong>&nbsp;Turn linked RAD into an artifact with <a href="export.md">Export</a>.</p>
