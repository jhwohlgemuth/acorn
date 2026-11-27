//! Named user properties on a CZML packet.
use super::graphics::OrderedMap;
use super::{CustomProperty, Property};

/// User-defined properties preserved in source order.
pub type CustomProperties = OrderedMap<Property<CustomProperty>>;
