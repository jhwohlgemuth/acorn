#[cfg(feature = "powerpoint")]
use super::powerpoint;
use super::{
    bag_destination, current_date, eyre, folder_size, format_bytes, fs, pdf_destination, updates, ApiResult, ArtifactPolicy, Bag, BagInfo, Builder,
    ExportBagInput, ExportBagOutput, ExportPreviewInput, ExportPreviewOutput, ExportRenderInput, ExportRenderOutput, ExportRenderServices, FromPath,
    InputLimits, InputReference, MimeType, Path, PathBuf, RoutedInputPolicy, Save, StandardCrosswalk, StringConversion,
};
#[cfg(feature = "pdf")]
use super::{initialize_page, render_html_to_pdf, PdfMargins};
#[cfg(any(feature = "pdf", feature = "powerpoint"))]
use super::{InputOutput, ResearchActivity};
#[cfg(feature = "pdf")]
use crate::io::export::updates::html::LogbookUpdatesHtml;
use acorn_host::fs::SafePath;
#[cfg(any(feature = "pdf", feature = "powerpoint"))]
use acorn_schema::{ImageObject, MediaObject};
use core::slice;
use std::fs::create_dir_all;

#[cfg(any(feature = "pdf", feature = "powerpoint"))]
trait BoundedMediaInput: Sized {
    fn read_with_bounded_media(root: &Path, source: &Path) -> ApiResult<Self>;
}
#[cfg(any(feature = "pdf", feature = "powerpoint"))]
impl BoundedMediaInput for ResearchActivity {
    fn read_with_bounded_media(root: &Path, source: &Path) -> ApiResult<Self> {
        Self::read(source.to_path_buf()).and_then(|activity| {
            let limits = InputLimits::default();
            RoutedInputPolicy::new(root, limits.clone()).and_then(|policy| {
                activity
                    .meta
                    .media
                    .iter()
                    .flat_map(|media| media.iter())
                    .filter_map(|media| match media {
                        | MediaObject::Image(ImageObject { content_url: Some(url), .. }) if !url.trim().is_empty() => Some(url.trim()),
                        | _ => None,
                    })
                    .enumerate()
                    .try_fold(0_u64, |total, (index, url)| {
                        source
                            .parent()
                            .unwrap_or_else(|| Path::new("."))
                            .join(url)
                            .canonicalize()
                            .map_err(|why| eyre!("Resolve export media dependency — {why}"))
                            .and_then(|path| policy.resolve(&InputReference::File(path)))
                            .and_then(|resolved| {
                                let bytes = resolved.files.iter().map(|file| u64::from(file.size.clone())).sum::<u64>();
                                match total.checked_add(bytes) {
                                    | Some(total) if index < limits.max_files && limits.max_total.can_contain(total) == Some(true) => Ok(total),
                                    | _ => Err(eyre!("Export media dependencies exceed the bounded input policy")),
                                }
                            })
                    })
                    .map(|_| activity)
            })
        })
    }
}
impl ExportBagInput {
    /// Build and verify a BagIt archive with the direct command's shared `Bag` service.
    pub fn execute(self, root: &Path) -> ApiResult<ExportBagOutput> {
        let supported = [MimeType::Gzip, MimeType::SevenZip, MimeType::Tar, MimeType::Zip];
        self.format
            .ensure_supported(&supported, "BagIt export")
            .map_err(Into::into)
            .and_then(|()| RoutedInputPolicy::new(root, InputLimits::default()))
            .and_then(|policy| {
                policy.resolve_path(&self.source).and_then(|source| {
                    policy.resolve(&InputReference::Directory(self.source)).and_then(|resolved| {
                        match source.is_dir() && resolved.failures.is_empty() && !resolved.files.is_empty() {
                            | false => Err(eyre!("BagIt source is empty or exceeds the bounded input policy")),
                            | true => Ok(()),
                        }
                        .and_then(|()| {
                            let expected = format!(".{}", self.format.file_type());
                            match self.destination.to_string_lossy().ends_with(&expected) {
                                | true => Ok(()),
                                | false => Err(eyre!("BagIt destination does not match the selected archive format")),
                            }
                        })
                        .and_then(|()| {
                            ArtifactPolicy::new(root, true, self.overwrite).and_then(|artifacts| {
                                let sources = resolved.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>();
                                artifacts.destination(&self.destination, &sources).and_then(|destination| {
                                    destination
                                        .parent()
                                        .ok_or_else(|| eyre!("BagIt destination has no parent"))
                                        .and_then(|parent| match parent.starts_with(&source) {
                                            | true => Err(eyre!("BagIt staging must be outside the source directory")),
                                            | false => artifacts.prepare_destination(&self.destination, &sources).and_then(|destination| {
                                                destination
                                                    .parent()
                                                    .ok_or_else(|| eyre!("BagIt destination has no parent"))
                                                    .and_then(|parent| {
                                                        Builder::new()
                                                            .prefix(".acorn-bag-")
                                                            .tempdir_in(parent)
                                                            .map_err(|why| eyre!("Create BagIt staging — {why}"))
                                                    })
                                            }),
                                        })
                                        .and_then(|staging| {
                                            let info = BagInfo::init()
                                                .date(current_date())
                                                .size(format_bytes(folder_size(source.display().to_string())))
                                                .build();
                                            let bag = Bag::init().base_directory(source.display().to_string()).info(info).build();
                                            artifacts.destination(&self.destination, &sources).and_then(|destination| {
                                                bag.save_as(staging.path().to_path_buf(), self.format.clone(), Some(destination.clone()))
                                                    .and_then(|_| Bag::verify(staging.path().to_path_buf()))
                                                    .map(|()| ExportBagOutput {
                                                        artifacts: vec![destination.display().to_string()],
                                                        destination,
                                                    })
                                            })
                                        })
                                })
                            })
                        })
                    })
                })
            })
    }
}
impl ExportPreviewInput {
    /// Plan a local export with the same shared path and conversion operations as the direct CLI.
    pub fn execute(self, root: &Path) -> ApiResult<ExportPreviewOutput> {
        let supported = [
            MimeType::Cff,
            MimeType::Gzip,
            MimeType::Json,
            MimeType::JsonLines,
            MimeType::LdJson,
            MimeType::Markdown,
            MimeType::Pdf,
            MimeType::Powerpoint,
            MimeType::RdfXml,
            MimeType::SevenZip,
            MimeType::Tar,
            MimeType::Turtle,
            MimeType::Yaml,
            MimeType::Zip,
            MimeType::Zon,
        ];
        self.format
            .ensure_supported(&supported, "Export preview")
            .map_err(Into::into)
            .and_then(|()| RoutedInputPolicy::new(root, InputLimits::default()))
            .and_then(|policy| {
                policy.resolve_path(&self.source).and_then(|source| {
                    let reference = match self.format.is_archive() {
                        | true => InputReference::Directory(self.source.clone()),
                        | false => InputReference::File(self.source.clone()),
                    };
                    policy
                        .resolve(&reference)
                        .and_then(|resolved| match resolved.failures.is_empty() && !resolved.files.is_empty() {
                            | false => Err(eyre!("Export preview source is empty or exceeds the bounded input policy")),
                            | true => root.canonicalize().map_err(|why| eyre!("Resolve export root — {why}")).and_then(|base| {
                                self.destination_and_warnings(&base, &source).and_then(|(destination, warnings)| {
                                    destination
                                        .strip_prefix(&base)
                                        .map_err(|_| eyre!("Export preview destination escapes the permitted root"))
                                        .map(Path::to_path_buf)
                                        .and_then(|relative| {
                                            ArtifactPolicy::new(&base, true, true).and_then(|artifacts| {
                                                let sources = resolved.files.iter().map(|file| file.path.clone()).collect::<Vec<_>>();
                                                artifacts.destination(&relative, &sources).map(|_| ExportPreviewOutput {
                                                    source,
                                                    destination,
                                                    warnings,
                                                })
                                            })
                                        })
                                })
                            }),
                        })
                })
            })
    }
    fn destination_and_warnings(&self, base: &Path, source: &Path) -> ApiResult<(PathBuf, Vec<String>)> {
        match &self.format {
            | format if format.is_archive() => self
                .destination_directory(base)
                .and_then(|requested| bag_destination(&requested, Some(format.clone())).map(|(destination, _)| (destination, Vec::new()))),
            | MimeType::Pdf => self
                .destination_directory(base)
                .and_then(|directory| pdf_destination(source, &directory).map(|destination| (destination, Vec::new()))),
            | MimeType::Powerpoint => self.destination_directory(base).map(|directory| {
                let folder = source.parent().unwrap_or_else(|| Path::new("."));
                let name = PathBuf::from(folder.file_name_with_parent()).with_extension(MimeType::Powerpoint.file_type());
                (directory.join(name), Vec::new())
            }),
            | MimeType::JsonLines => self
                .destination_file(base, source.with_extension(MimeType::JsonLines.file_type()))
                .and_then(|destination| {
                    self.crosswalk_warnings(source, MimeType::JsonLines)
                        .map(|warnings| (destination, warnings))
                }),
            | MimeType::Cff => {
                let destination = source.with_file_name(format!("CITATION.{}", MimeType::Cff.file_type()));
                self.destination_file(base, destination).map(|destination| (destination, Vec::new()))
            }
            | MimeType::Json | MimeType::LdJson | MimeType::Markdown | MimeType::RdfXml | MimeType::Turtle | MimeType::Yaml | MimeType::Zon => {
                self.structured_destination(base, source, self.format.clone())
            }
            | _ => Err(eyre!("Unsupported export preview format")),
        }
    }
    fn destination_directory(&self, base: &Path) -> ApiResult<PathBuf> {
        SafePath::new(self.destination.as_deref().unwrap_or_else(|| Path::new("export")))
            .map_err(|why| eyre!("Invalid export destination — {why}"))
            .and_then(|path| path.materialize_under(base).map_err(|why| eyre!("Resolve export destination — {why}")))
    }
    fn destination_file(&self, base: &Path, default: PathBuf) -> ApiResult<PathBuf> {
        self.destination.as_deref().map_or(Ok(default), |destination| {
            SafePath::new(destination)
                .map_err(|why| eyre!("Invalid export destination — {why}"))
                .and_then(|path| path.materialize_under(base).map_err(|why| eyre!("Resolve export destination — {why}")))
        })
    }
    fn structured_destination(&self, base: &Path, source: &Path, format: MimeType) -> ApiResult<(PathBuf, Vec<String>)> {
        self.destination_file(base, source.with_extension(format.file_type()))
            .and_then(|destination| self.crosswalk_warnings(source, format).map(|warnings| (destination, warnings)))
    }
    fn crosswalk_warnings(&self, source: &Path, format: MimeType) -> ApiResult<Vec<String>> {
        match (self.from, self.to) {
            | (None, None) => match format {
                | MimeType::LdJson | MimeType::RdfXml | MimeType::Turtle => {
                    Err(eyre!("Graph export preview requires explicit source and target standards"))
                }
                | _ => Ok(Vec::new()),
            },
            | (Some(from), Some(to)) => fs::read_to_string(source)
                .map_err(|why| eyre!("Read export source {} — {why}", source.display()))
                .and_then(|content| {
                    from.crosswalk_with_warnings(&content, MimeType::from_path(source), to, format)
                        .map_err(|why| eyre!("Crosswalk failed — {why}"))
                })
                .and_then(|conversion| {
                    let (_, warnings) = conversion.into_parts();
                    match self.strict && !warnings.is_empty() {
                        | true => Err(eyre!("Strict mode — {} crosswalk warning(s) found", warnings.len())),
                        | false => Ok(warnings.into_iter().map(|warning| warning.to_string()).collect()),
                    }
                }),
            | _ => Err(eyre!("Crosswalk preview requires explicit source and target standards")),
        }
    }
}
impl ExportRenderInput {
    fn plan(&self, root: &Path) -> ApiResult<(PathBuf, PathBuf, Option<PathBuf>)> {
        let supported = [MimeType::Docx, MimeType::Markdown, MimeType::Pdf, MimeType::Powerpoint];
        self.format
            .ensure_supported(&supported, "Export render")
            .map_err(Into::into)
            .and_then(|()| RoutedInputPolicy::new(root, InputLimits::default()))
            .and_then(|inputs| {
                inputs.resolve(&InputReference::File(self.source.clone())).and_then(|resolved| {
                    match resolved.files.as_slice() {
                        | [file] if resolved.failures.is_empty() => Ok(file.path.clone()),
                        | _ => Err(eyre!("Export render requires one bounded local RAD source file")),
                    }
                    .and_then(|source| {
                        self.reference
                            .as_deref()
                            .map(|reference| inputs.resolve_path(reference))
                            .transpose()
                            .and_then(|reference| {
                                match (&self.format, reference.as_deref()) {
                                    | (MimeType::Powerpoint, Some(path)) if path.is_file() => Ok(()),
                                    | (MimeType::Powerpoint, _) => Err(eyre!("PowerPoint render requires one local reference deck")),
                                    | (_, _) => Ok(()),
                                }
                                .and_then(|()| {
                                    ArtifactPolicy::new(root, true, self.overwrite).and_then(|artifacts| {
                                        artifacts
                                            .destination(&self.destination, slice::from_ref(&source))
                                            .and_then(|destination| {
                                                destination
                                                    .parent()
                                                    .ok_or_else(|| eyre!("Export destination has no parent"))
                                                    .and_then(|parent| self.expected_destination(&source, parent))
                                                    .and_then(|expected| match expected == destination {
                                                        | true => Ok((source, destination, reference)),
                                                        | false => Err(eyre!("Export destination does not match the selected format and source")),
                                                    })
                                            })
                                    })
                                })
                            })
                    })
                })
            })
    }
    fn expected_destination(&self, source: &Path, directory: &Path) -> ApiResult<PathBuf> {
        let destination = match (&self.format, self.since.as_deref()) {
            | (MimeType::Pdf, None) => pdf_destination(source, directory),
            | (MimeType::Powerpoint, _) => {
                let folder = source.parent().unwrap_or_else(|| Path::new("."));
                let name = PathBuf::from(folder.file_name_with_parent()).with_extension(MimeType::Powerpoint.file_type());
                Ok(directory.join(name))
            }
            | (MimeType::Docx | MimeType::Markdown | MimeType::Pdf, Some(since)) => updates::render_updates(source, since)
                .and_then(|projection| updates::updates_artifact_path(&projection, directory, &self.format.file_type())),
            | (MimeType::Docx | MimeType::Markdown, None) => Err(eyre!("Updates render requires a since timestamp")),
            | _ => Err(eyre!("Unsupported export render format")),
        };
        destination.and_then(|destination| {
            destination
                .strip_prefix(directory)
                .map_err(|_| eyre!("Expected export destination escapes its authorized directory"))
                .and_then(|relative| SafePath::new(relative).map_err(|why| eyre!("Invalid expected export destination — {why}")))
                .and_then(|relative| {
                    relative
                        .materialize_under(directory)
                        .map_err(|why| eyre!("Resolve expected export destination — {why}"))
                })
        })
    }
    /// Render one authorized artifact through the same shared command operations.
    pub async fn execute(self, root: &Path, offline: bool, services: &ExportRenderServices) -> ApiResult<ExportRenderOutput> {
        match self.plan(root) {
            | Ok((source, destination, reference)) => {
                let prepared = destination
                    .parent()
                    .ok_or_else(|| eyre!("Export destination has no parent"))
                    .and_then(|parent| create_dir_all(parent).map_err(|why| eyre!("Create export output directory — {why}")))
                    .and_then(|()| {
                        ArtifactPolicy::new(root, true, self.overwrite)
                            .and_then(|artifacts| artifacts.destination(&self.destination, slice::from_ref(&source)).map(|_| ()))
                    });
                match prepared {
                    | Ok(()) => {
                        let result = match (&self.format, self.since.is_some()) {
                            | (MimeType::Pdf, false) => self.write_fact_sheet_pdf(root, &source, &destination, offline, services).await,
                            | (MimeType::Powerpoint, _) => self.write_powerpoint(root, &source, &destination, reference.as_deref(), services),
                            | (MimeType::Docx, true) => self.write_updates(&source, &destination, updates::write_docx_updates),
                            | (MimeType::Markdown, true) => self.write_updates(&source, &destination, updates::write_markdown_updates),
                            | (MimeType::Pdf, true) => self.write_updates_pdf(&source, &destination, offline, services).await,
                            | _ => Err(eyre!("Unsupported export render format")),
                        };
                        result.map(|()| ExportRenderOutput {
                            artifacts: vec![destination.display().to_string()],
                            destination,
                        })
                    }
                    | Err(why) => Err(why),
                }
            }
            | Err(why) => Err(why),
        }
    }
    fn write_updates(
        &self,
        source: &Path,
        destination: &Path,
        write: fn(&acorn_schema::research_activity::LogbookUpdates, &Path) -> ApiResult<()>,
    ) -> ApiResult<()> {
        self.since
            .as_deref()
            .ok_or_else(|| eyre!("Updates render requires a since timestamp"))
            .and_then(|since| updates::render_updates(source, since))
            .and_then(|projection| write(&projection, destination))
    }
    #[cfg(feature = "powerpoint")]
    fn write_powerpoint(
        &self,
        root: &Path,
        source: &Path,
        destination: &Path,
        reference: Option<&Path>,
        services: &ExportRenderServices,
    ) -> ApiResult<()> {
        ResearchActivity::read_with_bounded_media(root, source)
            .and_then(|activity| (services.powerpoint_chart)(&activity))
            .and_then(|chart| {
                destination
                    .parent()
                    .ok_or_else(|| eyre!("PowerPoint destination has no parent"))
                    .and_then(|directory| {
                        powerpoint::render(
                            vec![source.to_path_buf()],
                            source,
                            reference,
                            directory,
                            &[chart],
                            services.fallback_image.as_ref(),
                        )
                    })
            })
            .and_then(|written| match written == destination {
                | true => Ok(()),
                | false => Err(eyre!("PowerPoint renderer wrote an unexpected destination")),
            })
    }
    #[cfg(not(feature = "powerpoint"))]
    fn write_powerpoint(
        &self,
        _root: &Path,
        _source: &Path,
        _destination: &Path,
        _reference: Option<&Path>,
        _services: &ExportRenderServices,
    ) -> ApiResult<()> {
        Err(eyre!("PowerPoint export requires the 'powerpoint' feature"))
    }
    #[cfg(feature = "pdf")]
    async fn write_fact_sheet_pdf(
        &self,
        root: &Path,
        source: &Path,
        destination: &Path,
        offline: bool,
        services: &ExportRenderServices,
    ) -> ApiResult<()> {
        let content = ResearchActivity::read_with_bounded_media(root, source)
            .map(|activity| activity.format_with(Some(source.to_path_buf())))
            .and_then(|activity| (services.fact_sheet_html)(&activity, source));
        self.write_html_pdf(content, Some(source), destination, offline, PdfMargins::FullBleed)
            .await
    }
    #[cfg(not(feature = "pdf"))]
    async fn write_fact_sheet_pdf(
        &self,
        _root: &Path,
        _source: &Path,
        _destination: &Path,
        _offline: bool,
        _services: &ExportRenderServices,
    ) -> ApiResult<()> {
        Err(eyre!("PDF export requires the 'pdf' feature"))
    }
    #[cfg(feature = "pdf")]
    async fn write_updates_pdf(&self, source: &Path, destination: &Path, offline: bool, services: &ExportRenderServices) -> ApiResult<()> {
        let content = self
            .since
            .as_deref()
            .ok_or_else(|| eyre!("Updates render requires a since timestamp"))
            .and_then(|since| updates::render_updates(source, since))
            .map(|projection| projection.to_html(self.full_timestamp, &services.updates_assets));
        self.write_html_pdf(content, None, destination, offline, PdfMargins::Stylesheet).await
    }
    #[cfg(not(feature = "pdf"))]
    async fn write_updates_pdf(&self, _source: &Path, _destination: &Path, _offline: bool, _services: &ExportRenderServices) -> ApiResult<()> {
        Err(eyre!("PDF export requires the 'pdf' feature"))
    }
    #[cfg(feature = "pdf")]
    async fn write_html_pdf(
        &self,
        content: ApiResult<String>,
        source: Option<&Path>,
        destination: &Path,
        offline: bool,
        margins: PdfMargins,
    ) -> ApiResult<()> {
        match content {
            | Ok(content) => match initialize_page(offline, self.chrome_path.as_deref()).await {
                | Ok(session) => {
                    let base = source
                        .map(reqwest::Url::from_file_path)
                        .transpose()
                        .map_err(|()| eyre!("Invalid PDF source base URL"));
                    let navigation = match base {
                        | Ok(Some(url)) => session
                            .page()
                            .goto(url.as_str())
                            .await
                            .map(|_| ())
                            .map_err(|why| eyre!("Set PDF source base — {why}")),
                        | Ok(None) => Ok(()),
                        | Err(why) => Err(why),
                    };
                    let result = match navigation {
                        | Ok(()) => render_html_to_pdf(session.page(), &content, destination, margins).await,
                        | Err(why) => Err(why),
                    };
                    session.close().await;
                    result
                }
                | Err(why) => Err(why),
            },
            | Err(why) => Err(why),
        }
    }
}
