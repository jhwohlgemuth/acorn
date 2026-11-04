use super::{
    eyre, fs, ApiResult, ArtifactPolicy, Batch, Deserialize, FromPath, InputLimits, InputOutput, InputReference, JsonSchema, MimeType, Path, PathBuf,
    ResearchActivity, RoutedInputPolicy, Serialize, Standard, StandardCrosswalk, StandardSchema,
};
use core::slice;
use std::fs::read_to_string;

/// One authorized JSONL batch export from a local RAD source.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportBatchInput {
    /// Local JSON or JSONL source beneath the permitted root.
    pub source: PathBuf,
    /// Exact JSONL destination relative to the permitted root.
    pub destination: PathBuf,
    /// Permit replacing an existing artifact.
    #[serde(default)]
    pub overwrite: bool,
    /// Explicit source standard for a crosswalk export.
    pub from: Option<Standard>,
    /// Explicit target standard for a crosswalk export.
    pub to: Option<Standard>,
    /// Reject crosswalk output when it loses fields.
    #[serde(default)]
    pub strict: bool,
}
/// Written JSONL batch artifact.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportBatchOutput {
    /// Canonical artifact path.
    pub destination: PathBuf,
    /// Bounded artifact reference for routed execution.
    pub artifacts: Vec<String>,
    /// Crosswalk field-loss warnings, when applicable.
    pub warnings: Vec<String>,
}
/// One explicit bounded structured crosswalk export.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportCrosswalkInput {
    /// Local source beneath the permitted root.
    pub source: PathBuf,
    /// Exact structured artifact path beneath the permitted root.
    pub destination: PathBuf,
    /// Output format.
    pub format: MimeType,
    /// Explicit source standard.
    pub from: Standard,
    /// Explicit target standard.
    pub to: Standard,
    /// Reject conversions with field-loss warnings.
    #[serde(default)]
    pub strict: bool,
    /// Permit replacing an existing artifact.
    #[serde(default)]
    pub overwrite: bool,
}
/// One confirmed RAD source and exact text document destination.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportDocumentInput {
    /// Source file beneath the permitted root.
    pub source: PathBuf,
    /// Exact document destination beneath the permitted root.
    pub destination: PathBuf,
    /// Output document format.
    pub format: MimeType,
    /// Permit replacing an existing artifact.
    #[serde(default)]
    pub overwrite: bool,
}
/// Written RAD text document.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportDocumentOutput {
    /// Canonical document path.
    pub destination: PathBuf,
    /// Bounded artifact reference for routed execution.
    pub artifacts: Vec<String>,
}
/// One schema export request, without a destination or write authority.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSchemaInput {
    /// Modeled metadata standard.
    pub standard: Standard,
    /// Serialization format.
    pub format: MimeType,
}
/// Schema text matching the direct `export --schema` command.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportSchemaOutput {
    /// Complete JSON or YAML schema text.
    pub content: String,
}
impl ExportBatchInput {
    /// Write a bounded JSONL batch through the direct command's shared batch codec.
    pub fn execute(self, root: &Path) -> ApiResult<ExportBatchOutput> {
        RoutedInputPolicy::new(root, InputLimits::default())
            .and_then(|policy| policy.resolve(&InputReference::File(self.source.clone())))
            .and_then(|resolved| match resolved.files.as_slice() {
                | [file] => Ok(file.path.clone()),
                | _ => Err(eyre!("JSONL export requires one local source file")),
            })
            .and_then(|source| match MimeType::from_path(&self.destination) == MimeType::JsonLines {
                | true => Ok(source),
                | false => Err(eyre!("JSONL export destination must end in .jsonl")),
            })
            .and_then(|source| {
                ArtifactPolicy::new(root, true, self.overwrite).and_then(|policy| {
                    policy
                        .destination(&self.destination, slice::from_ref(&source))
                        .and_then(|destination| match destination == source {
                            | true => Err(eyre!("JSONL export destination must differ from the source")),
                            | false => self.convert(&source).and_then(|converted| {
                                policy
                                    .prepare_destination(&self.destination, slice::from_ref(&source))
                                    .and_then(|destination| {
                                        let written = match converted {
                                            | Some((content, warnings)) => fs::write(&destination, content)
                                                .map_err(|why| eyre!("Write JSONL crosswalk artifact {} — {why}", destination.display()))
                                                .map(|()| warnings),
                                            | None => ResearchActivity::batch_read(&source)
                                                .and_then(|batch| batch.batch_write(destination.clone()))
                                                .map(|()| Vec::new()),
                                        };
                                        written.map(|warnings| ExportBatchOutput {
                                            artifacts: vec![destination.display().to_string()],
                                            destination,
                                            warnings,
                                        })
                                    })
                            }),
                        })
                })
            })
    }
    fn convert(&self, source: &Path) -> ApiResult<Option<(String, Vec<String>)>> {
        match (self.from, self.to) {
            | (None, None) => Ok(None),
            | (Some(from), Some(to)) => read_to_string(source)
                .map_err(|why| eyre!("Read export source {} — {why}", source.display()))
                .and_then(|content| {
                    from.crosswalk_with_warnings(&content, MimeType::from_path(source), to, MimeType::JsonLines)
                        .map_err(|why| eyre!("Crosswalk failed — {why}"))
                })
                .and_then(|conversion| {
                    let (content, warnings) = conversion.into_parts();
                    match self.strict && !warnings.is_empty() {
                        | true => Err(eyre!("Strict mode — {} crosswalk warning(s) found", warnings.len())),
                        | false => Ok(Some((content, warnings.into_iter().map(|warning| warning.to_string()).collect()))),
                    }
                }),
            | _ => Err(eyre!("Crosswalk export requires explicit source and target standards")),
        }
    }
}
impl ExportCrosswalkInput {
    /// Convert one confirmed source and write one exact structured artifact.
    pub fn execute(self, root: &Path) -> ApiResult<ExportBatchOutput> {
        RoutedInputPolicy::new(root, InputLimits::default())
            .and_then(|policy| policy.resolve(&InputReference::File(self.source.clone())))
            .and_then(|resolved| match resolved.files.as_slice() {
                | [file] if resolved.failures.is_empty() => Ok(file.path.clone()),
                | _ => Err(eyre!("Crosswalk export requires one bounded local source file")),
            })
            .and_then(|source| {
                let format = self.format.clone();
                let supported = [
                    MimeType::Json,
                    MimeType::LdJson,
                    MimeType::RdfXml,
                    MimeType::Turtle,
                    MimeType::Yaml,
                    MimeType::Zon,
                ];
                format
                    .ensure_supported(&supported, "Crosswalk export")
                    .map_err(Into::into)
                    .and_then(|()| match MimeType::from_path(&self.destination) == format {
                        | true => Ok(()),
                        | false => Err(eyre!("Crosswalk destination extension does not match the requested format")),
                    })
                    .and_then(|()| {
                        ArtifactPolicy::new(root, true, self.overwrite).and_then(|artifacts| {
                            artifacts
                                .destination(&self.destination, slice::from_ref(&source))
                                .and_then(|destination| match destination == source {
                                    | true => Err(eyre!("Crosswalk destination must differ from the source")),
                                    | false => Ok(destination),
                                })
                                .and_then(|destination| {
                                    read_to_string(&source)
                                        .map_err(|why| eyre!("Read export source {} — {why}", source.display()))
                                        .and_then(|content| {
                                            self.from
                                                .crosswalk_with_warnings(&content, MimeType::from_path(&source), self.to, format)
                                                .map_err(|why| eyre!("Crosswalk failed — {why}"))
                                        })
                                        .and_then(|conversion| {
                                            let (content, warnings) = conversion.into_parts();
                                            match self.strict && !warnings.is_empty() {
                                                | true => Err(eyre!("Strict mode — {} crosswalk warning(s) found", warnings.len())),
                                                | false => destination
                                                    .parent()
                                                    .ok_or_else(|| eyre!("Crosswalk destination has no parent"))
                                                    .and_then(|parent| {
                                                        fs::create_dir_all(parent).map_err(|why| eyre!("Create crosswalk output directory — {why}"))
                                                    })
                                                    .and_then(|()| artifacts.destination(&self.destination, slice::from_ref(&source)))
                                                    .and_then(|destination| {
                                                        fs::write(&destination, content)
                                                            .map_err(|why| eyre!("Write crosswalk artifact {} — {why}", destination.display()))
                                                            .map(|()| ExportBatchOutput {
                                                                artifacts: vec![destination.display().to_string()],
                                                                destination,
                                                                warnings: warnings.into_iter().map(|warning| warning.to_string()).collect(),
                                                            })
                                                    }),
                                            }
                                        })
                                })
                        })
                    })
            })
    }
}
impl ExportDocumentInput {
    /// Write one RAD document through the direct command's shared codec.
    pub fn execute(self, root: &Path) -> ApiResult<ExportDocumentOutput> {
        RoutedInputPolicy::new(root, InputLimits::default())
            .and_then(|policy| policy.resolve(&InputReference::File(self.source.clone())))
            .and_then(|resolved| match resolved.files.as_slice() {
                | [file] => Ok(file.path.clone()),
                | _ => Err(eyre!("Document export requires one local RAD source file")),
            })
            .and_then(|source| {
                let supported = [MimeType::Cff, MimeType::Json, MimeType::Markdown, MimeType::Yaml, MimeType::Zon];
                self.format
                    .ensure_supported(&supported, "Document export")
                    .map_err(Into::into)
                    .and_then(|()| match MimeType::from_path(&self.destination) == self.format {
                        | true => Ok(source),
                        | false => Err(eyre!("Document destination extension does not match the requested format")),
                    })
            })
            .and_then(|source| {
                ArtifactPolicy::new(root, true, self.overwrite).and_then(|policy| {
                    policy
                        .destination(&self.destination, slice::from_ref(&source))
                        .and_then(|destination| match destination == source {
                            | true => Err(eyre!("Document export destination must differ from the source")),
                            | false => ResearchActivity::read(source.clone()).and_then(|document| {
                                policy
                                    .prepare_destination(&self.destination, slice::from_ref(&source))
                                    .and_then(|destination| {
                                        document.write(destination.clone()).map(|()| ExportDocumentOutput {
                                            artifacts: vec![destination.display().to_string()],
                                            destination,
                                        })
                                    })
                            }),
                        })
                })
            })
    }
}
impl ExportSchemaInput {
    /// Render a schema without filesystem effects.
    pub fn execute(self) -> ApiResult<ExportSchemaOutput> {
        self.format
            .ensure_supported(&[MimeType::Json, MimeType::Yaml], "Schema export")
            .map_err(Into::into)
            .and_then(|()| self.standard.schema(&self.format))
            .map(|content| ExportSchemaOutput { content })
    }
}
