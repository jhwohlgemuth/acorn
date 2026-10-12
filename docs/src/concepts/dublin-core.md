# Dublin Core

ACORN models the current DCMI vocabularies as an RDF default graph. The model covers all 129 terms published in the 2020 DCMI snapshots for Metadata Terms, the legacy Element Set, the DCMI Type Vocabulary, and DCAM.

Use `dcmi` on the command line. `dublin-core` is also accepted where a standard name is parsed.

## Representations

ACORN reads and writes five Dublin Core representations:

| Representation | File suffix | Media type |
| --- | --- | --- |
| Native tagged JSON | `.json` | `application/json` |
| Native tagged YAML | `.yaml`, `.yml` | `application/yaml` |
| JSON-LD | `.jsonld` | `application/ld+json` |
| RDF/XML | `.rdf`, `.rdfxml` | `application/rdf+xml` |
| Turtle | `.ttl` | `text/turtle` |

The native JSON and YAML shape records each subject, its RDF types, and a predicate map. Values are tagged as resources or literals. Literals may have a language or datatype, but not both.

```json
{
  "descriptions": [
    {
      "subject": { "kind": "iri", "value": "https://example.org/dataset" },
      "types": ["http://purl.org/dc/dcmitype/Dataset"],
      "properties": {
        "http://purl.org/dc/terms/title": [
          {
            "kind": "literal",
            "value": { "language": "en", "value": "Example dataset" }
          }
        ]
      }
    }
  ]
}
```

Unknown absolute vocabulary IRIs, repeated values, language-tagged literals, typed literals, and blank-node relationships are retained. Serialization sorts and deduplicates the graph. Blank-node labels are canonicalized, so equivalent RDF graphs compare equal across concrete syntaxes.

ACORN rejects named graphs, RDF-star, relative IRIs, remote JSON-LD contexts, malformed RDF, and literals that combine a language with a datatype. JSON-LD input must be expanded or use an inline context. The decoder never fetches a context or vocabulary.

## Validation and checks

`acorn check` validates graph structure and the lexical forms of the DCMI syntax encoding schemes, including URI, W3CDTF, ISO language and country codes, and the DCMI Box, Period, and Point forms. At least one registered DCMI property or class is required.

DCMI domain, range, and related usage statements are guidance. ACORN reports mismatches as quality suggestions instead of rejecting the graph. FAIR checks use the metadata present in the graph, link checks follow resource IRIs when network checks are enabled, and prose and readability checks cover the descriptive literals.

```sh
acorn check metadata.ttl --standard dcmi
acorn --offline check metadata.jsonld --standard dcmi --skip link
acorn format metadata.rdf --standard dcmi
acorn diff before.ttl after.jsonld --standard dcmi
acorn export --schema --standard dcmi --format json
```

Generic `.xml` files are not treated as RDF/XML. Use `.rdf` or `.rdfxml`.

## Crosswalks

Dublin Core converts in both directions with DataCite, DCAT, InvenioRDM, and HuWise. RAD can be exported to Dublin Core. RAD export and routes to DCAT, InvenioRDM, and HuWise use DataCite as their shared intermediate model. ACORN reports known field loss as crosswalk warnings.

```sh
acorn export datacite.json \
  --from datacite \
  --to dcmi \
  --format turtle \
  --output ./export

acorn export metadata.ttl \
  --from dcmi \
  --to datacite \
  --format json \
  --output ./export
```

RAD requires a local identifier and a complete contact that Dublin Core cannot supply reliably. Provide both when importing:

```sh
acorn export metadata.ttl \
  --from dcmi \
  --to rads \
  --format json \
  --crosswalk-contact contact.json \
  --local-identifier imported-dataset \
  --output ./export
```

The contact file may be JSON or YAML and must match the RAD `ContactPoint` schema. ACORN fails the conversion when either required input is missing. It does not create placeholder identity data.

Store reusable import defaults under `options.crosswalk` in ACRE configuration:

```yaml
options:
  crosswalk:
    contact: contacts/default.yaml
    localIdentifier: imported-dataset
```

Configured contact paths are resolved relative to the configuration file. Pass `--config` to select a file explicitly; `--crosswalk-contact` and `--local-identifier` override configured values.

Crosswalks select the first description containing a title as the root resource. Additional graph descriptions and third-party predicates remain available in Dublin Core output, but a target model may not have an equivalent field. Those losses appear in the warning list, and `--strict` rejects the output.

## Rust and browser APIs

`acorn_schema::standard::dublin_core::DublinCoreDocument` implements `DocumentCodec` for every representation listed above. The public registry functions expose term definitions and declared relationships. `DublinCoreImportOptions` supplies the required RAD contact and local identifier.

The WebAssembly package validates Dublin Core in all five representations. `crosswalkMetadata` returns JSON output, while `crosswalkMetadataAs` accepts an explicit output format for Dublin Core targets. Both operate on supplied strings without filesystem or network access.
