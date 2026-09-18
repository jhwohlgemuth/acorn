use crate::pid::{PID, RRID};
use crate::research_activity::{TypedIdentifier, TypedRelatedResource};
use crate::standard::crosswalk::{ConversionWarning, Crosswalk, CrosswalkError, CrosswalkSchema};
use crate::standard::datacite::{RelatedIdentifier, RelatedIdentifierType, RelationType, ResourceTypeGeneral};
use acorn_core::prelude::alloc::{vec, String, ToString, Vec};

impl Crosswalk<RelatedIdentifier> for TypedRelatedResource {
    fn crosswalk(&self) -> Result<(RelatedIdentifier, Vec<ConversionWarning>), CrosswalkError> {
        let has_rrid_scheme = self.identifier.scheme.as_deref().is_some_and(|scheme| PID::from(scheme) == PID::RRID);
        match has_rrid_scheme {
            | true => self
                .identifier
                .value
                .parse::<RRID>()
                .map_err(|why| CrosswalkError::TransformationFailed {
                    field: "relatedResources[].identifier.value".to_string(),
                    reason: why.to_string(),
                })
                .and_then(|identifier| {
                    self.relation_type
                        .parse::<RelationType>()
                        .map_err(|why| CrosswalkError::TransformationFailed {
                            field: "relatedResources[].relationType".to_string(),
                            reason: why.to_string(),
                        })
                        .and_then(|relation_type| {
                            self.resource_type_general
                                .as_deref()
                                .map(|value| {
                                    value.parse::<ResourceTypeGeneral>().map_err(|why| CrosswalkError::TransformationFailed {
                                        field: "relatedResources[].resourceTypeGeneral".to_string(),
                                        reason: why.to_string(),
                                    })
                                })
                                .transpose()
                                .map(|resource_type_general| {
                                    let warnings = self
                                        .category
                                        .as_ref()
                                        .map(|_| {
                                            vec![ConversionWarning::no_equivalent(
                                                CrosswalkSchema::Rad,
                                                CrosswalkSchema::DataCite,
                                                "relatedResources[].category",
                                            )]
                                        })
                                        .unwrap_or_default();
                                    let related = RelatedIdentifier::init()
                                        .related_identifier(identifier)
                                        .related_identifier_type(RelatedIdentifierType::Rrid)
                                        .relation_type(relation_type)
                                        .maybe_scheme_uri(self.identifier.scheme_uri.clone())
                                        .maybe_resource_type_general(resource_type_general)
                                        .maybe_relation_type_information(self.relation_type_information.clone())
                                        .build();
                                    (related, warnings)
                                })
                        })
                }),
            | false => Err(CrosswalkError::TransformationFailed {
                field: "relatedResources[].identifier.scheme".to_string(),
                reason: "RRID crosswalk requires identifier scheme RRID".to_string(),
            }),
        }
    }
}
impl Crosswalk<TypedRelatedResource> for RelatedIdentifier {
    fn crosswalk(&self) -> Result<(TypedRelatedResource, Vec<ConversionWarning>), CrosswalkError> {
        match &self.related_identifier_type {
            | Some(RelatedIdentifierType::Rrid) => self
                .related_identifier
                .parse::<RRID>()
                .map_err(|why| CrosswalkError::TransformationFailed {
                    field: "relatedIdentifiers[].relatedIdentifier".to_string(),
                    reason: why.to_string(),
                })
                .and_then(|identifier| {
                    self.relation_type
                        .as_ref()
                        .ok_or_else(|| CrosswalkError::MissingRequiredField("relatedIdentifiers[].relationType".to_string()))
                        .map(|relation_type| relation_type.to_string())
                        .map(|relation_type| {
                            let resource_type_general = self.resource_type_general.as_ref().map(ToString::to_string);
                            let locations = [
                                self.related_metadata_scheme
                                    .as_ref()
                                    .map(|_| "relatedIdentifiers[].relatedMetadataScheme"),
                                self.scheme_type.as_ref().map(|_| "relatedIdentifiers[].schemeType"),
                            ];
                            let warnings = locations
                                .into_iter()
                                .flatten()
                                .map(|field| ConversionWarning::no_equivalent(CrosswalkSchema::DataCite, CrosswalkSchema::Rad, field))
                                .collect();
                            let value = TypedIdentifier::init()
                                .value(identifier)
                                .scheme("RRID")
                                .maybe_scheme_uri(self.scheme_uri.clone())
                                .build();
                            let related = TypedRelatedResource::init()
                                .identifier(value)
                                .relation_type(relation_type)
                                .maybe_relation_type_information(self.relation_type_information.clone())
                                .maybe_resource_type_general(resource_type_general)
                                .build();
                            (related, warnings)
                        })
                }),
            | _ => Err(CrosswalkError::TransformationFailed {
                field: "relatedIdentifiers[].relatedIdentifierType".to_string(),
                reason: "RRID crosswalk requires related identifier type RRID".to_string(),
            }),
        }
    }
}
