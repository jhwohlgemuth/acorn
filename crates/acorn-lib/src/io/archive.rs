//! Archive creation and extraction utilities.
use crate::{
    io::{files_all, standard_project_folder, ApiResult},
    util::constants::app::{ARCHIVE_INFERENCE_BYTES, MAX_ARCHIVE_ENTRIES, MAX_ARCHIVE_EXPANDED_BYTES},
};
use acorn_core::{prelude::Component, util::MimeType};
use acorn_host::fs::SafePath;
use alloc::collections::BTreeSet;
use color_eyre::{eyre::eyre, Report};
use core::{fmt, iter::once};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use sevenz_rust2::{ArchiveEntry, ArchiveReader, ArchiveWriter, Password};
use std::{
    fs::{create_dir_all, remove_file, File, OpenOptions},
    io::{self, Cursor, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};
use tar::{Archive as TarArchive, Builder as TarBuilder};
use zip::{write::SimpleFileOptions, ZipArchive, ZipWriter};
use zstd::stream::read::Decoder as ZstdDecoder;

/// Archive creation operations implemented by archive candidates
pub trait ArchiveCreation {
    /// Creates a 7z archive
    fn archive_7z(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Creates an uncompressed TAR archive
    fn archive_tar(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Creates a gzip-compressed TAR archive
    fn archive_tar_gzip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Creates a ZIP archive
    fn archive_zip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
}
trait ArchiveEntryExt {
    fn extract(&self, reader: &mut dyn Read, root: &Path) -> Result<bool, sevenz_rust2::Error>;
    fn validate(&self, seen: Vec<SafePath>) -> ApiResult<Vec<SafePath>>;
}
/// Archive extraction operations implemented by archive candidates
pub trait ArchiveExtraction {
    /// Extracts a 7z archive
    fn extract_7z(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Extracts an uncompressed TAR archive
    fn extract_tar(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Extracts a gzip-compressed TAR archive
    fn extract_tar_gzip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Extracts a Zstandard-compressed TAR archive.
    fn extract_tar_zstd(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Extracts a ZIP archive
    fn extract_zip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
}
/// Operations supported by archive MIME types.
pub trait ArchiveFormat {
    /// Creates an archive from `source`.
    fn archive(&self, candidate: ArchiveCandidate, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
    /// Extracts an archive into `destination`.
    fn extract(&self, candidate: ArchiveCandidate, destination: Option<PathBuf>) -> ApiResult<PathBuf>;
}
#[derive(Debug)]
enum ArchiveError {
    DestinationInsideSource,
    DestinationInvalid,
    DestinationMissingFilename,
    DuplicatePath(PathBuf),
    EntryExpandedSizeLimit,
    ExpandedSizeLimit,
    ExpandedSizeOverflow,
    FormatInference(PathBuf),
    InspectArchive { path: PathBuf, reason: Box<str> },
    InspectSourceChild { path: PathBuf, reason: Box<str> },
    OutputInvalid,
    ReadArchive { path: PathBuf, reason: Box<str> },
    ReplaceDestination(Box<str>),
    ResolveSource(Box<str>),
    SevenZip { operation: &'static str, reason: Box<str> },
    SourceLink(PathBuf),
    TooManyEntries,
    UnsafeZipPath(Box<str>),
    UnsupportedFormat(MimeType),
    UnsupportedTarEntry,
    ZipLink(PathBuf),
}
/// Source content for one deterministic ZIP entry.
#[derive(Clone, Debug)]
pub enum ZipEntrySource {
    /// In-memory entry content.
    Bytes(Vec<u8>),
    /// Regular file entry content.
    File(PathBuf),
}
/// A local path that may be created as or extracted from an archive.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArchiveCandidate(PathBuf);
/// One validated entry in a deterministic ZIP archive.
#[derive(Clone, Debug)]
pub struct DeterministicZipEntry {
    destination: SafePath,
    mode: u32,
    source: ZipEntrySource,
}
impl ArchiveCandidate {
    /// Creates an archive using the selected MIME type.
    pub fn archive(self, destination: Option<PathBuf>, mime_type: MimeType) -> ApiResult<PathBuf> {
        mime_type.archive(self, destination)
    }
    /// Extracts this archive, inferring its format when no override is supplied.
    pub fn extract(self, destination: Option<PathBuf>, archive_format: Option<MimeType>) -> ApiResult<PathBuf> {
        archive_format
            .map_or_else(|| self.infer(), Ok)
            .and_then(|format| format.extract(self, destination))
    }
    fn append_tar<W: Write>(self, writer: W) -> ApiResult<()> {
        self.relative_children().and_then(|children| {
            children
                .into_iter()
                .try_fold(TarBuilder::new(writer), |mut archive, (path, relative)| {
                    archive.append_path_with_name(path, relative).map(|()| archive).map_err(Into::into)
                })
                .and_then(|mut archive| archive.finish().map_err(Into::into))
        })
    }
    fn infer(&self) -> ApiResult<MimeType> {
        File::open(&self.0)
            .map_err(|why| ArchiveError::ReadArchive {
                path: self.0.clone(),
                reason: why.to_string().into(),
            })
            .and_then(|file| {
                let mut header = Vec::new();
                file.take(ARCHIVE_INFERENCE_BYTES)
                    .read_to_end(&mut header)
                    .map(|_| header)
                    .map_err(|why| ArchiveError::InspectArchive {
                        path: self.0.clone(),
                        reason: why.to_string().into(),
                    })
            })
            .map_err(Into::into)
            .and_then(|header| MimeType::infer(&header).ok_or_else(|| ArchiveError::FormatInference(self.0.clone()).into()))
    }
    fn prepare_output(&self, destination: Option<PathBuf>, format: MimeType) -> ApiResult<(File, PathBuf)> {
        let output = destination.unwrap_or_else(|| self.0.with_extension(format.file_type()));
        self.0
            .canonicalize()
            .map_err(|why| Report::from(ArchiveError::ResolveSource(why.to_string().into())))
            .and_then(|root| {
                let parent = output
                    .parent()
                    .filter(|path| !path.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
                parent
                    .canonicalize()
                    .map_err(Into::into)
                    .and_then(|parent| {
                        output
                            .file_name()
                            .map(|name| parent.join(name))
                            .ok_or_else(|| ArchiveError::DestinationMissingFilename.into())
                    })
                    .map(|output| (root, output))
            })
            .and_then(|(root, resolved)| match resolved.starts_with(&root) {
                | true => Err(ArchiveError::DestinationInsideSource.into()),
                | false => prepare_archive_destination(&output).map(|file| (file, output)),
            })
    }
    fn relative_children(&self) -> ApiResult<Vec<(PathBuf, SafePath)>> {
        self.0
            .canonicalize()
            .map_err(|why| Report::from(ArchiveError::ResolveSource(why.to_string().into())))
            .and_then(|root| {
                files_all(root.clone(), None::<Vec<String>>)
                    .into_iter()
                    .map(|path| {
                        path.symlink_metadata()
                            .map_err(|why| {
                                Report::from(ArchiveError::InspectSourceChild {
                                    path: path.clone(),
                                    reason: why.to_string().into(),
                                })
                            })
                            .and_then(|metadata| match metadata.file_type().is_symlink() {
                                | true => Err(ArchiveError::SourceLink(path.clone()).into()),
                                | false => path.canonicalize().map_err(Into::into).and_then(|absolute| {
                                    absolute
                                        .strip_prefix(&root)
                                        .map(Path::to_path_buf)
                                        .map_err(Into::into)
                                        .and_then(|relative| SafePath::new(relative).map_err(Into::into))
                                        .map(|relative| (absolute, relative))
                                }),
                            })
                    })
                    .collect()
            })
    }
}
impl ArchiveCreation for ArchiveCandidate {
    fn archive_7z(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        self.prepare_output(destination, MimeType::SevenZip).and_then(|(file, output)| {
            ArchiveWriter::new(file)
                .map_err(|why| {
                    Report::from(ArchiveError::SevenZip {
                        operation: "create 7z archive",
                        reason: why.to_string().into(),
                    })
                })
                .and_then(|writer| {
                    self.relative_children().and_then(|children| {
                        children
                            .into_iter()
                            .try_fold(writer, |mut writer, (path, relative)| {
                                let entry = ArchiveEntry::from_path(&path, relative.as_path().to_string_lossy().replace('\\', "/"));
                                path.is_file()
                                    .then(|| File::open(&path))
                                    .transpose()
                                    .map_err(Into::into)
                                    .and_then(|reader| match writer.push_archive_entry(entry, reader) {
                                        | Ok(_) => Ok(writer),
                                        | Err(why) => Err(ArchiveError::SevenZip {
                                            operation: "add 7z entry",
                                            reason: why.to_string().into(),
                                        }
                                        .into()),
                                    })
                            })
                            .and_then(|writer| {
                                writer.finish().map_err(|why| {
                                    ArchiveError::SevenZip {
                                        operation: "finish 7z archive",
                                        reason: why.to_string().into(),
                                    }
                                    .into()
                                })
                            })
                    })
                })
                .map(|_| output)
        })
    }
    fn archive_tar(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        self.prepare_output(destination, MimeType::Tar)
            .and_then(|(file, output)| self.append_tar(file).map(|()| output))
    }
    fn archive_tar_gzip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        self.prepare_output(destination, MimeType::Gzip)
            .and_then(|(file, output)| self.append_tar(GzEncoder::new(file, Compression::default())).map(|()| output))
    }
    fn archive_zip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        self.prepare_output(destination, MimeType::Zip).and_then(|(file, output)| {
            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            self.relative_children()
                .and_then(|children| {
                    children
                        .into_iter()
                        .try_fold(ZipWriter::new(file), |mut writer, (path, relative)| match path.is_dir() {
                            | true => writer.add_directory_from_path(relative, options).map(|()| writer).map_err(Into::into),
                            | false => writer
                                .start_file_from_path(relative, options)
                                .map_err(Into::into)
                                .and_then(|()| File::open(path).map_err(Into::into))
                                .and_then(|mut input| io::copy(&mut input, &mut writer).map_err(Into::into))
                                .map(|_| writer),
                        })
                })
                .and_then(|writer| writer.finish().map_err(Into::into))
                .map(|_| output)
        })
    }
}
impl ArchiveExtraction for ArchiveCandidate {
    fn extract_7z(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        extraction_root(destination).and_then(|root| {
            File::open(&self.0)
                .map_err(Into::into)
                .and_then(|file| {
                    ArchiveReader::new(file, Password::empty()).map_err(|why| {
                        ArchiveError::SevenZip {
                            operation: "read 7z archive",
                            reason: why.to_string().into(),
                        }
                        .into()
                    })
                })
                .and_then(|mut archive| {
                    let validation = {
                        let entries = archive.archive().files.as_slice();
                        let size = entries.iter().try_fold(0_u64, |total, entry| {
                            total.checked_add(entry.size()).ok_or_else(|| ArchiveError::ExpandedSizeOverflow.into())
                        });
                        match (entries.len() > MAX_ARCHIVE_ENTRIES, size) {
                            | (true, _) => Err(ArchiveError::TooManyEntries.into()),
                            | (_, Ok(size)) if size > MAX_ARCHIVE_EXPANDED_BYTES => Err(ArchiveError::ExpandedSizeLimit.into()),
                            | (_, Err(why)) => Err(why),
                            | _ => entries.iter().try_fold(Vec::new(), |seen, entry| entry.validate(seen)).and_then(|paths| {
                                paths
                                    .iter()
                                    .try_for_each(|relative| relative.materialize_under(&root).map(|_| ()).map_err(Into::into))
                            }),
                        }
                    };
                    validation.and_then(|()| {
                        archive.for_each_entries(|entry, reader| entry.extract(reader, &root)).map_err(|why| {
                            ArchiveError::SevenZip {
                                operation: "extract 7z archive",
                                reason: why.to_string().into(),
                            }
                            .into()
                        })
                    })
                })
                .map(|()| root)
        })
    }
    fn extract_tar(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        File::open(self.0)
            .map_err(Into::into)
            .and_then(|file| extract_tar_reader(file, destination))
    }
    fn extract_tar_gzip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        File::open(self.0)
            .map(GzDecoder::new)
            .map_err(Into::into)
            .and_then(|decoder| extract_tar_reader(decoder, destination))
    }
    fn extract_tar_zstd(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        File::open(self.0)
            .map_err(Into::into)
            .and_then(|file| ZstdDecoder::new(file).map_err(Into::into))
            .and_then(|decoder| extract_tar_reader(decoder, destination))
    }
    fn extract_zip(self, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        extraction_root(destination).and_then(|root| {
            File::open(&self.0)
                .map_err(Into::into)
                .and_then(|file| ZipArchive::new(file).map_err(Into::into))
                .and_then(|mut archive| match archive.len() > MAX_ARCHIVE_ENTRIES {
                    | true => Err(ArchiveError::TooManyEntries.into()),
                    | false => (0..archive.len()).try_fold((Vec::new(), 0_u64), |(seen, total), index| {
                        archive.by_index(index).map_err(Into::into).and_then(|mut entry| {
                            entry
                                .enclosed_name()
                                .ok_or_else(|| ArchiveError::UnsafeZipPath(entry.name().into()).into())
                                .and_then(|path| SafePath::new(path).map_err(Into::into))
                                .and_then(|relative| validate_entry(&root, seen, relative, entry.size()))
                                .and_then(|(seen, target)| {
                                    total
                                        .checked_add(entry.size())
                                        .ok_or_else(|| ArchiveError::ExpandedSizeOverflow.into())
                                        .and_then(|total| match total > MAX_ARCHIVE_EXPANDED_BYTES {
                                            | true => Err(ArchiveError::ExpandedSizeLimit.into()),
                                            | false => Ok((seen, target, total)),
                                        })
                                })
                                .and_then(|(seen, target, total)| {
                                    let mode = entry.unix_mode().unwrap_or_default() & 0o170000;
                                    match mode == 0o120000 {
                                        | true => Err(ArchiveError::ZipLink(target).into()),
                                        | false => {
                                            let is_directory = entry.is_dir();
                                            write_entry(&mut entry, &target, is_directory).map(|()| (seen, total))
                                        }
                                    }
                                })
                        })
                    }),
                })
                .map(|_| root)
        })
    }
}
impl From<&Path> for ArchiveCandidate {
    fn from(value: &Path) -> Self {
        Self(value.to_path_buf())
    }
}
impl From<PathBuf> for ArchiveCandidate {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}
impl ArchiveEntryExt for ArchiveEntry {
    fn extract(&self, reader: &mut dyn Read, root: &Path) -> Result<bool, sevenz_rust2::Error> {
        SafePath::new(self.name())
            .map_err(Report::from)
            .and_then(|relative| relative.materialize_under(root).map(|target| (relative, target)).map_err(Into::into))
            .and_then(|(relative, target)| write_entry(reader, &target, self.is_directory()).map(|()| relative))
            .map(|_| true)
            .map_err(|why| sevenz_rust2::Error::Other(why.to_string().into()))
    }
    fn validate(&self, seen: Vec<SafePath>) -> ApiResult<Vec<SafePath>> {
        SafePath::new(self.name())
            .map_err(Into::into)
            .and_then(|relative| match seen.contains(&relative) {
                | true => Err(ArchiveError::DuplicatePath(relative.into_path_buf()).into()),
                | false => Ok(seen.into_iter().chain(once(relative)).collect()),
            })
    }
}
impl core::error::Error for ArchiveError {}
impl fmt::Display for ArchiveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::DestinationInsideSource => write!(formatter, "Archive destination cannot be inside its source directory"),
            | Self::DestinationInvalid => write!(formatter, "Archive destination is not a regular directory"),
            | Self::DestinationMissingFilename => write!(formatter, "Archive destination must name a file"),
            | Self::DuplicatePath(path) => write!(formatter, "Duplicate archive path: {}", path.display()),
            | Self::EntryExpandedSizeLimit => write!(formatter, "Archive entry expands beyond the supported size limit"),
            | Self::ExpandedSizeLimit => write!(formatter, "Archive expands beyond the supported size limit"),
            | Self::ExpandedSizeOverflow => write!(formatter, "Archive expanded size overflow"),
            | Self::FormatInference(path) => write!(formatter, "Unable to infer archive format for {}", path.display()),
            | Self::InspectArchive { path, reason } => write!(formatter, "Failed to inspect archive {} — {reason}", path.display()),
            | Self::InspectSourceChild { path, reason } => {
                write!(formatter, "Failed to inspect archive source child {} — {reason}", path.display())
            }
            | Self::OutputInvalid => write!(formatter, "Archive destination exists and is not a regular file"),
            | Self::ReadArchive { path, reason } => write!(formatter, "Failed to read archive {} — {reason}", path.display()),
            | Self::ReplaceDestination(reason) => write!(formatter, "Failed to replace archive destination — {reason}"),
            | Self::ResolveSource(reason) => write!(formatter, "Failed to resolve archive source — {reason}"),
            | Self::SevenZip { operation, reason } => write!(formatter, "Failed to {operation} — {reason}"),
            | Self::SourceLink(path) => write!(formatter, "Archive source links are not supported: {}", path.display()),
            | Self::TooManyEntries => write!(formatter, "Archive contains too many entries"),
            | Self::UnsafeZipPath(path) => write!(formatter, "Unsafe ZIP path: {path}"),
            | Self::UnsupportedFormat(format) => write!(formatter, "Unsupported archive format: {format}"),
            | Self::UnsupportedTarEntry => write!(formatter, "Unsupported TAR entry"),
            | Self::ZipLink(path) => write!(formatter, "ZIP links are not supported: {}", path.display()),
        }
    }
}
impl DeterministicZipEntry {
    /// Create a deterministic ZIP with fixed metadata and explicit safe entry names.
    pub fn create(path: &Path, entries: &[Self]) -> ApiResult<PathBuf> {
        let unique = entries.iter().map(|entry| entry.destination.as_path()).collect::<BTreeSet<_>>();
        match (unique.len() == entries.len(), entries.iter().all(|entry| entry.mode & !0o777 == 0)) {
            | (false, _) => Err(eyre!("Deterministic ZIP contains duplicate entry names")),
            | (_, false) => Err(eyre!("Deterministic ZIP entry mode must contain permission bits only")),
            | (true, true) => {
                let mut entries = entries.iter().collect::<Vec<_>>();
                entries.sort_by(|left, right| left.destination.cmp(&right.destination));
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .map_err(Report::from)
                    .and_then(|file| entries.into_iter().try_fold(ZipWriter::new(file), |archive, entry| entry.write(archive)))
                    .and_then(|archive| archive.finish().map_err(Report::from))
                    .map(|_| path.to_path_buf())
            }
        }
    }
    /// Create an entry from in-memory content.
    pub fn from_bytes(destination: impl Into<PathBuf>, content: Vec<u8>, mode: u32) -> ApiResult<Self> {
        SafePath::new(destination.into())
            .map(|destination| Self {
                destination,
                mode,
                source: ZipEntrySource::Bytes(content),
            })
            .map_err(Into::into)
    }
    /// Create an entry from a regular file.
    pub fn from_file(destination: impl Into<PathBuf>, source: PathBuf, mode: u32) -> ApiResult<Self> {
        source
            .symlink_metadata()
            .map_err(Into::into)
            .and_then(|metadata| match metadata.file_type().is_symlink() || !metadata.is_file() {
                | true => Err(eyre!("ZIP entry source must be a regular file: {}", source.display())),
                | false => SafePath::new(destination.into())
                    .map(|destination| Self {
                        destination,
                        mode,
                        source: ZipEntrySource::File(source),
                    })
                    .map_err(Into::into),
            })
    }
    fn write(&self, mut archive: ZipWriter<File>) -> ApiResult<ZipWriter<File>> {
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .last_modified_time(zip::DateTime::default())
            .unix_permissions(self.mode);
        archive
            .start_file_from_path(self.destination.as_path(), options)
            .map_err(Report::from)
            .and_then(|()| match &self.source {
                | ZipEntrySource::Bytes(content) => archive.write_all(content).map(|()| archive).map_err(Report::from),
                | ZipEntrySource::File(path) => File::open(path)
                    .map_err(Report::from)
                    .and_then(|mut file| io::copy(&mut file, &mut archive).map_err(Report::from))
                    .map(|_| archive),
            })
    }
}
impl ArchiveFormat for MimeType {
    fn archive(&self, source: ArchiveCandidate, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        match self {
            | Self::Gzip => source.archive_tar_gzip(destination),
            | Self::SevenZip => source.archive_7z(destination),
            | Self::Tar => source.archive_tar(destination),
            | Self::Zip => source.archive_zip(destination),
            | _ => Err(ArchiveError::UnsupportedFormat(self.clone()).into()),
        }
    }
    fn extract(&self, source: ArchiveCandidate, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
        match self {
            | Self::Gzip => source.extract_tar_gzip(destination),
            | Self::SevenZip => source.extract_7z(destination),
            | Self::Tar => source.extract_tar(destination),
            | Self::Zip => source.extract_zip(destination),
            | Self::Zstd => source.extract_tar_zstd(destination),
            | _ => Err(ArchiveError::UnsupportedFormat(self.clone()).into()),
        }
    }
}
/// Creates an archive using the selected MIME type.
pub fn archive(source: PathBuf, destination: Option<PathBuf>, mime_type: MimeType) -> ApiResult<PathBuf> {
    ArchiveCandidate::from(source).archive(destination, mime_type)
}
/// Extracts an archive, inferring its format from a bounded content header when no override is supplied.
pub fn extract(source: PathBuf, destination: Option<PathBuf>, archive_format: Option<MimeType>) -> ApiResult<PathBuf> {
    ArchiveCandidate::from(source).extract(destination, archive_format)
}
fn extract_tar_reader<R: Read>(reader: R, destination: Option<PathBuf>) -> ApiResult<PathBuf> {
    extraction_root(destination).and_then(|root| {
        TarArchive::new(reader)
            .entries()
            .map_err(Into::into)
            .and_then(|entries| {
                entries
                    .enumerate()
                    .try_fold((Vec::new(), 0_u64), |(seen, total), (index, entry)| match index >= MAX_ARCHIVE_ENTRIES {
                        | true => Err(ArchiveError::TooManyEntries.into()),
                        | false => entry.map_err(Into::into).and_then(|mut entry| {
                            entry
                                .path()
                                .map_err(Into::into)
                                .and_then(|path| {
                                    let is_directory = entry.header().entry_type().is_dir();
                                    let is_file = entry.header().entry_type().is_file();
                                    let root_entry = path.components().all(|component| matches!(component, Component::CurDir));
                                    match (root_entry, is_directory, is_file) {
                                        | (true, true, _) => Ok((seen, root.clone(), true, total)),
                                        | (false, true, _) | (false, _, true) => SafePath::new(path.as_ref())
                                            .map_err(Into::into)
                                            .and_then(|relative| validate_entry(&root, seen, relative, entry.size()))
                                            .and_then(|(seen, target)| {
                                                total
                                                    .checked_add(entry.size())
                                                    .ok_or_else(|| ArchiveError::ExpandedSizeOverflow.into())
                                                    .and_then(|total| match total > MAX_ARCHIVE_EXPANDED_BYTES {
                                                        | true => Err(ArchiveError::ExpandedSizeLimit.into()),
                                                        | false => Ok((seen, target, is_directory, total)),
                                                    })
                                            }),
                                        | _ => Err(ArchiveError::UnsupportedTarEntry.into()),
                                    }
                                })
                                .and_then(|(seen, target, is_directory, total)| {
                                    write_entry(&mut entry, &target, is_directory).map(|()| (seen, total))
                                })
                        }),
                    })
            })
            .map(|_| root)
    })
}
fn extraction_root(destination: Option<PathBuf>) -> ApiResult<PathBuf> {
    let root = destination.unwrap_or_else(|| standard_project_folder("extract", None));
    match root.symlink_metadata() {
        | Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => Err(ArchiveError::DestinationInvalid.into()),
        | Ok(_) => Ok(root),
        | Err(why) if why.kind() == ErrorKind::NotFound => create_dir_all(&root).map(|()| root).map_err(Into::into),
        | Err(why) => Err(why.into()),
    }
}
fn prepare_archive_destination(path: &Path) -> ApiResult<File> {
    match path.symlink_metadata() {
        | Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => Err(ArchiveError::OutputInvalid.into()),
        | Ok(_) => remove_file(path).map_err(|why| ArchiveError::ReplaceDestination(why.to_string().into()).into()),
        | Err(why) if why.kind() == ErrorKind::NotFound => Ok(()),
        | Err(why) => Err(why.into()),
    }
    .and_then(|()| OpenOptions::new().write(true).create_new(true).open(path).map_err(Into::into))
}
fn validate_entry(root: &Path, seen: Vec<SafePath>, relative: SafePath, size: u64) -> ApiResult<(Vec<SafePath>, PathBuf)> {
    match size > MAX_ARCHIVE_EXPANDED_BYTES {
        | true => Err(ArchiveError::EntryExpandedSizeLimit.into()),
        | false if seen.contains(&relative) => Err(ArchiveError::DuplicatePath(relative.into_path_buf()).into()),
        | false => relative
            .materialize_under(root)
            .map(|target| (seen.into_iter().chain(once(relative)).collect(), target))
            .map_err(Into::into),
    }
}
fn write_entry(reader: &mut dyn Read, target: &Path, is_directory: bool) -> ApiResult<()> {
    match is_directory {
        | true => create_dir_all(target).map_err(Into::into),
        | false => target
            .parent()
            .map_or(Ok(()), |parent| create_dir_all(parent).map_err(Into::into))
            .and_then(|()| OpenOptions::new().write(true).create_new(true).open(target).map_err(Into::into))
            .and_then(|mut output| io::copy(reader, &mut output).map(|_| ()).map_err(Into::into)),
    }
}
/// Create an in-memory ZIP archive from validated relative entry names and byte content
pub fn zip_entries(entries: &[(&str, &[u8])]) -> ApiResult<Vec<u8>> {
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    entries
        .iter()
        .try_fold(
            (ZipWriter::new(Cursor::new(Vec::new())), Vec::<SafePath>::new()),
            |(mut archive, seen), (name, content)| {
                SafePath::new(name)
                    .map_err(|_| Report::from(ArchiveError::UnsafeZipPath((*name).into())))
                    .and_then(|relative| match seen.contains(&relative) {
                        | true => Err(ArchiveError::DuplicatePath(relative.into_path_buf()).into()),
                        | false => archive
                            .start_file(relative.as_path().to_string_lossy(), options)
                            .and_then(|()| archive.write_all(content).map_err(Into::into))
                            .map(|()| (archive, seen.into_iter().chain(once(relative)).collect()))
                            .map_err(Into::into),
                    })
            },
        )
        .and_then(|(archive, _)| archive.finish().map_err(Into::into))
        .map(Cursor::into_inner)
}
