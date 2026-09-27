//! Native tabular exports for analyzer checks
use crate::analyzer::Check;
use polars::datatypes::PlSmallStr;
use polars::frame::row::Row;
use polars::io::csv::write::CsvWriter;
use polars::prelude::{AnyValue, DataFrame, DataType, PolarsResult, Schema, SerWriter};
use std::io::Cursor;

/// Trait for converting a value to a Polars row.
pub trait IntoRow<'a> {
    /// Convert to a Polars row.
    fn to_row<T>(self) -> Row<'a>;
}
impl<'a> IntoRow<'a> for Check {
    fn to_row<Check>(self) -> Row<'a> {
        let fair = self.fair_details().map(|finding| finding.metadata());
        let Self {
            index,
            category,
            severity,
            message,
            locator,
            uri,
            context,
            ..
        } = self;
        let data = [
            Some(index.to_string()),
            Some(severity.to_string()),
            Some(category.to_string()),
            Some(uri.unwrap_or_default()),
            Some(locator.unwrap_or_default()),
            Some(message),
            Some(context.unwrap_or_default()),
            fair,
        ];
        Row::new(
            data.into_iter()
                .map(|value| value.map_or(AnyValue::Null, |value| AnyValue::StringOwned(value.into())))
                .collect::<Vec<_>>(),
        )
    }
}
/// Convert checks to CSV.
pub fn checks_to_csv(values: &[Check], include_headers: bool) -> PolarsResult<String> {
    checks_to_dataframe(values).and_then(|mut dataframe| {
        let mut buffer = Cursor::new(Vec::new());
        CsvWriter::new(&mut buffer)
            .include_header(include_headers)
            .finish(&mut dataframe)
            .map(|_| String::from_utf8_lossy(&buffer.into_inner()).into_owned())
    })
}
/// Convert checks to a Polars dataframe.
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
/// Convert values to a Polars dataframe with the supplied column names.
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
