# ACORN Schemas and Ontologies

The research activity schema organizes identity, metadata, technology context, narrative sections, contact information, and extension notes into one portable record. Field names below follow the serialized JSON/YAML form.

## Context awareness

ACORN's context awareness comes from explicit data. A research activity record places its narrative alongside project identity, persistent identifiers, people, organizations, technology, media, and relationships. Linked-data mappings and provenance show how that information connects and where it came from.

Commands use this context for different purposes. `acorn check` evaluates values against the appropriate schema and rules, `acorn format` normalizes controlled values, `acorn link` maps fields to defined ontologies, and `acorn export` carries the same project information into other formats. Optional model and agent workflows can receive the record, relevant evidence, and validation results as task context.

## Draft authoring

RAD can begin as a draft and become stricter as it matures. Input may omit `meta.archive` and `meta.draft`; ACORN defaults them to `false` and `true`, respectively. Narrative `sections` may be omitted while `draft` is true, but a non-draft record must provide them.

List-shaped metadata is also permissive at the authoring boundary. `keywords`, `technology`, and every optional metadata vector accept either one item or an array. ACORN normalizes a single item to a one-item array. For optional vectors, an omitted or `null` value means absent, while `[]` remains explicitly empty. Canonical JSON, YAML, and Markdown output uses arrays; empty `keywords` and `technology` fields are omitted.

Context-aware operation does not mean that ACORN knows facts absent from its inputs or that it has consciousness. Missing or unsupported information remains missing until a person or a defined provider supplies it. This boundary lets context reveal documented connections and gaps without turning an inference into project data.

> [!NOTE]
> ACORN provides field-level support for selected representations from several ISO standards: ISO 8601 dates and years, ISO 3166-1 alpha-2 country codes, ISO 639-1 language codes (with schema-specific support for other ISO 639 forms), ISO 2108 ISBNs, and ISO 7064 checksums used by ISNI, ORCID, and ROR identifiers. ACORN also parses and validates RAiD identifiers, which are governed by ISO 23527. This support validates the forms used by ACORN schemas; it does not claim complete implementation of, or certification against, each standard. See [Persistent identifier lifecycle](persistent-identifiers.md) for the exact local identifier checks.

<div style="height: 2rem" aria-hidden="true"></div>

```pikchr
scale = 0.72
Root: box wid 0.72in ht 0.72in rad 0.08in fill none thickness 0
Metadata: box "Metadata" bold wid 1.0in ht 0.52in rad 0.08in fill 0xE8F1FB thickness 0 at Root+(0.84in,-1.0in)
Lifecycle: box "Lifecycle" bold "classification · archive · draft · status" italic wid 2.7in ht 0.58in rad 0.08in fill 0xE8F1FB thickness 0 at Metadata+(2.45in,0)
Identifiers: box "Identifiers" bold "identifier · doi · books · patents" italic "publications · raid · ror" italic wid 2.7in ht 0.68in rad 0.08in fill 0xE8F1FB thickness 0 at Lifecycle+(0,-0.7in)
Content: box "Content" bold "additionalType · media · websites" italic "keywords" italic wid 2.7in ht 0.68in rad 0.08in fill 0xE8F1FB thickness 0 at Lifecycle+(0,-1.4in)
Technology: box "Technology" bold wid 2.7in ht 0.62in rad 0.08in fill 0xE8F1FB thickness 0 at Lifecycle+(0,-2.1in)
Relationships: box "Relationships" bold "sponsors · partners · related · resources" italic wid 2.7in ht 0.58in rad 0.08in fill 0xE8F1FB thickness 0 at Lifecycle+(0,-2.8in)
line from Root.s down until even with Metadata then to Metadata.w thick color 0x00662C
line from Metadata.e to Lifecycle.w thick color 0x00662C
line from Metadata.e right 0.3in then down until even with Identifiers then to Identifiers.w thick color 0x00662C
line from Metadata.e right 0.3in then down until even with Content then to Content.w thick color 0x00662C
line from Metadata.e right 0.3in then down until even with Technology then to Technology.w thick color 0x00662C
line from Metadata.e right 0.3in then down until even with Relationships then to Relationships.w thick color 0x00662C
Aspect: box "ASPECT" bold wid 1.0in ht 0.52in rad 0.08in fill 0xE7F3E9 thickness 0 at Root+(0.84in,-4.5in)
Framework: box "Framework" bold "data" italic "portability · motivity · autonomy · maturity" italic wid 2.7in ht 0.68in rad 0.08in fill 0xE7F3E9 thickness 0 at Aspect+(2.45in,0)
Data: box "Data Descriptions" bold "real · synthetic · model" italic "availability · license · modality · quality" italic wid 2.7in ht 0.68in rad 0.08in fill 0xE7F3E9 thickness 0 at Framework+(0,-0.7in)
line from Root.s down until even with Aspect then to Aspect.w thick color 0x00662C
line from Aspect.e to Framework.w thick color 0x00662C
line from Aspect.e right 0.3in then down until even with Data then to Data.w thick color 0x00662C
Sections: box "📑 Sections" bold wid 1.0in ht 0.52in rad 0.08in fill 0xFCE8E6 thickness 0 at Root+(0.84in,-6.0in)
TitleSubtitle: box "title · subtitle" italic wid 2.7in ht 0.58in rad 0.08in fill 0xFCE8E6 thickness 0 at Sections+(2.45in,0)
Narrative: box "Narrative" bold "mission · challenge · approach · impact" italic "achievement · capabilities" italic wid 2.7in ht 0.68in rad 0.08in fill 0xFCE8E6 thickness 0 at TitleSubtitle+(0,-0.7in)
Research: box "Research" bold "research.focus · research.areas" italic wid 2.7in ht 0.58in rad 0.08in fill 0xFCE8E6 thickness 0 at TitleSubtitle+(0,-1.4in)
line from Root.s down until even with Sections then to Sections.w thick color 0x00662C
line from Sections.e to TitleSubtitle.w thick color 0x00662C
line from Sections.e right 0.3in then down until even with Narrative then to Narrative.w thick color 0x00662C
line from Sections.e right 0.3in then down until even with Research then to Research.w thick color 0x00662C
Contact: box "📇 Contact" bold wid 1.0in ht 0.52in rad 0.08in fill 0xFFF4D6 thickness 0 at Root+(0.84in,-8.2in)
Identity: box "Identity" bold "jobTitle" italic "givenName · familyName · identifier" italic wid 2.7in ht 0.68in rad 0.08in fill 0xFFF4D6 thickness 0 at Contact+(2.45in,0)
Reach: box "Reachability" bold "email · telephone · url" italic "organization · affiliation" italic wid 2.7in ht 0.68in rad 0.08in fill 0xFFF4D6 thickness 0 at Identity+(0,-0.7in)
line from Root.s down until even with Contact then to Contact.w thick color 0x00662C
line from Contact.e to Identity.w thick color 0x00662C
line from Contact.e right 0.3in then down until even with Reach then to Reach.w thick color 0x00662C
Notes: box "📝 Notes" bold wid 1.0in ht 0.52in rad 0.08in fill 0xF2F2F2 thickness 0 at Root+(0.84in,-9.7in)
NotesFields: box "Free Text or Structured Notes" bold "managers · programs · presentation" italic wid 2.7in ht 0.58in rad 0.08in fill 0xF2F2F2 thickness 0 at Notes+(2.45in,0)
line from Root.s down until even with Notes then to Notes.w thick color 0x00662C
line from Notes.e to NotesFields.w thick color 0x00662C
```

<div id="schema-tree" data-pikchr-fit data-logo-base="../_assets/schema-logos" hidden></div>

<div style="height: 2rem" aria-hidden="true"></div>
