# ACORN Schemas and Ontologies

The research activity schema organizes identity, metadata, technology context, narrative sections, contact information, and extension notes into one portable record. Field names below follow the serialized JSON/YAML form.

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
