//! Doctor report export using the shared discovery record schema.

use crate::app::DoctorData;
use std::fs::write;
use std::path::Path;
/// Export doctor diagnostic records as JSON.
pub fn export_doctor_report(data: &DoctorData, path: Option<&Path>) -> Result<String, String> {
    serde_json::to_string_pretty(data)
        .map_err(|why| why.to_string())
        .and_then(|json| match path {
            | Some(path) => write(path, &json)
                .map_err(|why| format!("Failed to write report: {why}"))
                .map(|()| path.display().to_string()),
            | None => Ok(json),
        })
}
#[cfg(test)]
mod tests;
