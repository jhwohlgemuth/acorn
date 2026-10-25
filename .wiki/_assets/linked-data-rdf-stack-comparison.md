# Linked-data RDF stack comparison: Sophia and OxiXML

Date: 2026-10-12

## Recommendation

ACORN should keep Sophia as the planned linked-data foundation and should not migrate to OxiXML at this stage.

The immediate cleanup is still to avoid exposing `sophia::api::ns::Namespace` merely for namespace-string concatenation. ACORN can preserve its vocabulary constants and helper functions behind ACORN-owned APIs, then use Sophia when the first actual RDF graph, parser, serializer, or linked-data export feature is implemented.

OxiXML should be reconsidered only if ACORN needs a broader XML platform, direct Oxigraph-compatible concrete RDF types, or a unified XML/RDF stack covering functionality such as DOM, XPath, XSD, XSLT, XML canonicalization, or XML security.

## Current ACORN evidence

ACORN declares `sophia = { version = "0.9.0", default-features = false }` in the workspace and uses it only through `sophia::api::ns::Namespace` in:

- `crates/acorn-schema/src/namespaces.rs`
- `crates/acorn-schema/src/research_activity/logbook.rs`

There is no current use of Sophia's RDF terms, graphs, datasets, RDF/XML, Turtle, JSON-LD, SPARQL, reasoning, or canonicalization APIs. The current dependency is therefore preparatory rather than an implemented RDF integration.

## Architectural comparison

### Sophia

Sophia is a generic, trait-oriented RDF and Linked Data toolkit. `sophia_api` defines traits for terms, triples, quads, graphs, datasets, parsers, serializers, and SPARQL-related APIs. Concrete implementations and format crates are layered above it.

This makes Sophia suitable for library boundaries that should not dictate one graph implementation or storage strategy. Algorithms can work over Sophia traits, and applications can select appropriate term or graph implementations.

Sophia's model is deliberately generalized: it supports variables, relative IRIs, and broader term placement than strict RDF. Sophia 0.10 also documents RDF 1.2 support, including directional language strings and RDF-star-related API changes.

### OxiXML

OxiXML is a concrete, Oxigraph-shaped RDF and XML platform. `oxixml-model` provides owned and borrowed RDF terms (`NamedNode`, `Literal`, `Triple`, `Quad`, and their `*Ref` forms), plus concrete in-memory `Graph` and `Dataset` containers. Its `rdf-12` feature adds RDF 1.2/RDF-star capabilities, and `rdfc-10` adds RDF Dataset Canonicalization 1.0.

OxiXML's RDF components are designed as replacements for the Oxigraph crates (`oxrdf`, `oxrdfxml`, `oxttl`, and related crates). The API is straightforward for application code that wants one concrete RDF representation, but it is less implementation-agnostic than Sophia's trait layer.

## RDF/XML comparison

Both ecosystems provide RDF/XML parsing and serialization:

- Sophia: `sophia_xml`
- OxiXML: `oxixml-rdfxml`

Sophia's RDF/XML crate is based on the established `oxrdfxml` implementation and integrates with Sophia's generic term/source/serializer traits.

OxiXML's RDF/XML crate integrates directly with `oxixml-model`, exposes streaming parser and serializer APIs, and documents support for RDF/XML features including `rdf:about`, `rdf:ID`, `rdf:nodeID`, `rdf:resource`, `rdf:parseType`, containers, reification, `xml:lang`, `xml:base`, datatypes, and internal-DTD entities.

The deciding question is therefore not whether one supports RDF/XML. It is whether RDF/XML should produce Sophia-compatible abstractions or OxiXML-compatible concrete terms.

## Ecosystem scope

Sophia is focused on RDF and linked data. Its current ecosystem includes:

- generic RDF API and term implementations;
- in-memory graphs and datasets;
- Turtle-family, RDF/XML, and JSON-LD formats;
- canonicalization and graph isomorphism;
- reasoning;
- SPARQL and SPARQL protocol client support;
- resource-centric APIs.

OxiXML includes comparable RDF format and model crates, but also aims to cover a broad XML stack:

- XML pull parsing and writing;
- DOM and SAX;
- XPath and XQuery;
- XSD, RELAX NG, and Schematron;
- XSLT;
- XML canonicalization, signatures, encryption, and related security layers;
- an XML/RDF command-line tool.

This breadth is an advantage only if ACORN needs the breadth. For a schema and validation tool adding linked-data support incrementally, it is also additional ecosystem and API surface to track.

## Maturity and change risk

Sophia 0.10 is an established modular RDF toolkit with a long project history and a current RDF 1.2 release line. OxiXML's published crates are currently in the 0.1 series. OxiXML reports strong conformance results, but its 0.1.x release notes also document breaking corrections to borrowed-versus-owned APIs and serializer signatures.

This does not make OxiXML unsuitable. It means adopting OxiXML currently carries more API-evolution risk. ACORN should prefer it only when its XML-platform benefits or Oxigraph compatibility outweigh that risk.

## Dependency and API-boundary guidance

ACORN should not make `Namespace` a central schema type. Namespace URI constants are stable domain data and can remain ACORN-owned. A small helper such as `format!("{namespace}{suffix}")` is sufficient until RDF terms are actually needed.

When RDF work begins, put the integration behind an ACORN-owned adapter or module boundary. This allows ACORN to:

1. keep schema types independent of a third-party RDF model;
2. convert research-activity data into RDF deliberately;
3. test vocabulary and mapping semantics independently;
4. change between Sophia and OxiXML later without spreading third-party types through `acorn-schema`.

If Sophia remains the choice, ACORN should revisit whether it needs the umbrella `sophia` dependency or can depend on focused Sophia crates/features. The workspace should also update from the currently declared 0.9 line to the selected 0.10 line only as part of a deliberate compatibility change.

## Decision matrix

| Requirement | Better fit |
|---|---|
| Generic public RDF APIs | Sophia |
| Interchangeable graph and term implementations | Sophia |
| Focused RDF/Linked Data toolkit | Sophia |
| Reasoning and SPARQL-oriented ecosystem | Sophia |
| Concrete Oxigraph-shaped terms and graphs | OxiXML |
| Direct compatibility with the OxiGraph/Oxigraph family | OxiXML |
| RDF plus a broad XML implementation platform | OxiXML |
| DOM/XPath/XSD/XSLT/XML security in the same workspace | OxiXML |
| Lowest immediate change risk for ACORN | Sophia |

## Sources

- [Sophia API documentation](https://docs.rs/sophia_api/latest/sophia_api/)
- [Sophia toolkit documentation](https://docs.rs/sophia/latest/sophia/)
- [Sophia XML documentation](https://docs.rs/sophia_xml/latest/sophia_xml/)
- [Sophia repository and crate overview](https://github.com/pchampin/sophia_rs)
- [Sophia changelog](https://github.com/pchampin/sophia_rs/blob/main/CHANGELOG.txt)
- [OxiXML model documentation](https://docs.rs/crate/oxixml-model/0.1.2)
- [OxiXML RDF/XML documentation](https://docs.rs/crate/oxixml-rdfxml/0.1.0)
- [OxiXML unified RDF I/O documentation](https://docs.rs/oxixml-io/latest/oxixml_io/)
- [OxiXML release notes](https://github.com/cool-japan/oxixml/releases)
