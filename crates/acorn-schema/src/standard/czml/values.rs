//! Fixed-arity CZML value arrays and time-tagged samples.
use super::{Reference, SampleTime, Time};
use acorn_core::prelude::alloc::{format, String, Vec};
use core::fmt;
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Boolean value shorthand.
pub type BooleanValue = bool;
/// Bounding rectangle with four numeric components.
pub type BoundingRectangleValue = FixedValue<4>;
/// Flat two-component Cartesian list.
pub type Cartesian2ListValue = FlatList<2>;
/// Two-component Cartesian position or offset.
pub type Cartesian2Value = FixedValue<2>;
/// Nested three-component Cartesian lists.
pub type Cartesian3ListOfListsValue = ListOfLists<3>;
/// Flat three-component Cartesian list.
pub type Cartesian3ListValue = FlatList<3>;
/// Three-component Cartesian position or offset.
pub type Cartesian3Value = FixedValue<3>;
/// Cartesian position and velocity, with six numeric components.
pub type Cartesian3VelocityValue = FixedValue<6>;
/// Nested cartographic degrees lists.
pub type CartographicDegreesListOfListsValue = ListOfLists<3>;
/// Flat cartographic degrees list.
pub type CartographicDegreesListValue = FlatList<3>;
/// Longitude, latitude, and height in degrees.
pub type CartographicDegreesValue = FixedValue<3>;
/// Nested cartographic radians lists.
pub type CartographicRadiansListOfListsValue = ListOfLists<3>;
/// Flat cartographic radians list.
pub type CartographicRadiansListValue = FlatList<3>;
/// Longitude, latitude, and height in radians.
pub type CartographicRadiansValue = FixedValue<3>;
/// West, south, east, and north bounds in degrees.
pub type CartographicRectangleDegreesValue = FixedValue<4>;
/// West, south, east, and north bounds in radians.
pub type CartographicRectangleRadiansValue = FixedValue<4>;
/// Near and far distance bounds.
pub type DistanceDisplayConditionValue = FixedValue<2>;
/// List of floating-point numbers.
pub type DoubleListValue = Vec<f64>;
/// Floating-point scalar, one-element array, or sampled scalar array.
pub type DoubleValue = ScalarValue<f64, FixedValue<1>>;
/// CSS font value.
pub type FontValue = String;
/// Integer scalar, one-element array, or sampled scalar array.
pub type IntegerValue = ScalarValue<i64, IntegerArrayValue<1, { i64::MIN }, { i64::MAX }>>;
/// Near and far scalar values with their distances.
pub type NearFarScalarValue = FixedValue<4>;
/// Nested reference lists in source order.
pub type ReferenceListOfListsValue = Vec<Vec<Reference>>;
/// List of references in source order.
pub type ReferenceListValue = Vec<Reference>;
/// One reference value.
pub type ReferenceValue = Reference;
/// Red, green, blue, and alpha as floating-point components.
pub type RgbafValue = FixedValue<4>;
/// Integer RGBA color with components in the inclusive range 0–255.
pub type RgbaValue = IntegerArrayValue<4, 0, 255>;
/// Flat spherical list.
pub type SphericalListValue = FlatList<3>;
/// Clock, cone, and magnitude in spherical coordinates.
pub type SphericalValue = FixedValue<3>;
/// Authored string value.
pub type StringValue = String;
/// Flat unit Cartesian direction list.
pub type UnitCartesian3ListValue = FlatList<3>;
/// Three-component unit Cartesian direction.
pub type UnitCartesian3Value = FixedValue<3>;
/// Four-component unit quaternion.
pub type UnitQuaternionValue = FixedValue<4>;
/// Flat unit spherical direction list.
pub type UnitSphericalListValue = FlatList<2>;
/// Clock and cone in unit spherical coordinates.
pub type UnitSphericalValue = FixedValue<2>;
/// URI or data URI value.
pub type UriValue = String;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Atom {
    Time(String),
    Number(f64),
}
/// Constant components or an ordered sequence of time-tagged component arrays.
#[derive(Clone, Debug, PartialEq)]
pub enum FixedValue<const N: usize> {
    /// Exactly `N` numeric components.
    Constant([f64; N]),
    /// One or more samples, each with a time and exactly `N` components.
    Samples(Vec<(SampleTime, [f64; N])>),
}
/// Constant or sampled integer components constrained to an inclusive range.
#[derive(Clone, Debug, PartialEq)]
pub enum IntegerArrayValue<const N: usize, const MIN: i64, const MAX: i64> {
    /// Exactly `N` integer components.
    Constant([i64; N]),
    /// One or more time-tagged integer samples.
    Samples(Vec<(SampleTime, [i64; N])>),
}
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum IntegerAtom {
    Time(String),
    Number(serde_json::Number),
}
/// A scalar shorthand or the corresponding constant or sampled array form.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ScalarValue<T, A> {
    /// One scalar value applying for all time.
    Shorthand(T),
    /// A constant or sampled array value.
    Array(A),
}
struct FixedVisitor<const N: usize>;
/// A flat list of fixed-size floating-point tuples, without time tags.
#[derive(Clone, Debug, PartialEq)]
pub struct FlatList<const N: usize>(pub Vec<[f64; N]>);
struct IntegerVisitor<const N: usize, const MIN: i64, const MAX: i64>;
/// A list of independent flat fixed-size tuple lists.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ListOfLists<const N: usize>(pub Vec<FlatList<N>>);
impl Atom {
    fn component(self) -> Result<f64, String> {
        match self {
            | Self::Number(number) if number.is_finite() => Ok(number),
            | _ => Err("expected a finite numeric component".into()),
        }
    }
    fn sample_time(self) -> SampleTime {
        match self {
            | Self::Time(value) => SampleTime::Iso(Time(value)),
            | Self::Number(value) => SampleTime::Seconds(value),
        }
    }
}
impl<'de, const N: usize> Deserialize<'de> for FixedValue<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(FixedVisitor::<N>)
    }
}
impl<const N: usize> FixedValue<N> {
    fn from_atoms(atoms: Vec<Atom>) -> Result<Self, String> {
        let length = atoms.len();
        let width = N.saturating_add(1);
        if N > 0 && length == N {
            atoms
                .into_iter()
                .map(Atom::component)
                .collect::<Result<Vec<_>, _>>()
                .and_then(|parts| parts.try_into().map_err(|_| format!("expected {N} components")))
                .map(Self::Constant)
        } else if N > 0 && length >= width && length.is_multiple_of(width) {
            atoms
                .chunks_exact(width)
                .map(|chunk| {
                    let time = chunk
                        .first()
                        .cloned()
                        .map(Atom::sample_time)
                        .ok_or_else(|| format!("expected {N} components after time"));
                    let parts = chunk.iter().skip(1).cloned().map(Atom::component).collect::<Result<Vec<_>, _>>();
                    time.and_then(|time| {
                        parts
                            .and_then(|parts| parts.try_into().map_err(|_| format!("expected {N} components after time")))
                            .map(|parts| (time, parts))
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Samples)
        } else {
            Err(format!("expected {N} constant components or samples of {width} items; got {length}"))
        }
    }
}
impl<const N: usize> JsonSchema for FixedValue<N> {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        format!("FixedValue{N}").into()
    }
    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        let width = N.saturating_add(1);
        json_schema!({
            "oneOf": [
                {"type": "array", "items": {"type": "number"}, "minItems": N, "maxItems": N},
                {"type": "array", "items": {"anyOf": [{"type": "number"}, {"type": "string"}]}, "minItems": width}
            ]
        })
    }
}
impl<const N: usize> Serialize for FixedValue<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let length = match self {
            | Self::Constant(_) => N,
            | Self::Samples(samples) => samples.len().saturating_mul(N.saturating_add(1)),
        };
        serializer.serialize_seq(Some(length)).and_then(|mut sequence| {
            let result = match self {
                | Self::Constant(parts) => parts.iter().try_for_each(|part| sequence.serialize_element(part)),
                | Self::Samples(samples) => samples.iter().try_for_each(|(time, parts)| {
                    sequence
                        .serialize_element(time)
                        .and_then(|()| parts.iter().try_for_each(|part| sequence.serialize_element(part)))
                }),
            };
            result.and_then(|()| sequence.end())
        })
    }
}
impl<'de, const N: usize> Visitor<'de> for FixedVisitor<N> {
    type Value = FixedValue<N>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{N} constant components or time-tagged samples of {} items",
            N.saturating_add(1)
        )
    }
    fn visit_seq<A: SeqAccess<'de>>(self, sequence: A) -> Result<Self::Value, A::Error> {
        Vec::<Atom>::deserialize(serde::de::value::SeqAccessDeserializer::new(sequence))
            .and_then(|atoms| FixedValue::from_atoms(atoms).map_err(A::Error::custom))
    }
}
impl<'de, const N: usize> Deserialize<'de> for FlatList<N> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Vec::<f64>::deserialize(deserializer).and_then(|values| {
            if N == 0 || !values.len().is_multiple_of(N) {
                Err(D::Error::custom(format!("expected a flat list of {N}-component tuples")))
            } else {
                Ok(Self(values.as_chunks::<N>().0.to_vec()))
            }
        })
    }
}
impl<const N: usize> JsonSchema for FlatList<N> {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        format!("FlatList{N}").into()
    }
    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({"type": "array", "items": {"type": "number"}})
    }
}
impl<const N: usize> Serialize for FlatList<N> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_seq(Some(self.0.len().saturating_mul(N))).and_then(|mut sequence| {
            self.0
                .iter()
                .flat_map(|tuple| tuple.iter())
                .try_for_each(|component| sequence.serialize_element(component))
                .and_then(|()| sequence.end())
        })
    }
}
impl<'de, const N: usize, const MIN: i64, const MAX: i64> Deserialize<'de> for IntegerArrayValue<N, MIN, MAX> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_seq(IntegerVisitor::<N, MIN, MAX>)
    }
}
impl<const N: usize, const MIN: i64, const MAX: i64> IntegerArrayValue<N, MIN, MAX> {
    fn from_atoms(atoms: Vec<IntegerAtom>) -> Result<Self, String> {
        let length = atoms.len();
        let width = N.saturating_add(1);
        if N > 0 && length == N {
            atoms
                .into_iter()
                .map(IntegerAtom::component::<MIN, MAX>)
                .collect::<Result<Vec<_>, _>>()
                .and_then(|parts| parts.try_into().map_err(|_| format!("expected {N} components")))
                .map(Self::Constant)
        } else if N > 0 && length >= width && length.is_multiple_of(width) {
            atoms
                .chunks_exact(width)
                .map(|chunk| {
                    let time = chunk
                        .first()
                        .cloned()
                        .ok_or_else(|| format!("expected {N} components after time"))
                        .and_then(IntegerAtom::sample_time);
                    let parts = chunk
                        .iter()
                        .skip(1)
                        .cloned()
                        .map(IntegerAtom::component::<MIN, MAX>)
                        .collect::<Result<Vec<_>, _>>();
                    time.and_then(|time| {
                        parts
                            .and_then(|parts| parts.try_into().map_err(|_| format!("expected {N} components after time")))
                            .map(|parts| (time, parts))
                    })
                })
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Samples)
        } else {
            Err(format!("expected {N} constant components or samples of {width} items; got {length}"))
        }
    }
}
impl<const N: usize, const MIN: i64, const MAX: i64> JsonSchema for IntegerArrayValue<N, MIN, MAX> {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        format!("IntegerArrayValue{N}_{MIN}_{MAX}").into()
    }
    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        let width = N.saturating_add(1);
        json_schema!({
            "oneOf": [
                {"type": "array", "items": {"type": "integer", "minimum": MIN, "maximum": MAX}, "minItems": N, "maxItems": N},
                {"type": "array", "items": {"anyOf": [{"type": "integer"}, {"type": "string"}]}, "minItems": width}
            ]
        })
    }
}
impl<const N: usize, const MIN: i64, const MAX: i64> Serialize for IntegerArrayValue<N, MIN, MAX> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let length = match self {
            | Self::Constant(_) => N,
            | Self::Samples(samples) => samples.len().saturating_mul(N.saturating_add(1)),
        };
        serializer.serialize_seq(Some(length)).and_then(|mut sequence| {
            let result = match self {
                | Self::Constant(parts) => parts.iter().try_for_each(|part| sequence.serialize_element(part)),
                | Self::Samples(samples) => samples.iter().try_for_each(|(time, parts)| {
                    sequence
                        .serialize_element(time)
                        .and_then(|()| parts.iter().try_for_each(|part| sequence.serialize_element(part)))
                }),
            };
            result.and_then(|()| sequence.end())
        })
    }
}
impl IntegerAtom {
    fn component<const MIN: i64, const MAX: i64>(self) -> Result<i64, String> {
        match self {
            | Self::Number(number) => number
                .as_i64()
                .filter(|value| (MIN..=MAX).contains(value))
                .ok_or_else(|| format!("expected an integer component in {MIN}..={MAX}")),
            | Self::Time(_) => Err(format!("expected an integer component in {MIN}..={MAX}")),
        }
    }
    fn sample_time(self) -> Result<SampleTime, String> {
        match self {
            | Self::Time(value) => Ok(SampleTime::Iso(Time(value))),
            | Self::Number(number) => number
                .as_f64()
                .filter(|value| value.is_finite())
                .map(SampleTime::Seconds)
                .ok_or_else(|| "expected a finite sample time".into()),
        }
    }
}
impl<'de, const N: usize, const MIN: i64, const MAX: i64> Visitor<'de> for IntegerVisitor<N, MIN, MAX> {
    type Value = IntegerArrayValue<N, MIN, MAX>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{N} integer components or time-tagged samples of {} items",
            N.saturating_add(1)
        )
    }
    fn visit_seq<A: SeqAccess<'de>>(self, sequence: A) -> Result<Self::Value, A::Error> {
        Vec::<IntegerAtom>::deserialize(serde::de::value::SeqAccessDeserializer::new(sequence))
            .and_then(|atoms| IntegerArrayValue::from_atoms(atoms).map_err(A::Error::custom))
    }
}
