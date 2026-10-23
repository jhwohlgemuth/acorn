use super::{ArtifactPolicy, InputLimits, InputReference, RoutedInputPolicy};
use std::fs;
use std::path::Path;

#[test]
fn test_artifact_policy_requires_write_authority_and_rejects_collisions() {
    let root = tempfile::tempdir().expect("root");
    let existing = root.path().join("existing.json");
    fs::write(&existing, "original").expect("existing");
    let denied = ArtifactPolicy::new(root.path(), false, false).expect("denied policy");
    assert!(denied.destination(Path::new("new.json"), &[]).is_err());
    let allowed = ArtifactPolicy::new(root.path(), true, false).expect("allowed policy");
    assert!(allowed.destination(Path::new("../outside.json"), &[]).is_err());
    assert!(allowed.destination(Path::new("existing.json"), &[]).is_err());
    let proposed = allowed.destination(Path::new("new.json"), &[existing]).expect("new destination");
    assert_eq!(proposed, root.path().canonicalize().expect("canonical root").join("new.json"));
    assert!(!proposed.exists());
}
#[test]
fn test_directory_batches_are_sorted_and_report_partial_failures() {
    let root = tempfile::tempdir().expect("root");
    let folder = root.path().join("batch");
    fs::create_dir(&folder).expect("batch");
    fs::write(folder.join("z.json"), "z").expect("z");
    fs::write(folder.join("b.json"), "bb").expect("b");
    fs::write(folder.join("a.json"), "a").expect("a");
    fs::create_dir(folder.join("nested")).expect("nested");
    fs::write(folder.join("nested").join("inner.json"), "x").expect("inner");
    let policy = RoutedInputPolicy::new(
        root.path(),
        InputLimits {
            max_files: 2,
            max_total: acorn_schema::hardware::Memory::from(4_u64),
            max_depth: 0,
        },
    )
    .expect("policy");
    let result = policy.resolve(&InputReference::Directory("batch".into())).expect("batch");
    let files = result
        .files
        .iter()
        .map(|item| item.path.file_name().expect("name").to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(files, ["a.json", "b.json"]);
    let failures = result.failures.iter().map(|failure| failure.reason.as_str()).collect::<Vec<_>>();
    assert_eq!(failures, ["depth-limit", "file-limit"]);
}
#[cfg(unix)]
#[test]
fn test_symlinks_cannot_escape_input_or_output_roots() {
    use std::os::unix::fs::symlink;

    let root = tempfile::tempdir().expect("root");
    let outside = tempfile::tempdir().expect("outside");
    fs::write(outside.path().join("secret.json"), "secret").expect("secret");
    symlink(outside.path().join("secret.json"), root.path().join("linked.json")).expect("input symlink");
    let input = RoutedInputPolicy::new(root.path(), InputLimits::default()).expect("input policy");
    assert!(input.resolve(&InputReference::File("linked.json".into())).is_err());
    let batch = input.resolve(&InputReference::Directory(".".into())).expect("root batch");
    assert_eq!(batch.failures.len(), 1);
    assert_eq!(batch.failures.first().expect("one failure").reason, "symlink");
    symlink(outside.path(), root.path().join("escape")).expect("destination symlink");
    let output = ArtifactPolicy::new(root.path(), true, false).expect("artifact policy");
    assert!(output.destination(Path::new("escape/created.json"), &[]).is_err());
    assert!(!outside.path().join("created.json").exists());
}
#[test]
fn test_text_and_file_inputs_are_bounded_and_contained() {
    let root = tempfile::tempdir().expect("root");
    fs::write(root.path().join("valid.json"), "{}\n").expect("file");
    let policy = RoutedInputPolicy::new(
        root.path(),
        InputLimits {
            max_files: 2,
            max_total: acorn_schema::hardware::Memory::from(4_u64),
            max_depth: 1,
        },
    )
    .expect("policy");
    let text = policy.resolve(&InputReference::Text("abc".to_string())).expect("text");
    assert_eq!(text.text.as_deref(), Some("abc"));
    assert!(policy.resolve(&InputReference::Text("12345".to_string())).is_err());
    assert!(policy.resolve(&InputReference::Text(String::new())).is_err());
    let file = policy.resolve(&InputReference::File("valid.json".into())).expect("file");
    assert_eq!(file.files.len(), 1);
    assert_eq!(file.files.first().expect("one file").size, acorn_schema::hardware::Memory::from(3_u64));
    assert!(policy.resolve(&InputReference::File("../outside.json".into())).is_err());
    assert!(policy.resolve(&InputReference::File("missing.json".into())).is_err());
}
