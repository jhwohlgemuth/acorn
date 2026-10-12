//! Registered terms in the pinned DCMI vocabularies.
use super::{DublinCoreNamespace, DublinCoreTermDefinition, DublinCoreTermKind, DublinCoreTermStatus};

macro_rules! term_definition {
    ($namespace:ident, $status:ident, $kind:ident, $local:literal, $label:literal, $issued:literal, $definition:literal) => {
        DublinCoreTermDefinition {
            definition: $definition,
            issued: $issued,
            kind: DublinCoreTermKind::$kind,
            label: $label,
            local_name: $local,
            namespace: DublinCoreNamespace::$namespace,
            status: DublinCoreTermStatus::$status,
        }
    };
}

const TERMS: &[DublinCoreTermDefinition] = &[
    term_definition!(
        AbstractModel,
        Published,
        Property,
        "domainIncludes",
        "Domain Includes",
        "2020-01-20",
        "A suggested class for subjects of this property."
    ),
    term_definition!(
        AbstractModel,
        Published,
        Property,
        "memberOf",
        "Member Of",
        "2008-01-14",
        "A relationship between a resource and a vocabulary encoding scheme that indicates membership in the set."
    ),
    term_definition!(
        AbstractModel,
        Published,
        Property,
        "rangeIncludes",
        "Range Includes",
        "2020-01-20",
        "A suggested class for values of this property."
    ),
    term_definition!(
        AbstractModel,
        Published,
        Class,
        "VocabularyEncodingScheme",
        "Vocabulary Encoding Scheme",
        "2008-01-14",
        "An enumerated set of resources."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "contributor",
        "Contributor",
        "1999-07-02",
        "An entity responsible for making contributions to the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "coverage",
        "Coverage",
        "1999-07-02",
        "The spatial or temporal topic, applicability, or jurisdiction of the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "creator",
        "Creator",
        "1999-07-02",
        "An entity primarily responsible for making the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "date",
        "Date",
        "1999-07-02",
        "A point or period of time associated with an event in the lifecycle of the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "description",
        "Description",
        "1999-07-02",
        "An account of the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "format",
        "Format",
        "1999-07-02",
        "The file format, physical medium, or dimensions of the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "identifier",
        "Identifier",
        "1999-07-02",
        "An unambiguous reference to the resource within a given context."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "language",
        "Language",
        "1999-07-02",
        "A language of the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "publisher",
        "Publisher",
        "1999-07-02",
        "An entity responsible for making the resource available."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "relation",
        "Relation",
        "1999-07-02",
        "A related resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "rights",
        "Rights",
        "1999-07-02",
        "Information about rights held in and over the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "source",
        "Source",
        "1999-07-02",
        "A related resource from which the described resource is derived."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "subject",
        "Subject",
        "1999-07-02",
        "The topic of the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "title",
        "Title",
        "1999-07-02",
        "A name given to the resource."
    ),
    term_definition!(
        MetadataElements,
        Legacy,
        Property,
        "type",
        "Type",
        "1999-07-02",
        "The nature or genre of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "Agent",
        "Agent",
        "2008-01-14",
        "A resource that acts or has the power to act."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "AgentClass",
        "Agent Class",
        "2008-01-14",
        "A group of agents."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "BibliographicResource",
        "Bibliographic Resource",
        "2008-01-14",
        "A book, article, or other documentary resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "Box",
        "DCMI Box",
        "2000-07-11",
        "Regions in space defined by geographic coordinates according to the DCMI Box Encoding Scheme."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "DCMIType",
        "DCMI Type Vocabulary",
        "2000-07-11",
        "The classes specified by the DCMI Type Vocabulary."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "DDC",
        "DDC",
        "2000-07-11",
        "The conceptual resources specified by the Dewey Decimal Classification."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "FileFormat",
        "File Format",
        "2008-01-14",
        "A digital resource format."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "Frequency",
        "Frequency",
        "2008-01-14",
        "A rate at which something recurs."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "IMT",
        "IMT",
        "2000-07-11",
        "The media types specified by the Internet Assigned Numbers Authority."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "ISO3166",
        "ISO 3166",
        "2000-07-11",
        "The ISO 3166-1 codes for representing names of countries."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "ISO639-2",
        "ISO 639-2",
        "2000-07-11",
        "The ISO 639-2 three-letter codes for representing names of languages."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "ISO639-3",
        "ISO 639-3",
        "2008-01-14",
        "The ISO 639-3 three-letter codes for representing names of languages."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "Jurisdiction",
        "Jurisdiction",
        "2008-01-14",
        "The extent or range of judicial, law-enforcement, or other authority."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "LCC",
        "LCC",
        "2000-07-11",
        "The conceptual resources specified by the Library of Congress Classification."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "LCSH",
        "LCSH",
        "2000-07-11",
        "The labeled concepts specified by the Library of Congress Subject Headings."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "LicenseDocument",
        "License Document",
        "2008-01-14",
        "A legal document giving official permission to do something with a resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "LinguisticSystem",
        "Linguistic System",
        "2008-01-14",
        "A system of signs, symbols, sounds, gestures, or rules used in communication."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "Location",
        "Location",
        "2008-01-14",
        "A spatial region or named place."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "LocationPeriodOrJurisdiction",
        "Location, Period, or Jurisdiction",
        "2008-01-14",
        "A location, period of time, or jurisdiction."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "MESH",
        "MeSH",
        "2000-07-11",
        "The labeled concepts specified by the Medical Subject Headings."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "MediaType",
        "Media Type",
        "2008-01-14",
        "A file format or physical medium."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "MediaTypeOrExtent",
        "Media Type or Extent",
        "2008-01-14",
        "A media type or extent."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "MethodOfAccrual",
        "Method of Accrual",
        "2008-01-14",
        "A method by which resources are added to a collection."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "MethodOfInstruction",
        "Method of Instruction",
        "2008-01-14",
        "A process used to engender knowledge, attitudes, and skills."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "NLM",
        "NLM",
        "2005-06-13",
        "The conceptual resources specified by the National Library of Medicine Classification."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "Period",
        "DCMI Period",
        "2000-07-11",
        "Time intervals defined by their limits according to the DCMI Period Encoding Scheme."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "PeriodOfTime",
        "Period of Time",
        "2008-01-14",
        "An interval of time named or defined by its start and end dates."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "PhysicalMedium",
        "Physical Medium",
        "2008-01-14",
        "A physical material or carrier."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "PhysicalResource",
        "Physical Resource",
        "2008-01-14",
        "A material thing."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "Point",
        "DCMI Point",
        "2000-07-11",
        "Points in space defined by geographic coordinates according to the DCMI Point Encoding Scheme."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "Policy",
        "Policy",
        "2008-01-14",
        "A plan or course of action by an authority intended to influence decisions and actions."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "ProvenanceStatement",
        "Provenance Statement",
        "2008-01-14",
        "Significant changes in ownership and custody of a resource since its creation."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "RFC1766",
        "RFC 1766",
        "2000-07-11",
        "Language-identification tags constructed according to RFC 1766."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "RFC3066",
        "RFC 3066",
        "2002-07-13",
        "Language-identification tags constructed according to RFC 3066."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "RFC4646",
        "RFC 4646",
        "2008-01-14",
        "Language-identification tags constructed according to RFC 4646."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "RFC5646",
        "RFC 5646",
        "2010-10-11",
        "Language-identification tags constructed according to RFC 5646."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "RightsStatement",
        "Rights Statement",
        "2008-01-14",
        "A statement about intellectual-property, license, or access rights for a resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "SizeOrDuration",
        "Size or Duration",
        "2008-01-14",
        "A dimension or extent, or time taken to play or execute."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Class,
        "Standard",
        "Standard",
        "2008-01-14",
        "A reference point against which other things can be evaluated or compared."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "TGN",
        "TGN",
        "2000-07-11",
        "The places specified by the Getty Thesaurus of Geographic Names."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        VocabularyEncodingScheme,
        "UDC",
        "UDC",
        "2000-07-11",
        "The conceptual resources specified by the Universal Decimal Classification."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "URI",
        "URI",
        "2000-07-11",
        "Identifiers constructed according to the generic syntax for Uniform Resource Identifiers."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        SyntaxEncodingScheme,
        "W3CDTF",
        "W3C-DTF",
        "2000-07-11",
        "Dates and times constructed according to the W3C Date and Time Formats Specification."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "abstract",
        "Abstract",
        "2000-07-11",
        "A summary of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "accessRights",
        "Access Rights",
        "2003-02-15",
        "Information about who can access the resource or an indication of its security status."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "accrualMethod",
        "Accrual Method",
        "2005-06-13",
        "The method by which items are added to a collection."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "accrualPeriodicity",
        "Accrual Periodicity",
        "2005-06-13",
        "The frequency with which items are added to a collection."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "accrualPolicy",
        "Accrual Policy",
        "2005-06-13",
        "The policy governing the addition of items to a collection."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "alternative",
        "Alternative Title",
        "2000-07-11",
        "An alternative name for the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "audience",
        "Audience",
        "2001-05-21",
        "A class of agents for whom the resource is intended or useful."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "available",
        "Date Available",
        "2000-07-11",
        "Date that the resource became or will become available."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "bibliographicCitation",
        "Bibliographic Citation",
        "2003-02-15",
        "A bibliographic reference for the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "conformsTo",
        "Conforms To",
        "2001-05-21",
        "An established standard to which the described resource conforms."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "contributor",
        "Contributor",
        "2008-01-14",
        "An entity responsible for making contributions to the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "coverage",
        "Coverage",
        "2008-01-14",
        "The spatial or temporal topic, applicability, or jurisdiction of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "created",
        "Date Created",
        "2000-07-11",
        "Date of creation of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "creator",
        "Creator",
        "2008-01-14",
        "An entity responsible for making the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "date",
        "Date",
        "2008-01-14",
        "A point or period of time associated with an event in the lifecycle of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "dateAccepted",
        "Date Accepted",
        "2002-07-13",
        "Date of acceptance of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "dateCopyrighted",
        "Date Copyrighted",
        "2002-07-13",
        "Date of copyright of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "dateSubmitted",
        "Date Submitted",
        "2002-07-13",
        "Date of submission of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "description",
        "Description",
        "2008-01-14",
        "An account of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "educationLevel",
        "Audience Education Level",
        "2002-07-13",
        "A class of agents defined by progression through an educational or training context."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "extent",
        "Extent",
        "2000-07-11",
        "The size or duration of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "format",
        "Format",
        "2008-01-14",
        "The file format, physical medium, or dimensions of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "hasFormat",
        "Has Format",
        "2000-07-11",
        "A substantially similar related resource in another format."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "hasPart",
        "Has Part",
        "2000-07-11",
        "A related resource included physically or logically in the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "hasVersion",
        "Has Version",
        "2000-07-11",
        "A related resource that is a version, edition, or adaptation of the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "identifier",
        "Identifier",
        "2008-01-14",
        "An unambiguous reference to the resource within a given context."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "instructionalMethod",
        "Instructional Method",
        "2005-06-13",
        "A process that the resource is designed to support in engendering knowledge, attitudes, and skills."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "isFormatOf",
        "Is Format Of",
        "2000-07-11",
        "A pre-existing substantially similar related resource in another format."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "isPartOf",
        "Is Part Of",
        "2000-07-11",
        "A related resource in which the described resource is physically or logically included."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "isReferencedBy",
        "Is Referenced By",
        "2000-07-11",
        "A related resource that references, cites, or otherwise points to the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "isReplacedBy",
        "Is Replaced By",
        "2000-07-11",
        "A related resource that supplants, displaces, or supersedes the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "isRequiredBy",
        "Is Required By",
        "2000-07-11",
        "A related resource that requires the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "isVersionOf",
        "Is Version Of",
        "2000-07-11",
        "A related resource of which the described resource is a version, edition, or adaptation."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "issued",
        "Date Issued",
        "2000-07-11",
        "Date of formal issuance of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "language",
        "Language",
        "2008-01-14",
        "A language of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "license",
        "License",
        "2004-06-14",
        "A legal document giving official permission to do something with the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "mediator",
        "Mediator",
        "2001-05-21",
        "An entity that mediates access to the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "medium",
        "Medium",
        "2000-07-11",
        "The material or physical carrier of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "modified",
        "Date Modified",
        "2000-07-11",
        "Date on which the resource was changed."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "provenance",
        "Provenance",
        "2004-09-20",
        "A statement of significant changes in ownership and custody of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "publisher",
        "Publisher",
        "2008-01-14",
        "An entity responsible for making the resource available."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "references",
        "References",
        "2000-07-11",
        "A related resource referenced, cited, or otherwise pointed to by the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "relation",
        "Relation",
        "2008-01-14",
        "A related resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "replaces",
        "Replaces",
        "2000-07-11",
        "A related resource supplanted, displaced, or superseded by the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "requires",
        "Requires",
        "2000-07-11",
        "A related resource required by the described resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "rights",
        "Rights",
        "2008-01-14",
        "Information about rights held in and over the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "rightsHolder",
        "Rights Holder",
        "2004-06-14",
        "A person or organization owning or managing rights over the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "source",
        "Source",
        "2008-01-14",
        "A related resource from which the described resource is derived."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "spatial",
        "Spatial Coverage",
        "2000-07-11",
        "Spatial characteristics of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "subject",
        "Subject",
        "2008-01-14",
        "A topic of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "tableOfContents",
        "Table Of Contents",
        "2000-07-11",
        "A list of subunits of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "temporal",
        "Temporal Coverage",
        "2000-07-11",
        "Temporal characteristics of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "title",
        "Title",
        "2008-01-14",
        "A name given to the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "type",
        "Type",
        "2008-01-14",
        "The nature or genre of the resource."
    ),
    term_definition!(
        MetadataTerms,
        Published,
        Property,
        "valid",
        "Date Valid",
        "2000-07-11",
        "Date, often a range, of validity of a resource."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Collection",
        "Collection",
        "2000-07-11",
        "An aggregation of resources."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Dataset",
        "Dataset",
        "2000-07-11",
        "Data encoded in a defined structure."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Event",
        "Event",
        "2000-07-11",
        "A non-persistent, time-based occurrence."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Image",
        "Image",
        "2000-07-11",
        "A visual representation other than text."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "InteractiveResource",
        "Interactive Resource",
        "2000-07-11",
        "A resource requiring interaction from the user to be understood, executed, or experienced."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "MovingImage",
        "Moving Image",
        "2003-11-18",
        "A series of visual representations imparting an impression of motion when shown in succession."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "PhysicalObject",
        "Physical Object",
        "2002-07-13",
        "An inanimate, three-dimensional object or substance."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Service",
        "Service",
        "2000-07-11",
        "A system that provides one or more functions."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Software",
        "Software",
        "2000-07-11",
        "A computer program in source or compiled form."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Sound",
        "Sound",
        "2000-07-11",
        "A resource primarily intended to be heard."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "StillImage",
        "Still Image",
        "2003-11-18",
        "A static visual representation."
    ),
    term_definition!(
        TypeVocabulary,
        Published,
        Class,
        "Text",
        "Text",
        "2000-07-11",
        "A resource consisting primarily of words for reading."
    ),
];

/// Look up a registered DCMI term by absolute IRI.
pub fn term(iri: &str) -> Option<&'static DublinCoreTermDefinition> {
    TERMS.iter().find(|definition| {
        iri.strip_prefix(definition.namespace.iri())
            .is_some_and(|local_name| local_name == definition.local_name)
    })
}
/// Return every current registered DCMI term in deterministic namespace/local-name order.
pub const fn terms() -> &'static [DublinCoreTermDefinition] {
    TERMS
}
