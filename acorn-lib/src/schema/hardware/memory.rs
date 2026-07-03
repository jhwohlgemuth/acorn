//! Data model for harware memory resources, including memory units and (de)serialization logic
use crate::prelude::*;
use alloc::borrow::Cow;
use core::convert::TryInto;
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
                memory_from_number(value as f64)
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Memory, E> {
                memory_from_number(value as f64)
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Memory, E> {
                memory_from_number(value)
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Memory, E> {
                parse_memory_string(value)
            }
        }
        deserializer.deserialize_any(MemoryVisitor)
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
impl Memory {
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
fn memory_from_number<E: serde::de::Error>(amount: f64) -> Result<Memory, E> {
    if amount < 0.0 {
        Err(serde::de::Error::custom("Memory amount cannot be negative"))
    } else {
        Ok(Memory {
            amount,
            unit: MemoryUnit::GB,
        })
    }
}
fn parse_memory_string<E: serde::de::Error>(value: &str) -> Result<Memory, E> {
    let s = value.trim();
    match s.find(|c: char| !c.is_ascii_digit() && c != '.') {
        | Some(split) => match (s.get(..split), s.get(split..)) {
            | (Some(value), Some(unit)) => match value.trim().parse::<f64>() {
                | Ok(amount) => {
                    if amount < 0.0 {
                        Err(serde::de::Error::custom("Memory amount cannot be negative"))
                    } else {
                        match MemoryUnit::deserialize(serde::de::value::StrDeserializer::new(unit.trim())) {
                            | Ok(unit) => Ok(Memory { amount, unit }),
                            | Err(why) => Err(why),
                        }
                    }
                }
                | Err(_) => Err(serde::de::Error::custom(format!("Invalid memory amount — '{value}'"))),
            },
            | _ => Err(serde::de::Error::custom(format!("Invalid memory value — '{s}'"))),
        },
        | None => Err(serde::de::Error::custom(format!("Missing unit in memory value — '{s}'"))),
    }
}
