//! Native tabular exports for analyzer checks
use crate::analyzer::Check;
#[cfg(feature = "dataframe")]
use polars::datatypes::PlSmallStr;
#[cfg(feature = "dataframe")]
use polars::frame::row::Row;
#[cfg(feature = "dataframe")]
use polars::prelude::{AnyValue, DataFrame, DataType, Schema};
use polars_error::PolarsResult;

/// Trait for converting a value to a Polars row.
#[cfg(feature = "dataframe")]
pub trait IntoRow<'a> {
    /// Convert to a Polars row.
    fn to_row<T>(self) -> Row<'a>;
}
#[cfg(feature = "dataframe")]
impl<'a> IntoRow<'a> for Check {
    fn to_row<Check>(self) -> Row<'a> {
        Row::new(
            check_fields(self)
                .into_iter()
                .map(|value| value.map_or(AnyValue::Null, |value| AnyValue::StringOwned(value.into())))
                .collect::<Vec<_>>(),
        )
    }
}
fn check_fields(check: Check) -> [Option<String>; 8] {
    let fair = check.fair_details().map(|finding| finding.metadata());
    let Check {
        index,
        category,
        severity,
        message,
        locator,
        uri,
        context,
        ..
    } = check;
    [
        Some(index.to_string()),
        Some(severity.to_string()),
        Some(category.to_string()),
        Some(uri.unwrap_or_default()),
        Some(locator.unwrap_or_default()),
        Some(message),
        Some(context.unwrap_or_default()),
        fair,
    ]
}
/// Convert checks to CSV.
pub fn checks_to_csv(values: &[Check], include_headers: bool) -> PolarsResult<String> {
    let mut records = Vec::with_capacity(values.len().saturating_add(usize::from(include_headers)));
    if include_headers {
        records.push("index,severity,category,uri,locator,message,context,fair".to_string());
    }
    records.extend(
        values
            .iter()
            .enumerate()
            .map(|(index, check)| csv_record(check_fields(check.clone().with_index(index)))),
    );
    Ok(match records.is_empty() {
        | true => String::new(),
        | false => format!("{}\n", records.join("\n")),
    })
}
/// Convert checks to a Polars dataframe.
#[cfg(feature = "dataframe")]
pub fn checks_to_dataframe(values: &[Check]) -> PolarsResult<DataFrame> {
    let names = ["index", "severity", "category", "uri", "locator", "message", "context", "fair"];
    to_dataframe::<Check, _, &str>(
        values
            .iter()
            .enumerate()
            .map(|(index, check)| check.clone().with_index(index))
            .collect::<Vec<_>>(),
        names,
    )
}
pub(super) fn csv_field(value: Option<String>) -> String {
    value.map_or_else(String::new, |value| {
        match value.is_empty() || value.chars().any(|character| matches!(character, ',' | '"' | '\n' | '\r')) {
            | true => format!("\"{}\"", value.replace('"', "\"\"")),
            | false => value,
        }
    })
}
fn csv_record(fields: impl IntoIterator<Item = Option<String>>) -> String {
    fields.into_iter().map(csv_field).collect::<Vec<_>>().join(",")
}
/// Convert values to a Polars dataframe with the supplied column names.
#[cfg(feature = "dataframe")]
pub fn to_dataframe<'a, T, I, H>(values: Vec<T>, names: I) -> PolarsResult<DataFrame>
where
    T: IntoRow<'a>,
    H: Into<PlSmallStr>,
    I: IntoIterator<Item = H>,
{
    let rows = values.into_iter().map(|value| value.to_row::<T>()).collect::<Vec<_>>();
    let schema = Schema::from_iter(names.into_iter().map(|name| (name.into(), DataType::String)));
    DataFrame::from_rows_and_schema(&rows, &schema)
}
