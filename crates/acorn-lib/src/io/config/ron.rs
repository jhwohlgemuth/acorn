use core::fmt;
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeSeq},
    Serialize, Serializer,
};
use serde_json::{Number, Value};

pub(super) enum DataValue {
    Scalar(Value),
    Map(Vec<(String, Self)>),
    Sequence(Vec<Self>),
}
struct DataVisitor;
impl DataValue {
    pub(super) fn canonicalize(self, path: &str) -> Self {
        match self {
            | Self::Scalar(_) => self,
            | Self::Sequence(values) => Self::Sequence(values.into_iter().map(|value| value.canonicalize(path)).collect()),
            | Self::Map(entries) => {
                let mut entries = entries
                    .into_iter()
                    .map(|(key, value)| {
                        let child_path = if path.is_empty() { key.clone() } else { format!("{path}.{key}") };
                        (key, value.canonicalize(&child_path))
                    })
                    .collect::<Vec<_>>();
                if path != "auth.profiles" {
                    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
                }
                Self::Map(entries)
            }
        }
    }
}
impl Serialize for DataValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            | Self::Scalar(value) => value.serialize(serializer),
            | Self::Map(entries) => serializer.serialize_map(Some(entries.len())).and_then(|mut map| {
                entries
                    .iter()
                    .try_for_each(|(key, value)| map.serialize_entry(key, value))
                    .and_then(|()| map.end())
            }),
            | Self::Sequence(values) => serializer.serialize_seq(Some(values.len())).and_then(|mut sequence| {
                values
                    .iter()
                    .try_for_each(|value| sequence.serialize_element(value))
                    .and_then(|()| sequence.end())
            }),
        }
    }
}
impl<'de> Deserialize<'de> for DataValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(DataVisitor)
    }
}
impl<'de> Visitor<'de> for DataVisitor {
    type Value = DataValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON-compatible RON value")
    }
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue::Scalar(Value::Bool(value)))
    }
    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(value)
    }
    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if !value.is_finite() {
            Err(E::custom("non-finite RON numbers are not supported by the ACRE data model"))
        } else {
            Number::from_f64(value)
                .map(Value::Number)
                .map(DataValue::Scalar)
                .ok_or_else(|| E::custom("RON number is not supported by the ACRE data model"))
        }
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue::Scalar(Value::Number(Number::from(value))))
    }
    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        i64::try_from(value)
            .map(Number::from)
            .map(Value::Number)
            .map(DataValue::Scalar)
            .map_err(|_| E::custom("RON integer is outside the ACRE data model range"))
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        core::iter::from_fn(|| map.next_entry::<String, DataValue>().transpose())
            .try_fold(Vec::new(), |mut values, entry| {
                entry.and_then(|(key, value)| {
                    if values.iter().any(|(existing, _)| existing == &key) {
                        Err(de::Error::custom(format!("duplicate RON map key '{key}'")))
                    } else {
                        values.push((key, value));
                        Ok(values)
                    }
                })
            })
            .map(DataValue::Map)
    }
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        core::iter::from_fn(|| sequence.next_element::<DataValue>().transpose())
            .try_fold(Vec::new(), |mut values, value| {
                value.map(|value| {
                    values.push(value);
                    values
                })
            })
            .map(DataValue::Sequence)
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue::Scalar(Value::String(value.to_owned())))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue::Scalar(Value::Number(Number::from(value))))
    }
    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u64::try_from(value)
            .map(Number::from)
            .map(Value::Number)
            .map(DataValue::Scalar)
            .map_err(|_| E::custom("RON integer is outside the ACRE data model range"))
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue::Scalar(Value::Null))
    }
}
pub(super) fn parse(content: &str) -> Result<DataValue, ::ron::error::SpannedError> {
    ::ron::Options::default().from_str::<DataValue>(content)
}
pub(super) fn serialize(value: &DataValue) -> Result<String, ::ron::Error> {
    let pretty = ::ron::ser::PrettyConfig::new().indentor("  ").new_line("\n");
    ::ron::Options::default().to_string_pretty(value, pretty).map(|mut content| {
        content.push('\n');
        content
    })
}
