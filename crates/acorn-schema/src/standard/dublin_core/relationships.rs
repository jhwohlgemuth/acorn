//! Explicit relationships declared by the pinned DCMI vocabularies.
use super::{DublinCoreRelationshipKind, DublinCoreTermRelationship};

macro_rules! dcam_relationship {
    ($subject:literal, $kind:ident, $object:literal) => {
        DublinCoreTermRelationship {
            kind: DublinCoreRelationshipKind::$kind,
            object: $object,
            subject: concat!("http://purl.org/dc/dcam/", $subject),
        }
    };
}
macro_rules! dcterms_relationship {
    ($subject:literal, $kind:ident, $object:literal) => {
        DublinCoreTermRelationship {
            kind: DublinCoreRelationshipKind::$kind,
            object: $object,
            subject: concat!("http://purl.org/dc/terms/", $subject),
        }
    };
}
macro_rules! type_relationship {
    ($subject:literal, $kind:ident, $object:literal) => {
        DublinCoreTermRelationship {
            kind: DublinCoreRelationshipKind::$kind,
            object: $object,
            subject: concat!("http://purl.org/dc/dcmitype/", $subject),
        }
    };
}

const RELATIONSHIPS: &[DublinCoreTermRelationship] = &[
    dcam_relationship!(
        "VocabularyEncodingScheme",
        SeeAlso,
        "https://www.dublincore.org/specifications/dublin-core/2007/06/04/abstract-model/"
    ),
    dcam_relationship!("memberOf", Range, "http://purl.org/dc/dcam/VocabularyEncodingScheme"),
    dcam_relationship!(
        "memberOf",
        SeeAlso,
        "https://www.dublincore.org/specifications/dublin-core/2007/06/04/abstract-model/"
    ),
    dcterms_relationship!("AgentClass", SubClassOf, "http://www.w3.org/2000/01/rdf-schema#Class"),
    dcterms_relationship!("Box", SeeAlso, "https://www.dublincore.org/specifications/dublin-core/dcmi-box/"),
    dcterms_relationship!("DCMIType", SeeAlso, "http://purl.org/dc/dcmitype/"),
    dcterms_relationship!("DDC", SeeAlso, "http://www.oclc.org/dewey/"),
    dcterms_relationship!("FileFormat", SubClassOf, "http://purl.org/dc/terms/MediaType"),
    dcterms_relationship!("IMT", SeeAlso, "http://www.iana.org/assignments/media-types/"),
    dcterms_relationship!(
        "ISO3166",
        SeeAlso,
        "http://www.iso.org/iso/en/prods-services/iso3166ma/02iso-3166-code-lists/list-en1.html"
    ),
    dcterms_relationship!("ISO639-2", SeeAlso, "http://lcweb.loc.gov/standards/iso639-2/langhome.html"),
    dcterms_relationship!("ISO639-3", SeeAlso, "http://www.sil.org/iso639-3/"),
    dcterms_relationship!("Jurisdiction", SubClassOf, "http://purl.org/dc/terms/LocationPeriodOrJurisdiction"),
    dcterms_relationship!("LCC", SeeAlso, "http://lcweb.loc.gov/catdir/cpso/lcco/lcco.html"),
    dcterms_relationship!("LicenseDocument", SubClassOf, "http://purl.org/dc/terms/RightsStatement"),
    dcterms_relationship!("Location", SubClassOf, "http://purl.org/dc/terms/LocationPeriodOrJurisdiction"),
    dcterms_relationship!("MESH", SeeAlso, "http://www.nlm.nih.gov/mesh/meshhome.html"),
    dcterms_relationship!("MediaType", SubClassOf, "http://purl.org/dc/terms/MediaTypeOrExtent"),
    dcterms_relationship!("NLM", SeeAlso, "http://wwwcf.nlm.nih.gov/class/"),
    dcterms_relationship!("Period", SeeAlso, "https://www.dublincore.org/specifications/dublin-core/dcmi-period/"),
    dcterms_relationship!("PeriodOfTime", SubClassOf, "http://purl.org/dc/terms/LocationPeriodOrJurisdiction"),
    dcterms_relationship!("PhysicalMedium", SubClassOf, "http://purl.org/dc/terms/MediaType"),
    dcterms_relationship!("Point", SeeAlso, "https://www.dublincore.org/specifications/dublin-core/dcmi-point/"),
    dcterms_relationship!("RFC1766", SeeAlso, "http://www.ietf.org/rfc/rfc1766.txt"),
    dcterms_relationship!("RFC3066", SeeAlso, "http://www.ietf.org/rfc/rfc3066.txt"),
    dcterms_relationship!("RFC4646", SeeAlso, "http://www.ietf.org/rfc/rfc4646.txt"),
    dcterms_relationship!("RFC5646", SeeAlso, "http://www.ietf.org/rfc/rfc5646.txt"),
    dcterms_relationship!("SizeOrDuration", SubClassOf, "http://purl.org/dc/terms/MediaTypeOrExtent"),
    dcterms_relationship!("TGN", SeeAlso, "http://www.getty.edu/research/tools/vocabulary/tgn/index.html"),
    dcterms_relationship!("UDC", SeeAlso, "http://www.udcc.org/"),
    dcterms_relationship!("URI", SeeAlso, "http://www.ietf.org/rfc/rfc3986.txt"),
    dcterms_relationship!("W3CDTF", SeeAlso, "http://www.w3.org/TR/NOTE-datetime"),
    dcterms_relationship!("accessRights", RangeIncludes, "http://purl.org/dc/terms/RightsStatement"),
    dcterms_relationship!("accrualMethod", Domain, "http://purl.org/dc/dcmitype/Collection"),
    dcterms_relationship!("accrualMethod", RangeIncludes, "http://purl.org/dc/terms/MethodOfAccrual"),
    dcterms_relationship!("accrualPeriodicity", Domain, "http://purl.org/dc/dcmitype/Collection"),
    dcterms_relationship!("accrualPeriodicity", RangeIncludes, "http://purl.org/dc/terms/Frequency"),
    dcterms_relationship!("accrualPolicy", Domain, "http://purl.org/dc/dcmitype/Collection"),
    dcterms_relationship!("accrualPolicy", RangeIncludes, "http://purl.org/dc/terms/Policy"),
    dcterms_relationship!("alternative", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("audience", RangeIncludes, "http://purl.org/dc/terms/AgentClass"),
    dcterms_relationship!("available", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("bibliographicCitation", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("conformsTo", RangeIncludes, "http://purl.org/dc/terms/Standard"),
    dcterms_relationship!("contributor", RangeIncludes, "http://purl.org/dc/terms/Agent"),
    dcterms_relationship!("coverage", RangeIncludes, "http://purl.org/dc/terms/Jurisdiction"),
    dcterms_relationship!("coverage", RangeIncludes, "http://purl.org/dc/terms/Location"),
    dcterms_relationship!("coverage", RangeIncludes, "http://purl.org/dc/terms/Period"),
    dcterms_relationship!("created", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("creator", EquivalentProperty, "http://xmlns.com/foaf/0.1/maker"),
    dcterms_relationship!("creator", RangeIncludes, "http://purl.org/dc/terms/Agent"),
    dcterms_relationship!("date", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("dateAccepted", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("dateCopyrighted", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("dateSubmitted", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("educationLevel", RangeIncludes, "http://purl.org/dc/terms/AgentClass"),
    dcterms_relationship!("extent", RangeIncludes, "http://purl.org/dc/terms/SizeOrDuration"),
    dcterms_relationship!("format", RangeIncludes, "http://purl.org/dc/terms/Extent"),
    dcterms_relationship!("format", RangeIncludes, "http://purl.org/dc/terms/MediaType"),
    dcterms_relationship!("identifier", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("instructionalMethod", RangeIncludes, "http://purl.org/dc/terms/MethodOfInstruction"),
    dcterms_relationship!("issued", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("language", RangeIncludes, "http://purl.org/dc/terms/LinguisticSystem"),
    dcterms_relationship!("license", RangeIncludes, "http://purl.org/dc/terms/LicenseDocument"),
    dcterms_relationship!("mediator", RangeIncludes, "http://purl.org/dc/terms/AgentClass"),
    dcterms_relationship!("medium", DomainIncludes, "http://purl.org/dc/terms/PhysicalResource"),
    dcterms_relationship!("medium", RangeIncludes, "http://purl.org/dc/terms/PhysicalMedium"),
    dcterms_relationship!("modified", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("provenance", RangeIncludes, "http://purl.org/dc/terms/ProvenanceStatement"),
    dcterms_relationship!("publisher", RangeIncludes, "http://purl.org/dc/terms/Agent"),
    dcterms_relationship!("rights", RangeIncludes, "http://purl.org/dc/terms/RightsStatement"),
    dcterms_relationship!("rightsHolder", RangeIncludes, "http://purl.org/dc/terms/Agent"),
    dcterms_relationship!("spatial", RangeIncludes, "http://purl.org/dc/terms/Location"),
    dcterms_relationship!("temporal", RangeIncludes, "http://purl.org/dc/terms/PeriodOfTime"),
    dcterms_relationship!("title", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    dcterms_relationship!("valid", Range, "http://www.w3.org/2000/01/rdf-schema#Literal"),
    type_relationship!("MovingImage", SubClassOf, "http://purl.org/dc/dcmitype/Image"),
    type_relationship!("StillImage", SubClassOf, "http://purl.org/dc/dcmitype/Image"),
];

/// Return every explicit relationship in the pinned DCMI vocabulary snapshot.
pub const fn relationships() -> &'static [DublinCoreTermRelationship] {
    RELATIONSHIPS
}
