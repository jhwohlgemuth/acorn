//! Models, node transforms, and ordered articulation maps.
use super::super::material::validate_optional;
use super::super::{Articulation, Property, PropertyForm, Rotation, Scale, TimeInterval, Translation};
use crate::validation::{report_of, Validate, ValidationReport};
use acorn_core::prelude::alloc::{format, String, Vec};
use acorn_core::validation::ValidationError;
use core::{fmt, marker::PhantomData};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Values assigned to named articulation stages in source order.
pub type Articulations = OrderedMap<Property<Articulation>>;
/// Transformations assigned to named glTF nodes in source order.
pub type NodeTransformations = OrderedMap<Property<NodeTransformation>>;
/// A 3D model.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, serde::Deserialize, schemars::JsonSchema, PartialEq, serde::Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Model {
    /// articulations property of the model graphic.
    pub articulations: Option<Articulations>,
    /// color property of the model graphic.
    pub color: Option<super::super::Property<super::super::Color>>,
    /// colorBlendAmount property of the model graphic.
    pub color_blend_amount: Option<super::super::Property<super::super::Double>>,
    /// colorBlendMode property of the model graphic.
    pub color_blend_mode: Option<super::super::Property<super::super::ColorBlendMode>>,
    /// distanceDisplayCondition property of the model graphic.
    pub distance_display_condition: Option<super::super::Property<super::super::DistanceDisplayCondition>>,
    /// gltf property of the model graphic.
    pub gltf: Option<super::super::Property<super::super::Uri>>,
    /// heightReference property of the model graphic.
    pub height_reference: Option<super::super::Property<super::super::HeightReference>>,
    /// incrementallyLoadTextures property of the model graphic.
    pub incrementally_load_textures: Option<super::super::Property<super::super::Boolean>>,
    /// maximumScale property of the model graphic.
    pub maximum_scale: Option<super::super::Property<super::super::Double>>,
    /// minimumPixelSize property of the model graphic.
    pub minimum_pixel_size: Option<super::super::Property<super::super::Double>>,
    /// nodeTransformations property of the model graphic.
    pub node_transformations: Option<NodeTransformations>,
    /// runAnimations property of the model graphic.
    pub run_animations: Option<super::super::Property<super::super::Boolean>>,
    /// scale property of the model graphic.
    pub scale: Option<super::super::Property<super::super::Double>>,
    /// shadows property of the model graphic.
    pub shadows: Option<super::super::Property<super::super::ShadowMode>>,
    /// show property of the model graphic.
    pub show: Option<super::super::Property<super::super::Boolean>>,
    /// silhouetteColor property of the model graphic.
    pub silhouette_color: Option<super::super::Property<super::super::Color>>,
    /// silhouetteSize property of the model graphic.
    pub silhouette_size: Option<super::super::Property<super::super::Double>>,
}
/// Time-varying translation, rotation, and scale for one glTF node.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NodeTransformation {
    /// Interval in which this transformation applies.
    pub interval: Option<TimeInterval>,
    /// Node rotation.
    pub rotation: Option<Property<Rotation>>,
    /// Node scale.
    pub scale: Option<Property<Scale>>,
    /// Node translation.
    pub translation: Option<Property<Translation>>,
}
/// An object map that retains the source order of its keys.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OrderedMap<T>(pub Vec<(String, T)>);
struct OrderedMapVisitor<T>(PhantomData<T>);
impl Validate for Model {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = ValidationReport::new();
        let report = report.merge("articulations", validate_articulations(&self.articulations));
        let report = report.merge("color", super::super::material::validate_optional(&self.color));
        let report = report.merge("colorBlendAmount", super::super::material::validate_optional(&self.color_blend_amount));
        let report = report.merge("colorBlendMode", super::super::material::validate_optional(&self.color_blend_mode));
        let report = report.merge(
            "distanceDisplayCondition",
            super::super::material::validate_optional(&self.distance_display_condition),
        );
        let report = report.merge("gltf", super::super::material::validate_optional(&self.gltf));
        let report = report.merge("heightReference", super::super::material::validate_optional(&self.height_reference));
        let report = report.merge(
            "incrementallyLoadTextures",
            super::super::material::validate_optional(&self.incrementally_load_textures),
        );
        let report = report.merge("maximumScale", super::super::material::validate_optional(&self.maximum_scale));
        let report = report.merge("minimumPixelSize", super::super::material::validate_optional(&self.minimum_pixel_size));
        let report = report.merge(
            "nodeTransformations",
            super::super::material::validate_optional(&self.node_transformations),
        );
        let report = report.merge("runAnimations", super::super::material::validate_optional(&self.run_animations));
        let report = report.merge("scale", super::super::material::validate_optional(&self.scale));
        let report = report.merge("shadows", super::super::material::validate_optional(&self.shadows));
        let report = report.merge("show", super::super::material::validate_optional(&self.show));
        let report = report.merge("silhouetteColor", super::super::material::validate_optional(&self.silhouette_color));
        let report = report.merge("silhouetteSize", super::super::material::validate_optional(&self.silhouette_size));
        report.finish()
    }
}
impl PropertyForm for NodeTransformation {
    type Scalar = ();
    const ALLOW_SCALAR: bool = false;
}
impl Validate for NodeTransformation {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::new()
            .merge("interval", validate_optional(&self.interval))
            .merge("rotation", validate_optional(&self.rotation))
            .merge("scale", validate_optional(&self.scale))
            .merge("translation", validate_optional(&self.translation))
            .finish()
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for OrderedMap<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(OrderedMapVisitor(PhantomData))
    }
}
impl<T: JsonSchema> JsonSchema for OrderedMap<T> {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        format!("OrderedMapOf{}", T::schema_name()).into()
    }
    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        json_schema!({"type":"object","additionalProperties":generator.subschema_for::<T>()})
    }
}
impl<T: Serialize> Serialize for OrderedMap<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_map(Some(self.0.len())).and_then(|mut map| {
            self.0
                .iter()
                .try_for_each(|(key, value)| map.serialize_entry(key, value))
                .and_then(|()| map.end())
        })
    }
}
impl<T: Validate> Validate for OrderedMap<T> {
    fn validate(&self) -> Result<(), ValidationReport> {
        self.0
            .iter()
            .fold(ValidationReport::new(), |report, (key, value)| {
                report.merge(key, report_of(value.validate()))
            })
            .finish()
    }
}
impl<'de, T: Deserialize<'de>> Visitor<'de> for OrderedMapVisitor<T> {
    type Value = OrderedMap<T>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an ordered object map")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut entries = Vec::new();
        loop {
            match map.next_entry::<String, T>() {
                | Ok(Some(entry)) => entries.push(entry),
                | Ok(None) => break Ok(OrderedMap(entries)),
                | Err(error) => break Err(error),
            }
        }
    }
}
pub(super) fn validate_articulations(value: &Option<Articulations>) -> ValidationReport {
    let report = validate_optional(value);
    value.as_ref().map_or(report.clone(), |articulations| {
        articulations.0.iter().fold(report, |report, (key, _)| {
            let valid = key
                .split_once(' ')
                .is_some_and(|(name, stage)| !name.is_empty() && !stage.is_empty() && !stage.contains(' '));
            match valid {
                | true => report,
                | false => report.merge(
                    key,
                    ValidationReport::from_error(
                        "",
                        ValidationError::new("czml_articulation_key").with_message("Articulation key must contain a name and stage"),
                    ),
                ),
            }
        })
    })
}
