use core::fmt;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

struct DataValue(Value);
struct DataVisitor;
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
        Ok(DataValue(Value::Bool(value)))
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
                .map(DataValue)
                .ok_or_else(|| E::custom("RON number is not supported by the ACRE data model"))
        }
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue(Value::Number(Number::from(value))))
    }
    fn visit_i128<E>(self, value: i128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        i64::try_from(value)
            .map(Number::from)
            .map(Value::Number)
            .map(DataValue)
            .map_err(|_| E::custom("RON integer is outside the ACRE data model range"))
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some((key, DataValue(value))) = map.next_entry::<String, DataValue>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate RON map key '{key}'")));
            }
            values.insert(key, value);
        }
        Ok(DataValue(Value::Object(values)))
    }
    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(DataValue(value)) = sequence.next_element::<DataValue>()? {
            values.push(value);
        }
        Ok(DataValue(Value::Array(values)))
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue(Value::String(value.to_owned())))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue(Value::Number(Number::from(value))))
    }
    fn visit_u128<E>(self, value: u128) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        u64::try_from(value)
            .map(Number::from)
            .map(Value::Number)
            .map(DataValue)
            .map_err(|_| E::custom("RON integer is outside the ACRE data model range"))
    }
    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(DataValue(Value::Null))
    }
}
pub(super) fn parse(content: &str) -> Result<Value, ::ron::error::SpannedError> {
    ::ron::Options::default().from_str::<DataValue>(content).map(|value| value.0)
}
pub(super) fn serialize(value: &Value) -> Result<String, ::ron::Error> {
    let pretty = ::ron::ser::PrettyConfig::new().indentor("  ").new_line("\n");
    ::ron::Options::default().to_string_pretty(value, pretty).map(|mut content| {
        content.push('\n');
        content
    })
}
