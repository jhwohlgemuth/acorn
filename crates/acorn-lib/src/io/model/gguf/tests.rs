#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing, clippy::unwrap_used)]
use super::*;
use std::fs::write;

enum MetadataValue<'a> {
    String(&'a str),
    U32(u32),
    U64(u64),
}
fn gguf(metadata: &[(&str, MetadataValue<'_>)]) -> Vec<u8> {
    let mut bytes = b"GGUF".to_vec();
    bytes.extend(3u32.to_le_bytes());
    bytes.extend(1u64.to_le_bytes());
    bytes.extend((metadata.len() as u64).to_le_bytes());
    metadata.iter().for_each(|(key, value)| {
        bytes.extend((key.len() as u64).to_le_bytes());
        bytes.extend(key.as_bytes());
        match value {
            | MetadataValue::String(value) => {
                bytes.extend(8u32.to_le_bytes());
                bytes.extend((value.len() as u64).to_le_bytes());
                bytes.extend(value.as_bytes());
            }
            | MetadataValue::U32(value) => {
                bytes.extend(4u32.to_le_bytes());
                bytes.extend(value.to_le_bytes());
            }
            | MetadataValue::U64(value) => {
                bytes.extend(10u32.to_le_bytes());
                bytes.extend(value.to_le_bytes());
            }
        }
    });
    bytes.extend(6u64.to_le_bytes());
    bytes.extend(b"weight");
    bytes.extend(1u32.to_le_bytes());
    bytes.extend(1u64.to_le_bytes());
    bytes.extend(0u32.to_le_bytes());
    bytes.extend(0u64.to_le_bytes());
    let padding = (32 - bytes.len() % 32) % 32;
    bytes.extend(vec![0; padding + 4]);
    bytes
}
fn write_gguf(root: &Path, name: &str, metadata: &[(&str, MetadataValue<'_>)]) -> PathBuf {
    let path = root.join(name);
    write(&path, gguf(metadata)).unwrap();
    path
}

#[test]
fn test_context_length_accepts_u32_and_metadata_order() {
    let directory = tempfile::tempdir().unwrap();
    let path = write_gguf(
        directory.path(),
        "model.gguf",
        &[
            ("llama.context_length", MetadataValue::U32(32_768)),
            ("general.architecture", MetadataValue::String("llama")),
        ],
    );
    assert_eq!(context_length(&[path]).unwrap(), Some(32_768));
}
#[test]
fn test_context_length_accepts_u64() {
    let directory = tempfile::tempdir().unwrap();
    let path = write_gguf(
        directory.path(),
        "model.gguf",
        &[
            ("general.architecture", MetadataValue::String("qwen2")),
            ("qwen2.context_length", MetadataValue::U64(131_072)),
        ],
    );
    assert_eq!(context_length(&[path]).unwrap(), Some(131_072));
}
#[test]
fn test_context_length_ignores_auxiliary_and_adapter_files() {
    let directory = tempfile::tempdir().unwrap();
    let primary = write_gguf(
        directory.path(),
        "model.gguf",
        &[
            ("general.architecture", MetadataValue::String("llama")),
            ("llama.context_length", MetadataValue::U32(8_192)),
        ],
    );
    let projector = write_gguf(
        directory.path(),
        "mmproj-model.gguf",
        &[
            ("general.architecture", MetadataValue::String("clip")),
            ("clip.context_length", MetadataValue::U32(77)),
        ],
    );
    let adapter = write_gguf(
        directory.path(),
        "adapter.gguf",
        &[
            ("general.type", MetadataValue::String("adapter")),
            ("general.architecture", MetadataValue::String("llama")),
            ("llama.context_length", MetadataValue::U32(4_096)),
        ],
    );
    assert_eq!(context_length(&[primary, projector, adapter]).unwrap(), Some(8_192));
}
#[test]
fn test_context_length_rejects_conflicting_split_values() {
    let directory = tempfile::tempdir().unwrap();
    let first = write_gguf(
        directory.path(),
        "model-00001-of-00002.gguf",
        &[
            ("general.architecture", MetadataValue::String("llama")),
            ("llama.context_length", MetadataValue::U32(8_192)),
            ("split.no", MetadataValue::U32(0)),
            ("split.count", MetadataValue::U32(2)),
        ],
    );
    let second = write_gguf(
        directory.path(),
        "model-00002-of-00002.gguf",
        &[
            ("general.architecture", MetadataValue::String("llama")),
            ("llama.context_length", MetadataValue::U32(16_384)),
            ("split.no", MetadataValue::U32(1)),
            ("split.count", MetadataValue::U32(2)),
        ],
    );
    assert!(context_length(&[first, second]).is_err());
}
#[test]
fn test_context_length_returns_matching_split_value() {
    let directory = tempfile::tempdir().unwrap();
    let paths = [0u32, 1u32]
        .into_iter()
        .map(|index| {
            write_gguf(
                directory.path(),
                format!("model-{:05}-of-00002.gguf", index + 1).as_str(),
                &[
                    ("general.architecture", MetadataValue::String("llama")),
                    ("llama.context_length", MetadataValue::U32(8_192)),
                    ("split.no", MetadataValue::U32(index)),
                    ("split.count", MetadataValue::U32(2)),
                ],
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(context_length(&paths).unwrap(), Some(8_192));
}
#[test]
fn test_context_length_returns_none_without_matching_metadata() {
    let directory = tempfile::tempdir().unwrap();
    let path = write_gguf(directory.path(), "model.gguf", &[("other.context_length", MetadataValue::U32(8_192))]);
    assert_eq!(context_length(&[path]).unwrap(), None);
}
