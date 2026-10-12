//! Shared mapping between DataCite relation types and DCMI relation properties.
use core::fmt;

/// DCMI relation property paired with its DataCite relation type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Relation {
    /// `dcterms:conformsTo`, DataCite `IsDocumentedBy`.
    ConformsTo,
    /// `dcterms:hasFormat`, DataCite `HasFormat`.
    HasFormat,
    /// `dcterms:hasPart`, DataCite `HasPart`.
    HasPart,
    /// `dcterms:hasVersion`, DataCite `HasVersion`.
    HasVersion,
    /// `dcterms:isFormatOf`, DataCite `IsFormatOf`.
    IsFormatOf,
    /// `dcterms:isPartOf`, DataCite `IsPartOf`.
    IsPartOf,
    /// `dcterms:isReferencedBy`, DataCite `IsCitedBy`.
    IsReferencedBy,
    /// `dcterms:isReplacedBy`, DataCite `IsObsoletedBy`.
    IsReplacedBy,
    /// `dcterms:isRequiredBy`, DataCite `IsRequiredBy`.
    IsRequiredBy,
    /// `dcterms:isVersionOf`, DataCite `IsVersionOf`.
    IsVersionOf,
    /// `dcterms:references`, DataCite `References`.
    References,
    /// `dcterms:relation`, DataCite `Other`.
    Relation,
    /// `dcterms:replaces`, DataCite `Obsoletes`.
    Replaces,
    /// `dcterms:requires`, DataCite `Requires`.
    Requires,
    /// `dcterms:source`, DataCite `IsDerivedFrom`.
    Source,
}
impl fmt::Display for Relation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::ConformsTo => "conformsTo",
            | Self::HasFormat => "hasFormat",
            | Self::HasPart => "hasPart",
            | Self::HasVersion => "hasVersion",
            | Self::IsFormatOf => "isFormatOf",
            | Self::IsPartOf => "isPartOf",
            | Self::IsReferencedBy => "isReferencedBy",
            | Self::IsReplacedBy => "isReplacedBy",
            | Self::IsRequiredBy => "isRequiredBy",
            | Self::IsVersionOf => "isVersionOf",
            | Self::References => "references",
            | Self::Relation => "relation",
            | Self::Replaces => "replaces",
            | Self::Requires => "requires",
            | Self::Source => "source",
        })
    }
}
impl Relation {
    /// Every mapped relation, in DCMI property order.
    pub const ALL: [Self; 15] = [
        Self::ConformsTo,
        Self::HasFormat,
        Self::HasPart,
        Self::HasVersion,
        Self::IsFormatOf,
        Self::IsPartOf,
        Self::IsReferencedBy,
        Self::IsReplacedBy,
        Self::IsRequiredBy,
        Self::IsVersionOf,
        Self::References,
        Self::Relation,
        Self::Replaces,
        Self::Requires,
        Self::Source,
    ];
    /// Resolve a DataCite relation type, using `dcterms:relation` when the type has no DCMI equivalent.
    pub fn from_relation_type(value: &str) -> Self {
        match value {
            | "HasFormat" => Self::HasFormat,
            | "HasPart" => Self::HasPart,
            | "HasVersion" => Self::HasVersion,
            | "IsCitedBy" => Self::IsReferencedBy,
            | "IsDerivedFrom" => Self::Source,
            | "IsDocumentedBy" => Self::ConformsTo,
            | "IsFormatOf" => Self::IsFormatOf,
            | "IsObsoletedBy" => Self::IsReplacedBy,
            | "IsPartOf" => Self::IsPartOf,
            | "IsRequiredBy" => Self::IsRequiredBy,
            | "IsVersionOf" => Self::IsVersionOf,
            | "Obsoletes" => Self::Replaces,
            | "References" => Self::References,
            | "Requires" => Self::Requires,
            | _ => Self::Relation,
        }
    }
    /// DataCite relation type represented by this DCMI property.
    pub const fn relation_type(self) -> &'static str {
        match self {
            | Self::ConformsTo => "IsDocumentedBy",
            | Self::HasFormat => "HasFormat",
            | Self::HasPart => "HasPart",
            | Self::HasVersion => "HasVersion",
            | Self::IsFormatOf => "IsFormatOf",
            | Self::IsPartOf => "IsPartOf",
            | Self::IsReferencedBy => "IsCitedBy",
            | Self::IsReplacedBy => "IsObsoletedBy",
            | Self::IsRequiredBy => "IsRequiredBy",
            | Self::IsVersionOf => "IsVersionOf",
            | Self::References => "References",
            | Self::Relation => "Other",
            | Self::Replaces => "Obsoletes",
            | Self::Requires => "Requires",
            | Self::Source => "IsDerivedFrom",
        }
    }
}
