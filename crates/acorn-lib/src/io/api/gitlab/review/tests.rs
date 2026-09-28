use super::*;
#[test]
fn classifies_supported_and_skipped_inputs() {
    let deleted = MergeRequestDiff {
        old_path: "old.json".to_string(),
        new_path: "old.json".to_string(),
        new_file: false,
        renamed_file: false,
        deleted_file: true,
        generated_file: false,
        collapsed: false,
        too_large: false,
    };
    let unsupported = MergeRequestDiff {
        old_path: "main.rs".to_string(),
        new_path: "main.rs".to_string(),
        deleted_file: false,
        ..deleted.clone()
    };
    assert!(matches!(Input::from(&deleted), Input::Skipped { reason, .. } if reason == "deleted file"));
    assert!(matches!(Input::from(&unsupported), Input::Skipped { reason, .. } if reason == "unsupported file type"));
    assert!(Input::supported("metadata/CITATION.cff"));
    assert!(Input::supported("activity.JSONC"));
}
