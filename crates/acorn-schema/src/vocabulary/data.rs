//! Embedded vocabulary preparation.
use super::matching::valid_asset_value;
use super::types::{Entry, VocabularyError, VocabularyKind};
use crate::vocabulary::{csv, from_asset};
use crate::Organization;
use acorn_core::prelude::alloc::{format, ToString, Vec};
use core::iter::once;

fn csv_entries(kind: VocabularyKind) -> Result<Vec<Entry>, VocabularyError> {
    let rows = csv(kind.to_string());
    let expected_columns = match kind {
        | VocabularyKind::Keywords | VocabularyKind::Organizations => None,
        | VocabularyKind::Partners | VocabularyKind::Sponsors => Some(3),
        | VocabularyKind::Technology => Some(6),
    };
    rows.iter()
        .enumerate()
        .find(|(_, row)| {
            row.first().is_none_or(|canonical| !valid_asset_value(canonical)) || expected_columns.is_some_and(|expected| row.len() != expected)
        })
        .map_or(Ok(()), |(index, row)| {
            Err(VocabularyError::InvalidAsset {
                message: format!("row {} has {} columns or an invalid canonical value", index.saturating_add(1), row.len()),
                vocabulary: kind,
            })
        })
        .map(|()| {
            rows.into_iter()
                .enumerate()
                .filter_map(|(index, row)| {
                    row.first().map(|canonical| {
                        let (aliases, identifiers) = match kind {
                            | VocabularyKind::Keywords => {
                                (row.iter().skip(1).filter(|value| valid_asset_value(value)).cloned().collect(), Vec::new())
                            }
                            | VocabularyKind::Partners => (
                                row.get(1).cloned().filter(|value| valid_asset_value(value)).into_iter().collect(),
                                row.get(2).cloned().filter(|value| valid_asset_value(value)).into_iter().collect(),
                            ),
                            | VocabularyKind::Sponsors => (
                                row.get(1)
                                    .cloned()
                                    .filter(|value| valid_asset_value(value))
                                    .into_iter()
                                    .flat_map(|alias| {
                                        let short = alias.strip_prefix("DOE-").map(ToString::to_string);
                                        once(alias).chain(short)
                                    })
                                    .collect(),
                                row.get(2).cloned().filter(|value| valid_asset_value(value)).into_iter().collect(),
                            ),
                            | VocabularyKind::Technology => (
                                row.get(2)
                                    .filter(|value| valid_asset_value(value))
                                    .into_iter()
                                    .flat_map(|aliases| aliases.split('|').map(ToString::to_string))
                                    .collect(),
                                [row.get(1), row.get(3)]
                                    .into_iter()
                                    .flatten()
                                    .filter(|value| valid_asset_value(value))
                                    .cloned()
                                    .collect(),
                            ),
                            | VocabularyKind::Organizations => (Vec::new(), Vec::new()),
                        };
                        Entry {
                            aliases,
                            canonical: canonical.trim().to_string(),
                            identifiers,
                            locator: format!("{}[{index}]", kind),
                        }
                    })
                })
                .collect()
        })
}
pub(super) fn entries(kind: VocabularyKind) -> Result<Vec<Entry>, VocabularyError> {
    match kind {
        | VocabularyKind::Organizations => from_asset("organization.json")
            .and_then(|content| serde_json::from_str::<Vec<Organization>>(content).ok())
            .map(|organizations| {
                organizations
                    .iter()
                    .flat_map(|organization| organization_entries(organization, "organizations"))
                    .collect()
            })
            .ok_or_else(|| VocabularyError::InvalidAsset {
                message: "embedded JSON is unavailable or malformed".to_string(),
                vocabulary: kind,
            }),
        | _ => csv_entries(kind),
    }
}
fn organization_entries(organization: &Organization, parent: &str) -> Vec<Entry> {
    let locator = format!("{parent}/{}", organization.name);
    once(Entry {
        aliases: organization.alternative_name.clone().into_iter().collect(),
        canonical: organization.name.clone(),
        identifiers: organization.ror.clone().into_iter().collect(),
        locator: locator.clone(),
    })
    .chain(organization.member.iter().flat_map(|member| organization_entries(member, &locator)))
    .collect()
}
