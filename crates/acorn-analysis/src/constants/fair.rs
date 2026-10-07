//! Constants used to normalize and assess FAIR metadata evidence.

/// Metadata names that identify an access or landing-page location.
pub const ACCESS_URI_NAMES: [&str; 5] = ["accessURL", "alternate_url", "landingPage", "url", "websites"];
/// Metadata names that describe authentication or access restrictions.
pub const AUTHENTICATION_NAMES: [&str; 5] = ["accessRestriction", "accessRights", "authentication", "embargo", "restricted"];
/// Metadata names that identify creators or contributors.
pub const CREATOR_NAMES: [&str; 4] = ["contributor", "contributors", "creator", "creators"];
/// Metadata names that describe data formats.
pub const DATA_FORMAT_NAMES: [&str; 5] = ["compressFormat", "format", "formats", "mediaType", "packageFormat"];
/// Metadata names that identify data or digital objects.
pub const DATA_IDENTIFIER_NAMES: [&str; 5] = ["@id", "dataset_id", "doi", "id", "identifier"];
/// Metadata names that contain descriptions or abstracts.
pub const DESCRIPTION_NAMES: [&str; 3] = ["abstract", "description", "descriptions"];
/// Metadata names that identify direct, machine-retrievable data locations.
pub const DIRECT_URI_NAMES: [&str; 3] = ["downloadURL", "endpointURL", "url"];
/// Metadata names that describe reuse licenses.
pub const LICENSE_NAMES: [&str; 5] = ["license", "license_url", "rights", "rights_list", "rightsList"];
/// Tokens commonly used by standard reuse licenses.
pub const LICENSE_TOKENS: [&str; 7] = ["apache-", "bsd-", "cc-", "gpl-", "lgpl-", "mit", "mpl-"];
/// Metadata names that identify metadata records.
pub const METADATA_IDENTIFIER_NAMES: [&str; 3] = ["metadata_id", "metadataId", "metadataIdentifier"];
/// Terms that describe open access.
pub const OPEN_ACCESS_TERMS: [&str; 3] = ["open", "public", "unrestricted"];
/// Metadata names that describe provenance.
pub const PROVENANCE_NAMES: [&str; 3] = ["enrichment", "source-of-data", "wasGeneratedBy"];
/// Metadata names that identify publishers or repositories.
pub const PUBLISHER_NAMES: [&str; 2] = ["publisher", "repository"];
/// Metadata names that describe qualified relationships.
pub const QUALIFIED_RELATION_NAMES: [&str; 2] = ["qualifiedRelation", "relationType"];
/// Metadata names that describe relationships to other resources.
pub const RELATION_NAMES: [&str; 5] = [
    "related_identifiers",
    "related_object",
    "relatedIdentifiers",
    "relatedResources",
    "relation",
];
/// Metadata names that describe retention policies.
pub const RETENTION_NAMES: [&str; 2] = ["retention", "tombstone"];
/// Metadata names that contain subjects, keywords, or classifications.
pub const SUBJECT_NAMES: [&str; 6] = ["keyword", "keywords", "subject", "subjects", "theme", "themes"];
/// Metadata names that contain human-readable titles.
pub const TITLE_NAMES: [&str; 2] = ["title", "titles"];
/// Metadata names that identify controlled vocabularies or schemas.
pub const VOCABULARY_NAMES: [&str; 6] = ["@context", "conformsTo", "schemaURI", "schemaUri", "schemeURI", "valueUri"];
