//! GGUF model selection and binary validation
use super::ContextLength;
use crate::io::{
    oci::{OciFile, OciFiles, OciResolution},
    ApiResult, PathExt,
};
use acorn_core::prelude::{HashSet, String, ToString, Vec};
use acorn_schema::oci::ModelLayerRole;
use alloc::collections::BTreeSet;
use color_eyre::eyre::eyre;
use std::{
    fs::{self, remove_file, File},
    io::{BufReader, Read},
    path::{Path, PathBuf},
};

pub(super) const MIME_TYPE: &str = "application/gguf";
#[derive(Clone, Debug)]
enum GgufValue {
    String(String),
    Unsigned(u64),
}
#[derive(Clone, Debug, Default)]
pub(in crate::io) struct GgufInfo {
    architecture: Option<String>,
    context_lengths: Vec<(String, u64)>,
    general_type: Option<String>,
    split_no: Option<u64>,
    split_count: Option<u64>,
}
struct GgufReader<R> {
    inner: R,
    position: u64,
    length: u64,
}
impl ContextLength for GgufInfo {
    type Output = Option<u64>;

    fn context_length(&self, (): ()) -> Self::Output {
        self.architecture.as_ref().and_then(|architecture| {
            let key = format!("{architecture}.context_length");
            self.context_lengths
                .iter()
                .find_map(|(candidate, value)| (candidate == &key).then_some(*value))
        })
    }
}
impl GgufInfo {
    pub(in crate::io) fn is_adapter(&self) -> bool {
        self.general_type.as_deref() == Some("adapter")
    }
    pub(in crate::io) fn load(path: PathBuf) -> ApiResult<Self> {
        fs::metadata(&path)
            .map_err(|why| eyre!("Failed to inspect GGUF '{}' — {why}", path.display()))
            .and_then(|metadata| {
                File::open(&path)
                    .map_err(|why| eyre!("Failed to open GGUF '{}' — {why}", path.display()))
                    .map(|file| (file, metadata.len()))
            })
            .and_then(|(file, length)| {
                let mut reader = GgufReader {
                    inner: BufReader::new(file),
                    position: 0,
                    length,
                };
                reader
                    .bytes(4)
                    .and_then(|magic| match magic.as_slice() {
                        | b"GGUF" => Ok(()),
                        | _ => Err(eyre!("Invalid GGUF magic in '{}'", path.display())),
                    })
                    .and_then(|()| reader.u32())
                    .and_then(|version| match version {
                        | 2 | 3 => Ok(()),
                        | _ => Err(eyre!("Unsupported GGUF container version {version} in '{}'", path.display())),
                    })
                    .and_then(|()| reader.u64())
                    .and_then(|tensor_count| {
                        reader.u64().and_then(|metadata_count| match (tensor_count, metadata_count) {
                            | (0, _) => Err(eyre!("GGUF '{}' contains no tensors", path.display())),
                            | (value, _) if value > 1_000_000 => Err(eyre!("GGUF tensor count exceeds the supported limit")),
                            | (_, value) if value > 1_000_000 => Err(eyre!("GGUF metadata count exceeds the supported limit")),
                            | _ => Ok((tensor_count, metadata_count)),
                        })
                    })
                    .and_then(|(tensor_count, metadata_count)| reader.inspect_metadata(metadata_count).map(|value| (tensor_count, value)))
                    .and_then(|(tensor_count, (info, alignment))| reader.inspect_tensors(tensor_count, alignment, length).map(|()| info))
            })
    }
    pub(in crate::io) fn validate_split_metadata(&self, path: &Path) -> ApiResult<()> {
        match (path.to_gguf_parts(), self.split_no, self.split_count) {
            | (Some((_, index, count)), Some(split_no), Some(split_count)) if split_no.saturating_add(1) == index && split_count == count => Ok(()),
            | (Some(_), _, _) => Err(eyre!("Split GGUF '{}' has missing or inconsistent split metadata", path.display())),
            | (None, Some(_), Some(count)) if count > 1 => Err(eyre!(
                "GGUF '{}' declares a split set but its filename is not a split shard",
                path.display()
            )),
            | _ => Ok(()),
        }
    }
}
impl<R: Read> GgufReader<R> {
    fn bytes(&mut self, length: usize) -> ApiResult<Vec<u8>> {
        let end = self
            .position
            .checked_add(length as u64)
            .filter(|end| *end <= self.length)
            .ok_or_else(|| eyre!("GGUF structure exceeds file bounds"));
        end.and_then(|end| {
            let mut bytes = vec![0; length];
            self.inner
                .read_exact(&mut bytes)
                .map_err(|why| eyre!("Failed to read GGUF structure — {why}"))
                .map(|()| {
                    self.position = end;
                    bytes
                })
        })
    }
    fn fixed<const N: usize>(&mut self) -> ApiResult<[u8; N]> {
        self.bytes(N)
            .and_then(|bytes| bytes.try_into().map_err(|_| eyre!("Failed to read fixed-width GGUF value")))
    }
    fn u8(&mut self) -> ApiResult<u8> {
        self.fixed::<1>().map(u8::from_le_bytes)
    }
    fn u16(&mut self) -> ApiResult<u16> {
        self.fixed::<2>().map(u16::from_le_bytes)
    }
    fn u32(&mut self) -> ApiResult<u32> {
        self.fixed::<4>().map(u32::from_le_bytes)
    }
    fn u64(&mut self) -> ApiResult<u64> {
        self.fixed::<8>().map(u64::from_le_bytes)
    }
    fn string(&mut self) -> ApiResult<String> {
        self.u64().and_then(|length| match length <= 16 * 1024 * 1024 {
            | false => Err(eyre!("GGUF string exceeds the supported metadata limit")),
            | true => usize::try_from(length)
                .map_err(|_| eyre!("GGUF string length is not supported on this platform"))
                .and_then(|length| self.bytes(length))
                .and_then(|bytes| String::from_utf8(bytes).map_err(|why| eyre!("GGUF string is not UTF-8 — {why}"))),
        })
    }
    fn value(&mut self, kind: u32, depth: u8) -> ApiResult<Option<GgufValue>> {
        match (kind, depth > 8) {
            | (_, true) => Err(eyre!("GGUF metadata arrays are nested too deeply")),
            | (0, _) => self.u8().map(|value| Some(GgufValue::Unsigned(u64::from(value)))),
            | (1, _) => self.u8().map(|_| None),
            | (2, _) => self.u16().map(|value| Some(GgufValue::Unsigned(u64::from(value)))),
            | (3, _) => self.u16().map(|_| None),
            | (4, _) => self.u32().map(|value| Some(GgufValue::Unsigned(u64::from(value)))),
            | (5 | 6, _) => self.u32().map(|_| None),
            | (7, _) => self.u8().and_then(|value| match value {
                | 0 | 1 => Ok(None),
                | _ => Err(eyre!("GGUF boolean metadata value is invalid")),
            }),
            | (8, _) => self.string().map(|value| Some(GgufValue::String(value))),
            | (9, _) => self
                .u32()
                .and_then(|element_type| {
                    self.u64().and_then(|length| match length <= 16_000_000 {
                        | false => Err(eyre!("GGUF metadata array exceeds the supported element limit")),
                        | true => (0..length).try_for_each(|_| self.value(element_type, depth.saturating_add(1)).map(|_| ())),
                    })
                })
                .map(|()| None),
            | (10, _) => self.u64().map(|value| Some(GgufValue::Unsigned(value))),
            | (11 | 12, _) => self.u64().map(|_| None),
            | _ => Err(eyre!("GGUF metadata contains unsupported value type {kind}")),
        }
    }
    fn check_tensor_bounds(&self, tensors: &[(u64, u32, u64)], alignment: u64, length: u64) -> ApiResult<()> {
        let padding = self
            .position
            .checked_rem(alignment)
            .and_then(|remainder| alignment.checked_sub(remainder))
            .and_then(|value| value.checked_rem(alignment))
            .ok_or_else(|| eyre!("GGUF alignment calculation failed"));
        let data_start = padding.and_then(|padding| self.position.checked_add(padding).ok_or_else(|| eyre!("GGUF data offset overflow")));
        data_start.and_then(|data_start| match data_start <= length {
            | false => Err(eyre!("GGUF tensor data starts beyond file bounds")),
            | true => tensors.iter().try_for_each(|(elements, kind, offset)| {
                let available = length.saturating_sub(data_start);
                tensor_size(*kind, *elements).and_then(|size| offset.checked_add(size)).map_or_else(
                    || {
                        (*offset < available)
                            .then_some(())
                            .ok_or_else(|| eyre!("GGUF tensor offset exceeds file bounds"))
                    },
                    |end| {
                        (end <= available)
                            .then_some(())
                            .ok_or_else(|| eyre!("GGUF tensor data exceeds file bounds"))
                    },
                )
            }),
        })
    }
    fn inspect_metadata(&mut self, count: u64) -> ApiResult<(GgufInfo, u64)> {
        (0..count).try_fold((GgufInfo::default(), 32u64), |(info, alignment), _| {
            self.string()
                .and_then(|key| self.u32().and_then(|kind| self.value(kind, 0).map(|value| (key, value))))
                .map(|(key, value)| match (key.as_str(), value) {
                    | ("general.architecture", Some(GgufValue::String(value))) => (
                        GgufInfo {
                            architecture: Some(value),
                            ..info
                        },
                        alignment,
                    ),
                    | ("general.type", Some(GgufValue::String(value))) => (
                        GgufInfo {
                            general_type: Some(value),
                            ..info
                        },
                        alignment,
                    ),
                    | ("split.no", Some(GgufValue::Unsigned(value))) => (
                        GgufInfo {
                            split_no: Some(value),
                            ..info
                        },
                        alignment,
                    ),
                    | ("split.count", Some(GgufValue::Unsigned(value))) => (
                        GgufInfo {
                            split_count: Some(value),
                            ..info
                        },
                        alignment,
                    ),
                    | ("general.alignment", Some(GgufValue::Unsigned(value))) => (info, value),
                    | (key, Some(GgufValue::Unsigned(value))) if key.ends_with(".context_length") => (
                        GgufInfo {
                            context_lengths: info.context_lengths.into_iter().chain([(key.to_string(), value)]).collect(),
                            ..info
                        },
                        alignment,
                    ),
                    | _ => (info, alignment),
                })
        })
    }
    fn inspect_tensors(&mut self, count: u64, alignment: u64, length: u64) -> ApiResult<()> {
        match alignment.is_power_of_two() && alignment <= 4096 {
            | false => Err(eyre!("GGUF alignment is invalid")),
            | true => (0..count)
                .try_fold(Vec::new(), |offsets, _| {
                    self.string()
                        .and_then(|_| self.u32())
                        .and_then(|dimensions| match dimensions {
                            | 1..=4 => (0..dimensions).try_fold(1u64, |elements, _| {
                                self.u64()
                                    .and_then(|dimension| elements.checked_mul(dimension).ok_or_else(|| eyre!("GGUF tensor dimensions overflow")))
                            }),
                            | _ => Err(eyre!("GGUF tensor dimension count {dimensions} is unsupported")),
                        })
                        .and_then(|elements| self.u32().map(|kind| (elements, kind)))
                        .and_then(|(elements, kind)| self.u64().map(|offset| (elements, kind, offset)))
                        .map(|tensor| offsets.into_iter().chain([tensor]).collect())
                })
                .and_then(|tensors| self.check_tensor_bounds(&tensors, alignment, length)),
        }
    }
}
pub(in crate::io) fn context_length(paths: &[PathBuf]) -> ApiResult<Option<u64>> {
    paths
        .iter()
        .filter(|path| path.is_gguf() && !path.is_auxiliary_gguf())
        .map(|path| GgufInfo::load(path.clone()).and_then(|info| info.validate_split_metadata(path).map(|()| info)))
        .collect::<ApiResult<Vec<_>>>()
        .and_then(|values| {
            let contexts = values
                .iter()
                .filter(|info| info.general_type.as_deref() != Some("adapter"))
                .filter_map(|info| info.context_length(()))
                .collect::<BTreeSet<_>>();
            match contexts.into_iter().collect::<Vec<_>>().as_slice() {
                | [] => Ok(None),
                | [value] => Ok(Some(*value)),
                | values => Err(eyre!(
                    "Selected GGUF files report conflicting context lengths ({})",
                    values.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
                )),
            }
        })
}
pub(in crate::io) fn select_staged_files(
    staging: &Path,
    resolution: &OciResolution,
    origins: &[(String, String, ModelLayerRole)],
) -> ApiResult<Vec<OciFile>> {
    origins
        .iter()
        .map(|(path, digest, role)| match Path::new(path).is_gguf() {
            | false => Err(eyre!("OCI model layer materialized non-GGUF file '{path}'")),
            | true => fs::metadata(staging.join(path))
                .map_err(|why| eyre!("Failed to inspect OCI model file '{path}' — {why}"))
                .map(|metadata| OciFile {
                    media_type: MIME_TYPE.to_string(),
                    digest: digest.clone(),
                    size: metadata.len(),
                    path: path.clone(),
                    installed_size: Some(metadata.len()),
                    layer_digest: digest.clone(),
                    role: *role,
                }),
        })
        .collect::<ApiResult<Vec<_>>>()
        .map(OciFiles)
        .and_then(|files| files.select_known(&resolution.filter, &resolution.ignore, &resolution.quantization))
        .and_then(|selected| {
            let selected_paths = selected.0.iter().map(|file| file.path.clone()).collect::<HashSet<_>>();
            origins
                .iter()
                .filter(|(path, _, _)| !selected_paths.contains(path.as_str()))
                .try_for_each(|(path, _, _)| {
                    remove_file(staging.join(path)).map_err(|why| eyre!("Failed to remove unselected OCI file '{path}' — {why}"))
                })
                .map(|()| selected)
        })
        .and_then(|selected| selected.validate_content(staging))
        .map(|selected| selected.0)
}
fn tensor_size(kind: u32, elements: u64) -> Option<u64> {
    let (block, bytes): (u64, u64) = match kind {
        | 0 => (1, 4),
        | 1 => (1, 2),
        | 2 => (32, 18),
        | 3 => (32, 20),
        | 6 => (32, 22),
        | 7 => (32, 24),
        | 8 => (32, 34),
        | 9 => (32, 40),
        | 10 => (256, 84),
        | 11 => (256, 110),
        | 12 => (256, 144),
        | 13 => (256, 176),
        | 14 => (256, 210),
        | 15 => (256, 292),
        | _ => return None,
    };
    elements
        .checked_add(block.saturating_sub(1))
        .and_then(|value| value.checked_div(block))
        .and_then(|blocks| blocks.checked_mul(bytes))
}
#[cfg(test)]
mod tests;
