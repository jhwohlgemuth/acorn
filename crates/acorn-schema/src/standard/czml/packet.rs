//! CZML packets, simulation clocks, and ordered documents.
use super::graphics::OrderedMap;
use super::{ClockRange, ClockStep, CustomProperties, Property, Time, TimeInterval, TimeIntervalCollection};
use acorn_core::prelude::alloc::{String as RustString, Vec};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// Initial simulation clock settings on a document packet.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Clock {
    /// Time interval spanned by the simulation clock.
    pub interval: Option<TimeInterval>,
    /// Current simulation time.
    pub current_time: Option<Time>,
    /// Rate of clock advancement.
    pub multiplier: Option<f64>,
    /// Behavior at the interval boundaries.
    pub range: Option<ClockRange>,
    /// Clock advancement mode.
    pub step: Option<ClockStep>,
}
/// A complete CZML JSON array in packet order.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Document(pub Vec<Packet>);
/// One CZML packet describing or updating an entity.
#[serde_with::skip_serializing_none]
#[derive(bon::Builder, Clone, Debug, eserde::Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Packet {
    /// availability field of this packet.
    #[eserde(compat)]
    pub availability: Option<TimeIntervalCollection>,
    /// billboard field of this packet.
    #[eserde(compat)]
    pub billboard: Option<super::Billboard>,
    /// box field of this packet.
    #[eserde(compat)]
    pub r#box: Option<super::Box>,
    /// clock field of this packet.
    #[eserde(compat)]
    pub clock: Option<Clock>,
    /// corridor field of this packet.
    #[eserde(compat)]
    pub corridor: Option<super::Corridor>,
    /// cylinder field of this packet.
    #[eserde(compat)]
    pub cylinder: Option<super::Cylinder>,
    /// delete field of this packet.
    #[eserde(compat)]
    pub delete: Option<bool>,
    /// description field of this packet.
    #[eserde(compat)]
    pub description: Option<Property<super::String>>,
    /// ellipse field of this packet.
    #[eserde(compat)]
    pub ellipse: Option<super::Ellipse>,
    /// ellipsoid field of this packet.
    #[eserde(compat)]
    pub ellipsoid: Option<super::Ellipsoid>,
    /// id field of this packet.
    #[eserde(compat)]
    pub id: Option<RustString>,
    /// label field of this packet.
    #[eserde(compat)]
    pub label: Option<super::Label>,
    /// model field of this packet.
    #[eserde(compat)]
    pub model: Option<super::Model>,
    /// name field of this packet.
    #[eserde(compat)]
    pub name: Option<RustString>,
    /// orientation field of this packet.
    #[eserde(compat)]
    pub orientation: Option<Property<super::Orientation>>,
    /// parent field of this packet.
    #[eserde(compat)]
    pub parent: Option<RustString>,
    /// path field of this packet.
    #[eserde(compat)]
    pub path: Option<super::Path>,
    /// point field of this packet.
    #[eserde(compat)]
    pub point: Option<super::Point>,
    /// polygon field of this packet.
    #[eserde(compat)]
    pub polygon: Option<super::Polygon>,
    /// polyline field of this packet.
    #[eserde(compat)]
    pub polyline: Option<super::Polyline>,
    /// polylineVolume field of this packet.
    #[eserde(compat)]
    pub polyline_volume: Option<super::PolylineVolume>,
    /// position field of this packet.
    #[eserde(compat)]
    pub position: Option<Property<super::Position>>,
    /// properties field of this packet.
    #[eserde(compat)]
    pub properties: Option<CustomProperties>,
    /// rectangle field of this packet.
    #[eserde(compat)]
    pub rectangle: Option<super::Rectangle>,
    /// tileset field of this packet.
    #[eserde(compat)]
    pub tileset: Option<super::Tileset>,
    /// version field of this packet.
    #[eserde(compat)]
    pub version: Option<RustString>,
    /// viewFrom field of this packet.
    #[eserde(compat)]
    pub view_from: Option<Property<super::ViewFrom>>,
    /// wall field of this packet.
    #[eserde(compat)]
    pub wall: Option<super::Wall>,
    /// Unknown top-level properties retained in source order.
    #[serde(default, flatten)]
    #[eserde(compat)]
    pub extensions: OrderedMap<serde_json::Value>,
}
impl<'de> eserde::EDeserialize<'de> for Document {
    fn deserialize_for_errors<D: Deserializer<'de>>(deserializer: D) -> Result<(), ()> {
        <Vec<Packet> as eserde::EDeserialize>::deserialize_for_errors(deserializer)
    }
}
