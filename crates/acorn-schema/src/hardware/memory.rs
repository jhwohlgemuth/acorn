//! Data model for harware memory resources, including memory units and (de)serialization logic
use acorn_core::prelude::alloc::{format, ToString};
use acorn_core::{AcornError, AcornResult};
use alloc::borrow::Cow;
use core::{convert::TryInto, str::FromStr};
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Memory unit for hardware resources
#[derive(Clone, Copy, Debug, Display, PartialEq, Serialize)]
pub enum MemoryUnit {
    /// Gigabytes
    GB,
    /// Kilobytes
    KB,
    /// Megabytes
    MB,
    /// Terabytes
    TB,
}
/// Memory amount with explicit unit
///
/// Supports string formats like `"80GB"`, `"64KB"`, `"512MB"`, `"1TB"` and plain number (defaults to GB).
#[derive(Clone, Debug, PartialEq)]
pub struct Memory {
    /// Numeric amount of memory
    pub amount: f64,
    /// Unit of measurement
    pub unit: MemoryUnit,
}
impl Memory {
    /// Convert the memory amount to bytes using binary unit multipliers
    ///
    /// Returns `None` when the amount is not finite, is negative, or exceeds `u64`
    pub fn checked_bytes(&self) -> Option<u64> {
        let multiplier = match self.unit {
            | MemoryUnit::KB => 1024_f64,
            | MemoryUnit::MB => 1024_f64.powi(2),
            | MemoryUnit::GB => 1024_f64.powi(3),
            | MemoryUnit::TB => 1024_f64.powi(4),
        };
        let bytes = self.amount * multiplier;
        (bytes.is_finite() && bytes >= 0.0 && bytes < u64::MAX as f64)
            .then(|| format!("{bytes:.0}").parse::<u64>().ok())
            .flatten()
    }
    /// Determine whether a byte count fits in this memory amount
    pub fn can_contain(&self, bytes: u64) -> Option<bool> {
        self.checked_bytes().map(|available| bytes <= available)
    }
    /// Create a Memory value in GB
    pub fn gb(amount: impl Into<f64>) -> Self {
        Memory {
            amount: amount.into(),
            unit: MemoryUnit::GB,
        }
    }
    /// Create a Memory value in KB
    pub fn kb(amount: impl Into<f64>) -> Self {
        Memory {
            amount: amount.into(),
            unit: MemoryUnit::KB,
        }
    }
    /// Create a Memory value in MB
    pub fn mb(amount: impl Into<f64>) -> Self {
        Memory {
            amount: amount.into(),
            unit: MemoryUnit::MB,
        }
    }
    /// Create a Memory value in TB
    pub fn tb(amount: impl Into<f64>) -> Self {
        Memory {
            amount: amount.into(),
            unit: MemoryUnit::TB,
        }
    }
}
impl<'de> Deserialize<'de> for Memory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MemoryVisitor;

        impl<'de> serde::de::Visitor<'de> for MemoryVisitor {
            type Value = Memory;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str(r#"a memory string (e.g. "80GB", "2.5GB", "512MB") or a number (treated as GB)"#)
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Memory, E> {
                memory_from_number(value as f64).map_err(E::custom)
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Memory, E> {
                memory_from_number(value as f64).map_err(E::custom)
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Memory, E> {
                memory_from_number(value).map_err(E::custom)
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Memory, E> {
                value.parse().map_err(E::custom)
            }
        }
        deserializer.deserialize_any(MemoryVisitor)
    }
}
impl FromStr for Memory {
    type Err = AcornError;
    fn from_str(value: &str) -> AcornResult<Self> {
        parse_memory_string(value)
    }
}
impl JsonSchema for Memory {
    fn schema_name() -> Cow<'static, str> {
        "Memory".into()
    }
    fn json_schema(_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
        #[allow(clippy::unwrap_used)]
        serde_json::json!({"type": "string", "pattern": "^\\d+(\\.\\d+)?\\s*(GB|KB|MB|TB)$"})
            .try_into()
            .unwrap()
    }
    fn inline_schema() -> bool {
        true
    }
}
impl Serialize for Memory {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let s = format!("{}{}", self.amount, self.unit);
        serializer.serialize_str(&s)
    }
}
impl<'de> Deserialize<'de> for MemoryUnit {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct MemoryUnitVisitor;
        impl<'de> serde::de::Visitor<'de> for MemoryUnitVisitor {
            type Value = MemoryUnit;

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_str("a memory unit string (e.g. 'GB', 'KB', 'MB', 'TB')")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<MemoryUnit, E> {
                match value.trim().to_uppercase().as_str() {
                    | "GB" | "G" | "GIB" => Ok(MemoryUnit::GB),
                    | "KB" | "K" | "KIB" => Ok(MemoryUnit::KB),
                    | "MB" | "M" | "MIB" => Ok(MemoryUnit::MB),
                    | "TB" | "T" | "TIB" => Ok(MemoryUnit::TB),
                    | other => Err(serde::de::Error::custom(format!("Invalid memory unit: '{other}'"))),
                }
            }
        }
        deserializer.deserialize_str(MemoryUnitVisitor)
    }
}
impl JsonSchema for MemoryUnit {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "MemoryUnit".into()
    }
    fn json_schema(_gen: &mut schemars::generate::SchemaGenerator) -> schemars::Schema {
        #[allow(clippy::unwrap_used)]
        serde_json::json!({"type": "string", "enum": ["GB", "KB", "MB", "TB"]})
            .try_into()
            .unwrap()
    }
    fn inline_schema() -> bool {
        true
    }
}
impl From<Memory> for u64 {
    fn from(memory: Memory) -> Self {
        memory.checked_bytes().unwrap_or_default()
    }
}
fn memory_from_number(amount: f64) -> AcornResult<Memory> {
    if !amount.is_finite() {
        Err(AcornError::new("Memory amount must be finite"))
    } else if amount < 0.0 {
        Err(AcornError::new("Memory amount cannot be negative"))
    } else {
        Ok(Memory {
            amount,
            unit: MemoryUnit::GB,
        })
    }
}
fn parse_memory_string(value: &str) -> AcornResult<Memory> {
    let s = value.trim();
    let suffix = s
        .char_indices()
        .rev()
        .take_while(|(_, character)| character.is_ascii_alphabetic())
        .last()
        .map(|(index, _)| index);
    match suffix {
        | Some(split) => match (s.get(..split), s.get(split..)) {
            | (Some(value), Some(unit)) => match value.trim().parse::<f64>() {
                | Ok(amount) => memory_from_number(amount).and_then(|_| {
                    MemoryUnit::deserialize(serde::de::value::StrDeserializer::<serde::de::value::Error>::new(unit.trim()))
                        .map(|unit| Memory { amount, unit })
                        .map_err(|why| AcornError::new(why.to_string()))
                }),
                | Err(_) => Err(AcornError::new(format!("Invalid memory amount — '{value}'"))),
            },
            | _ => Err(AcornError::new(format!("Invalid memory value — '{s}'"))),
        },
        | None => Err(AcornError::new(format!("Missing unit in memory value — '{s}'"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::{
        prelude::*,
        test_runner::{Config as ProptestConfig, FileFailurePersistence},
    };

    fn finite_amount() -> impl Strategy<Value = f64> {
        prop_oneof![0.0_f64..1.0e12, Just(1.0e-300), Just(1.0e300), Just(f64::MAX)]
    }
    fn memory_unit() -> impl Strategy<Value = MemoryUnit> {
        prop_oneof![Just(MemoryUnit::GB), Just(MemoryUnit::KB), Just(MemoryUnit::MB), Just(MemoryUnit::TB)]
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/hardware/memory.txt"))),
            ..ProptestConfig::default()
        })]
        #[test]
        fn test_memory_round_trips_constructible_values(amount in finite_amount(), unit in memory_unit()) {
            let memory = Memory { amount, unit };
            let encoded = serde_json::to_string(&memory).expect("generated memory should serialize");
            let decoded = serde_json::from_str::<Memory>(&encoded).expect("serialized memory should deserialize");
            prop_assert_eq!(decoded, memory.clone());
            let text = encoded.trim_matches('"');
            let parsed = text.parse::<Memory>().expect("serialized memory should parse directly");
            prop_assert_eq!(parsed, memory);
        }
        #[test]
        fn test_memory_scientific_notation_converges(mantissa in 0.0_f64..10.0, exponent in -300_i16..=300, unit in memory_unit()) {
            let alias = match unit {
                | MemoryUnit::GB => "GiB",
                | MemoryUnit::KB => "kib",
                | MemoryUnit::MB => "mIb",
                | MemoryUnit::TB => "T",
            };
            let source = format!("{mantissa}e{exponent}{alias}");
            let expected = format!("{mantissa}e{exponent}").parse::<f64>();
            match expected {
                | Ok(expected) if expected.is_finite() => {
                    let parsed = source.parse::<Memory>().expect("finite generated memory should parse");
                    prop_assert_eq!(parsed, Memory { amount: expected, unit });
                }
                | _ => prop_assert!(source.parse::<Memory>().is_err()),
            }
        }
    }
    #[test]
    fn test_memory_binary_aliases_are_equivalent() {
        let gb = "24GB".parse::<Memory>().unwrap();
        let gib = "24GiB".parse::<Memory>().unwrap();
        assert_eq!(gb.checked_bytes(), gib.checked_bytes());
        assert_eq!(gb.can_contain(24 * 1024 * 1024 * 1024), Some(true));
    }
    #[test]
    fn test_memory_from_str_and_serde_share_parsing() {
        let parsed = "1.5GB".parse::<Memory>().unwrap();
        let deserialized = serde_json::from_str::<Memory>(r#""1.5GiB""#).unwrap();
        assert_eq!(parsed, deserialized);
        assert_eq!(parsed.checked_bytes(), Some(1_610_612_736));
    }
    #[test]
    fn test_memory_into_u64_uses_binary_units_and_defaults_invalid_values() {
        let kilobytes: u64 = Memory::kb(1).into();
        assert_eq!(kilobytes, 1_024);
        assert_eq!(u64::from(Memory::mb(1.5)), 1_572_864);
        assert_eq!(u64::from(Memory::gb(1)), 1_073_741_824);
        assert_eq!(u64::from(Memory::tb(1)), 1_099_511_627_776);
        assert_eq!(u64::from(Memory::gb(-1.0)), 0);
        assert_eq!(u64::from(Memory::tb(f64::MAX)), 0);
    }
    #[test]
    fn test_memory_rejects_invalid_values_and_checked_overflow() {
        assert!("24XB".parse::<Memory>().is_err());
        assert!("-1GB".parse::<Memory>().is_err());
        assert!("infGB".parse::<Memory>().is_err());
        assert!("NaNGB".parse::<Memory>().is_err());
        assert_eq!(Memory::tb(f64::MAX).checked_bytes(), None);
    }
}
