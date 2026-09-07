use super::backend::{self, BackendRow, Connection};
use super::schema::{LogbookEntryRow, ResearchActivityRevisionRow, ResearchActivityRow, Table};
use super::{transaction, Database, Row};
use crate::io::ApiResult;
use acorn_core::util::to_rfc3339;
use acorn_schema::research_activity::{LogbookEntry, ResearchActivity};
use acorn_schema::validation::rules;
use color_eyre::eyre::eyre;
use jiff::Timestamp;
use serde::Serialize;

const GRADUATION_REASON: &str = "logbook-graduation";
const RESEARCH_ACTIVITY_COLUMNS: &str = "id, iid, rad_json, identity_keys_json, provenance_json, created_at, updated_at";

trait ResearchActivityPersistence {
    fn archived_entries(&self) -> Vec<&LogbookEntry>;
    fn persist(&self, connection: &Connection) -> ApiResult<String>;
    fn synchronize_logbook(&self, connection: &Connection, research_activity_iid: &str, now: Timestamp) -> ApiResult<()>;
}
/// Durable result of graduating logbook entries into canonical research activity data.
#[derive(Clone, Debug, Serialize)]
pub struct LogbookGraduationPersistence {
    /// Graduated canonical research activity.
    pub activity: ResearchActivity,
    /// Number of entries newly archived by this graduation.
    pub archived_count: usize,
    /// Portable identifier of the persisted canonical research activity.
    pub research_activity_iid: String,
    /// Portable identifier of the immutable pre-graduation backup.
    pub revision_iid: String,
}
impl Database<Table> {
    /// Graduate accepted logbook entries while transactionally backing up the original activity.
    pub fn graduate_logbook(&self, activity: ResearchActivity, accepted_ids: &[String]) -> ApiResult<LogbookGraduationPersistence> {
        let before = activity.archived_entries().len();
        let original = activity.clone();
        activity.graduate(accepted_ids).map_err(|why| eyre!(why)).and_then(|graduated| {
            let archived_count = graduated.archived_entries().len().saturating_sub(before);
            self.migrate_research_activity_store().and_then(|()| {
                self.with_connection(|connection| {
                    transaction(connection, |connection| {
                        original.persist(connection).and_then(|research_activity_iid| {
                            insert_revision(connection, &research_activity_iid, &original).and_then(|revision_iid| {
                                graduated.persist(connection).map(|_| LogbookGraduationPersistence {
                                    activity: graduated,
                                    archived_count,
                                    research_activity_iid,
                                    revision_iid,
                                })
                            })
                        })
                    })
                })
            })
        })
    }
    /// Load all durable logbook entries linked to a canonical research activity.
    pub fn logbook_entries_by_research_activity(&self, iid: &str) -> ApiResult<Vec<LogbookEntry>> {
        self.migrate_table(Table::LogbookEntries).and_then(|()| {
            self.with_connection(|connection| {
                connection
                    .prepare(
                        "SELECT id, research_activity_iid, entry_identifier, entry_json, timestamp, archived, created_at, updated_at \
                         FROM logbook_entries WHERE research_activity_iid = ? ORDER BY timestamp DESC",
                    )
                    .map_err(|why| eyre!("Failed to prepare logbook entry lookup — {why}"))
                    .and_then(|mut statement| {
                        statement
                            .query_map(backend::params![iid], |row: &BackendRow<'_>| Ok(LogbookEntryRow::from(row)))
                            .map_err(|why| eyre!("Failed to query logbook entries — {why}"))
                            .and_then(|rows| {
                                rows.collect::<core::result::Result<Vec<_>, _>>()
                                    .map_err(|why| eyre!("Failed to read logbook entries — {why}"))
                            })
                    })
                    .and_then(|rows| {
                        rows.into_iter()
                            .map(|row| deserialize_required(row.entry_json.as_deref(), "logbook entry"))
                            .collect()
                    })
            })
        })
    }
    fn migrate_research_activity_store(&self) -> ApiResult<()> {
        [Table::ResearchActivities, Table::LogbookEntries, Table::ResearchActivityRevisions]
            .into_iter()
            .try_fold((), |(), table| self.migrate_table(table))
    }
    /// Persist a complete research activity and synchronize its linked logbook entries.
    pub fn persist_research_activity(&self, activity: &ResearchActivity) -> ApiResult<String> {
        self.migrate_research_activity_store()
            .and_then(|()| self.with_connection(|connection| transaction(connection, |connection| activity.persist(connection))))
    }
    /// Load canonical research activity data by its portable database identifier.
    pub fn research_activity_data_by_iid(&self, iid: &str) -> ApiResult<Option<ResearchActivity>> {
        self.research_activity_by_iid(iid).and_then(|row| {
            row.map(|row| deserialize_required(row.rad_json.as_deref(), "research activity data"))
                .transpose()
        })
    }
    /// Load immutable backups for a canonical research activity, newest first.
    pub fn research_activity_revisions(&self, iid: &str) -> ApiResult<Vec<ResearchActivityRevisionRow>> {
        self.migrate_table(Table::ResearchActivityRevisions).and_then(|()| {
            self.with_connection(|connection| {
                connection
                    .prepare(
                        "SELECT id, iid, research_activity_iid, rad_json, reason, created_at \
                         FROM research_activity_revisions WHERE research_activity_iid = ? ORDER BY created_at DESC",
                    )
                    .map_err(|why| eyre!("Failed to prepare research activity revision lookup — {why}"))
                    .and_then(|mut statement| {
                        statement
                            .query_map(backend::params![iid], |row: &BackendRow<'_>| Ok(ResearchActivityRevisionRow::from(row)))
                            .map_err(|why| eyre!("Failed to query research activity revisions — {why}"))
                            .and_then(|rows| {
                                rows.collect::<core::result::Result<Vec<_>, _>>()
                                    .map_err(|why| eyre!("Failed to read research activity revisions — {why}"))
                            })
                    })
            })
        })
    }
}
impl ResearchActivityPersistence for ResearchActivity {
    fn archived_entries(&self) -> Vec<&LogbookEntry> {
        self.logbook
            .as_ref()
            .map(|logbook| logbook.entries.iter().filter(|entry| entry.archived).collect())
            .unwrap_or_default()
    }
    fn persist(&self, connection: &Connection) -> ApiResult<String> {
        let identity_key = format!("rad:{}", self.meta.identifier);
        ResearchActivityRow::find(connection, &identity_key, &self.meta.identifier).and_then(|existing| {
            let now = Timestamp::now();
            serialize(self).and_then(|rad_json| match existing {
                | Some(row) => row
                    .iid
                    .clone()
                    .ok_or_else(|| eyre!("Persisted research activity is missing iid"))
                    .and_then(|iid| {
                        row.identity_keys_with(identity_key.clone()).and_then(|identity_keys_json| {
                            connection
                                .execute(
                                    "UPDATE research_activities SET rad_json = ?, identity_keys_json = ?, updated_at = ? WHERE iid = ?",
                                    backend::params![rad_json, identity_keys_json, to_rfc3339(now), iid.clone()],
                                )
                                .map_err(|why| eyre!("Failed to update research activity {iid} — {why}"))
                                .and_then(|_| self.synchronize_logbook(connection, &iid, now).map(|()| iid))
                        })
                    }),
                | None => Table::from("research_activities").unique_iid(connection).and_then(|iid| {
                    serialize(&vec![identity_key]).and_then(|identity_keys_json| {
                        ResearchActivityRow::init()
                            .iid(iid.clone())
                            .rad_json(rad_json)
                            .identity_keys_json(identity_keys_json)
                            .provenance_json("[]")
                            .created_at(now)
                            .updated_at(now)
                            .build()
                            .insert(connection)
                            .and_then(|_| self.synchronize_logbook(connection, &iid, now).map(|()| iid))
                    })
                }),
            })
        })
    }
    /// Synchronize the activity's logbook with its durable entry rows.
    fn synchronize_logbook(&self, connection: &Connection, research_activity_iid: &str, now: Timestamp) -> ApiResult<()> {
        self.logbook.as_ref().map_or(Ok(()), |logbook| {
            logbook.entries.iter().try_fold((), |(), entry| {
                serialize(entry).and_then(|entry_json| {
                    rules::parse_timestamp(&entry.timestamp)
                        .map_err(|why| eyre!("Invalid persisted logbook timestamp '{}' — {why}", entry.timestamp))
                        .and_then(|timestamp| {
                            connection
                                .query_row(
                                    "SELECT COUNT(*) FROM logbook_entries WHERE research_activity_iid = ? AND entry_identifier = ?",
                                    backend::params![research_activity_iid, entry.identifier.as_str()],
                                    |row| row.get::<_, i64>(0),
                                )
                                .map_err(|why| eyre!("Failed to locate logbook entry '{}' — {why}", entry.identifier))
                                .and_then(|count| match count {
                                    | 0 => LogbookEntryRow::init()
                                        .research_activity_iid(research_activity_iid)
                                        .entry_identifier(entry.identifier.clone())
                                        .entry_json(entry_json)
                                        .timestamp(timestamp)
                                        .archived(entry.archived)
                                        .created_at(now)
                                        .updated_at(now)
                                        .build()
                                        .insert(connection)
                                        .map(|_| ()),
                                    | _ => connection
                                        .execute(
                                            "UPDATE logbook_entries SET entry_json = ?, timestamp = ?, archived = ?, updated_at = ? \
                                             WHERE research_activity_iid = ? AND entry_identifier = ?",
                                            backend::params![
                                                entry_json,
                                                to_rfc3339(timestamp),
                                                i32::from(entry.archived),
                                                to_rfc3339(now),
                                                research_activity_iid,
                                                entry.identifier.as_str(),
                                            ],
                                        )
                                        .map_err(|why| eyre!("Failed to update logbook entry '{}' — {why}", entry.identifier))
                                        .map(|_| ()),
                                })
                        })
                })
            })
        })
    }
}
impl ResearchActivityRow {
    fn find(connection: &Connection, identity_key: &str, activity_identifier: &str) -> ApiResult<Option<Self>> {
        connection
            .prepare(&format!("SELECT {RESEARCH_ACTIVITY_COLUMNS} FROM research_activities"))
            .map_err(|why| eyre!("Failed to prepare canonical research activity lookup — {why}"))
            .and_then(|mut statement| {
                statement
                    .query_map(backend::params![], |row: &BackendRow<'_>| Ok(Self::from(row)))
                    .map_err(|why| eyre!("Failed to query canonical research activities — {why}"))
                    .and_then(|rows| {
                        rows.collect::<core::result::Result<Vec<_>, _>>()
                            .map_err(|why| eyre!("Failed to read canonical research activities — {why}"))
                    })
            })
            .and_then(|rows| {
                rows.into_iter().try_fold(None, |matched, row| {
                    deserialize_required::<Vec<String>>(row.identity_keys_json.as_deref(), "research activity identity keys").and_then(|keys| {
                        row.persisted_activity_identifier().and_then(|persisted_identifier| {
                            let matches = keys.iter().any(|key| key == identity_key) || persisted_identifier.as_deref() == Some(activity_identifier);
                            match (matched, matches) {
                                | (Some(_), true) => Err(eyre!("Several canonical research activities match {identity_key}")),
                                | (None, true) => Ok(Some(row)),
                                | (matched, false) => Ok(matched),
                            }
                        })
                    })
                })
            })
    }
    fn identity_keys_with(&self, identity_key: String) -> ApiResult<String> {
        deserialize_required::<Vec<String>>(self.identity_keys_json.as_deref(), "research activity identity keys").and_then(|mut keys| {
            keys.push(identity_key);
            keys.sort();
            keys.dedup();
            serialize(&keys)
        })
    }
    fn persisted_activity_identifier(&self) -> ApiResult<Option<String>> {
        deserialize_required::<serde_json::Value>(self.rad_json.as_deref(), "research activity data").map(|rad| {
            rad.pointer("/meta/identifier")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
    }
}
fn deserialize_required<T>(json: Option<&str>, label: &str) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
{
    json.ok_or_else(|| eyre!("Persisted {label} is missing JSON"))
        .and_then(|json| serde_json::from_str(json).map_err(|why| eyre!("Failed to deserialize persisted {label} — {why}")))
}
fn insert_revision(connection: &Connection, research_activity_iid: &str, activity: &ResearchActivity) -> ApiResult<String> {
    Table::from("research_activity_revisions").unique_iid(connection).and_then(|iid| {
        serialize(activity).and_then(|rad_json| {
            ResearchActivityRevisionRow::init()
                .iid(iid.clone())
                .research_activity_iid(research_activity_iid)
                .rad_json(rad_json)
                .reason(GRADUATION_REASON)
                .created_at(Timestamp::now())
                .build()
                .insert(connection)
                .map(|_| iid)
        })
    })
}
fn serialize<T>(value: &T) -> ApiResult<String>
where
    T: Serialize + ?Sized,
{
    serde_json::to_string(value).map_err(|why| eyre!("Failed to serialize research activity database value — {why}"))
}
