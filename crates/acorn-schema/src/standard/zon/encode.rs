use super::ZonErrorCode::{E101, E103};
use super::{decode::resolves_to_compound, validate_encoded, validate_value, Limits, ZonError};
use crate::validation::rules;
use alloc::{format, string::String, string::ToString, vec::Vec};
use serde_json::{Map, Number, Value};
fn canonical_number(value: &Number) -> String {
    let source = value.to_string();
    let lower = source.to_ascii_lowercase();
    match lower.split_once('e') {
        | _ if source.starts_with('-') && source.parse::<f64>().is_ok_and(|value| value == 0.0) => "0".to_string(),
        | None => source,
        | Some((mantissa, exponent)) => expand_exponent(mantissa, exponent.parse::<i32>().unwrap_or_default()),
    }
}
fn encode_compound_value(value: &Value, depth: usize) -> Result<String, ZonError> {
    match value {
        | Value::Array(values) => values
            .iter()
            .map(|value| encode_compound_value(value, depth + 1))
            .collect::<Result<Vec<_>, _>>()
            .map(|values| format!("[{}]", values.join(","))),
        | Value::Object(fields) => encode_inline_object(fields, depth),
        | _ => encode_inline_value(value, depth),
    }
}
fn encode_csv_string(value: &str) -> Result<String, ZonError> {
    rules::zon(value)
        .map_err(|_| ZonError::from(E101))
        .map(|()| match requires_quotes(value) {
            | false => value.to_string(),
            | true => format!("\"{}\"", escape_csv_string(value)),
        })
}
fn encode_inline_object(fields: &Map<String, Value>, depth: usize) -> Result<String, ZonError> {
    sorted_fields(fields)
        .into_iter()
        .map(|(key, value)| {
            encode_key(key).and_then(|key| match value {
                | Value::Object(nested) => encode_inline_object(nested, depth + 1).map(|value| format!("{key}{value}")),
                | Value::Array(values) => values
                    .iter()
                    .map(|value| encode_compound_value(value, depth + 1))
                    .collect::<Result<Vec<_>, _>>()
                    .map(|values| format!("{key}:[{}]", values.join(","))),
                | _ => encode_inline_value(value, depth + 1).map(|value| format!("{key}:{value}")),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|values| format!("{{{}}}", values.join(",")))
}
fn encode_inline_value(value: &Value, depth: usize) -> Result<String, ZonError> {
    match value {
        | Value::Array(values) => values
            .iter()
            .map(|value| encode_compound_value(value, depth + 1))
            .collect::<Result<Vec<_>, _>>()
            .map(|values| format!("\"{}\"", escape_outer_compound(&format!("[{}]", values.join(","))))),
        | Value::Bool(value) => Ok(if *value { "T" } else { "F" }.to_string()),
        | Value::Null => Ok("null".to_string()),
        | Value::Number(value) => Ok(canonical_number(value)),
        | Value::Object(fields) if fields.is_empty() => Ok("\"{}\"".to_string()),
        | Value::Object(fields) => encode_inline_object(fields, depth),
        | Value::String(value) => encode_string(value),
    }
}
fn encode_key(value: &str) -> Result<String, ZonError> {
    rules::zon(value)
        .map_err(|_| ZonError::from(E101))
        .map(|()| match requires_quotes(value) {
            | false => value.to_string(),
            | true => format!("\"{}\"", escape_string(value)),
        })
}
fn encode_root(value: &Value) -> Result<String, ZonError> {
    match value {
        | Value::Object(fields) if fields.is_empty() => Ok(String::new()),
        | Value::Object(fields) => sorted_fields(fields)
            .into_iter()
            .map(|(key, value)| encode_root_field(key, value))
            .collect::<Result<Vec<_>, _>>()
            .map(|lines| lines.join("\n")),
        | Value::Array(values) if table_shape(values).is_some() => encode_table(None, values),
        | _ => encode_inline_value(value, 0),
    }
}
fn encode_root_field(key: &str, value: &Value) -> Result<String, ZonError> {
    encode_key(key).and_then(|key| match value {
        | Value::Array(values) if table_shape(values).is_some() => encode_table(Some(&key), values),
        | Value::Object(fields) if !fields.is_empty() => encode_inline_object(fields, 1).map(|object| format!("{key}{object}")),
        | _ => encode_inline_value(value, 1).map(|value| format!("{key}:{value}")),
    })
}
fn encode_string(value: &str) -> Result<String, ZonError> {
    rules::zon(value)
        .map_err(|_| ZonError::from(E101))
        .map(|()| match requires_quotes(value) {
            | false => value.to_string(),
            | true if resolves_to_compound(value) => {
                let nested = format!("\"{}\"", escape_string(value));
                format!("\"{}\"", escape_string(&nested))
            }
            | true => format!("\"{}\"", escape_string(value)),
        })
}
fn encode_table(key: Option<&str>, values: &[Value]) -> Result<String, ZonError> {
    match table_shape(values) {
        | None => Err(ZonError::from(E103)),
        | Some(columns) => columns
            .iter()
            .map(|column| encode_table_key(column))
            .collect::<Result<Vec<_>, _>>()
            .map(|columns| columns.join(","))
            .and_then(|header_columns| {
                let header = format!(
                    "{}@({}):{header_columns}",
                    key.map(|key| format!("{key}:")).unwrap_or_default(),
                    values.len()
                );
                values
                    .iter()
                    .map(|value| match value {
                        | Value::Object(fields) => columns
                            .iter()
                            .map(|column| fields.get(column).ok_or_else(|| ZonError::from(E103)))
                            .map(|value| value.and_then(encode_table_cell))
                            .collect::<Result<Vec<_>, _>>()
                            .map(|cells| cells.join(",")),
                        | _ => Err(ZonError::from(E103)),
                    })
                    .collect::<Result<Vec<_>, _>>()
                    .map(|rows| core::iter::once(header).chain(rows).collect::<Vec<_>>().join("\n"))
            }),
    }
}
fn encode_table_cell(value: &Value) -> Result<String, ZonError> {
    match value {
        | Value::String(value) => encode_csv_string(value),
        | _ => encode_inline_value(value, 0),
    }
}
fn encode_table_key(value: &str) -> Result<String, ZonError> {
    match requires_quotes(value) {
        | true => encode_csv_string(value),
        | false => Ok(value.to_string()),
    }
}
pub(super) fn encode_value_with_limits(value: &Value, limits: Limits) -> Result<String, ZonError> {
    validate_value(value, 0, limits)
        .and_then(|()| encode_root(value))
        .and_then(|encoded| validate_encoded(&encoded, limits).map(|()| encoded))
}
fn escape_csv_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('"', "\"\"")
}
fn escape_outer_compound(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
fn escape_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
fn expand_exponent(mantissa: &str, exponent: i32) -> String {
    let negative = mantissa.starts_with('-');
    let unsigned = mantissa.trim_start_matches('-');
    let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    let digits = format!("{whole}{fraction}");
    let decimal = whole.len() as i32 + exponent;
    let expanded = if decimal <= 0 {
        format!("0.{}{}", "0".repeat((-decimal) as usize), digits)
    } else if decimal as usize >= digits.len() {
        format!("{}{}", digits, "0".repeat(decimal as usize - digits.len()))
    } else {
        let split = decimal as usize;
        format!("{}.{}", &digits[..split], &digits[split..])
    };
    let normalized = match expanded.contains('.') {
        | true => expanded.trim_end_matches('0').trim_end_matches('.').to_string(),
        | false => expanded,
    };
    let normalized = if normalized.is_empty() { "0".to_string() } else { normalized };
    match negative && normalized != "0" {
        | true => format!("-{normalized}"),
        | false => normalized,
    }
}
fn requires_quotes(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    value.is_empty()
        || value.trim() != value
        || value
            .chars()
            .any(|character| matches!(character, ',' | ':' | '[' | ']' | '{' | '}' | '"' | '\'' | '\\' | '\n' | '\r' | '\t'))
        || matches!(lower.as_str(), "f" | "false" | "nil" | "none" | "null" | "t" | "true")
        || value
            .strip_prefix('@')
            .and_then(|rest| rest.chars().next())
            .is_some_and(|character| character == '(' || character.is_ascii_digit())
        || rules::numeric(value).is_ok()
}
fn table_shape(values: &[Value]) -> Option<Vec<String>> {
    values.first().and_then(Value::as_object).and_then(|first| {
        let columns = first.keys().cloned().collect::<Vec<_>>();
        (!columns.is_empty()
            && values.iter().all(|value| {
                value.as_object().is_some_and(|fields| {
                    fields.len() == columns.len()
                        && columns
                            .iter()
                            .all(|column| fields.get(column).is_some_and(|value| !value.is_array() && !value.is_object()))
                })
            }))
        .then_some(columns)
    })
}
fn sorted_fields(fields: &Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut fields = fields.iter().collect::<Vec<_>>();
    fields.sort_by(|left, right| left.0.cmp(right.0));
    fields
}
