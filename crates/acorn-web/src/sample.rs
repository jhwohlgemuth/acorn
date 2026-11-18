//! Bundled fictional research bucket with an NSSD-style directory layout.
use acorn_web::session::{SuppliedFile, Workspace};

const FILES: [(&str, &str); 40] = [
    (".gitlab-ci.yml", include_str!("../assets/examples/bucket/.gitlab-ci.yml")),
    (
        "amaterasu-mirror-garden/00.svg",
        include_str!("../assets/examples/bucket/amaterasu-mirror-garden/00.svg"),
    ),
    (
        "amaterasu-mirror-garden/01.svg",
        include_str!("../assets/examples/bucket/amaterasu-mirror-garden/01.svg"),
    ),
    (
        "amaterasu-mirror-garden/index.json",
        include_str!("../assets/examples/bucket/amaterasu-mirror-garden/index.json"),
    ),
    (
        "bamboo-shoots-after-rain/00.svg",
        include_str!("../assets/examples/bucket/bamboo-shoots-after-rain/00.svg"),
    ),
    (
        "bamboo-shoots-after-rain/index.md",
        include_str!("../assets/examples/bucket/bamboo-shoots-after-rain/index.md"),
    ),
    (
        "chang-e-moon-ledger/00.svg",
        include_str!("../assets/examples/bucket/chang-e-moon-ledger/00.svg"),
    ),
    (
        "chang-e-moon-ledger/01.svg",
        include_str!("../assets/examples/bucket/chang-e-moon-ledger/01.svg"),
    ),
    (
        "chang-e-moon-ledger/index.json",
        include_str!("../assets/examples/bucket/chang-e-moon-ledger/index.json"),
    ),
    (
        "dragon-gate-sketches/0.svg",
        include_str!("../assets/examples/bucket/dragon-gate-sketches/0.svg"),
    ),
    (
        "dragon-gate-sketches/1.svg",
        include_str!("../assets/examples/bucket/dragon-gate-sketches/1.svg"),
    ),
    (
        "dragon-gate-sketches/2.svg",
        include_str!("../assets/examples/bucket/dragon-gate-sketches/2.svg"),
    ),
    (
        "inari-foxfire-trails/00.svg",
        include_str!("../assets/examples/bucket/inari-foxfire-trails/00.svg"),
    ),
    (
        "inari-foxfire-trails/01.svg",
        include_str!("../assets/examples/bucket/inari-foxfire-trails/01.svg"),
    ),
    (
        "inari-foxfire-trails/index.json",
        include_str!("../assets/examples/bucket/inari-foxfire-trails/index.json"),
    ),
    (
        "jade-hare-observatory/00.svg",
        include_str!("../assets/examples/bucket/jade-hare-observatory/00.svg"),
    ),
    (
        "jade-hare-observatory/01.svg",
        include_str!("../assets/examples/bucket/jade-hare-observatory/01.svg"),
    ),
    (
        "jade-hare-observatory/index.json",
        include_str!("../assets/examples/bucket/jade-hare-observatory/index.json"),
    ),
    (
        "jade-river-crossing/index.json",
        include_str!("../assets/examples/bucket/jade-river-crossing/index.json"),
    ),
    (
        "kaguya-bamboo-signal/00.svg",
        include_str!("../assets/examples/bucket/kaguya-bamboo-signal/00.svg"),
    ),
    (
        "kaguya-bamboo-signal/index.json",
        include_str!("../assets/examples/bucket/kaguya-bamboo-signal/index.json"),
    ),
    (
        "kitsune-lantern-atlas/00.svg",
        include_str!("../assets/examples/bucket/kitsune-lantern-atlas/00.svg"),
    ),
    (
        "kitsune-lantern-atlas/index.yaml",
        include_str!("../assets/examples/bucket/kitsune-lantern-atlas/index.yaml"),
    ),
    (
        "moonlit-bamboo-grove/00.svg",
        include_str!("../assets/examples/bucket/moonlit-bamboo-grove/00.svg"),
    ),
    (
        "moonlit-bamboo-grove/01.svg",
        include_str!("../assets/examples/bucket/moonlit-bamboo-grove/01.svg"),
    ),
    (
        "moonlit-bamboo-grove/index.json",
        include_str!("../assets/examples/bucket/moonlit-bamboo-grove/index.json"),
    ),
    (
        "nuwa-stone-bridges/00.svg",
        include_str!("../assets/examples/bucket/nuwa-stone-bridges/00.svg"),
    ),
    (
        "nuwa-stone-bridges/01.svg",
        include_str!("../assets/examples/bucket/nuwa-stone-bridges/01.svg"),
    ),
    (
        "nuwa-stone-bridges/index.json",
        include_str!("../assets/examples/bucket/nuwa-stone-bridges/index.json"),
    ),
    ("README.md", include_str!("../assets/examples/bucket/README.md")),
    (
        "seven-rivers-one-sea/00.svg",
        include_str!("../assets/examples/bucket/seven-rivers-one-sea/00.svg"),
    ),
    (
        "seven-rivers-one-sea/01.svg",
        include_str!("../assets/examples/bucket/seven-rivers-one-sea/01.svg"),
    ),
    (
        "seven-rivers-one-sea/index.json",
        include_str!("../assets/examples/bucket/seven-rivers-one-sea/index.json"),
    ),
    (
        "susanoo-storm-loom/00.svg",
        include_str!("../assets/examples/bucket/susanoo-storm-loom/00.svg"),
    ),
    (
        "susanoo-storm-loom/01.svg",
        include_str!("../assets/examples/bucket/susanoo-storm-loom/01.svg"),
    ),
    (
        "susanoo-storm-loom/index.md",
        include_str!("../assets/examples/bucket/susanoo-storm-loom/index.md"),
    ),
    (
        "thousand-mile-wayfinder/00.svg",
        include_str!("../assets/examples/bucket/thousand-mile-wayfinder/00.svg"),
    ),
    (
        "thousand-mile-wayfinder/index.json",
        include_str!("../assets/examples/bucket/thousand-mile-wayfinder/index.json"),
    ),
    (
        "torii-wind-compass/00.svg",
        include_str!("../assets/examples/bucket/torii-wind-compass/00.svg"),
    ),
    (
        "torii-wind-compass/index.json",
        include_str!("../assets/examples/bucket/torii-wind-compass/index.json"),
    ),
];

pub(crate) fn files() -> Vec<SuppliedFile> {
    FILES
        .iter()
        .map(|(path, content)| SuppliedFile {
            path: (*path).into(),
            content: (*content).into(),
        })
        .collect()
}
pub(crate) fn is_loaded(workspace: &Workspace) -> bool {
    workspace.source.is_none()
        && workspace.files.len() == FILES.len()
        && workspace.files.iter().all(|file| FILES.iter().any(|(path, _)| file.path == *path))
}
#[cfg(test)]
mod tests {
    use super::*;
    use acorn_schema::pid::{PersistentIdentifierParse, ORCID, ROR};
    use acorn_schema::research_activity::ResearchActivity;
    use acorn_schema::Validate;
    use acorn_web::session::Selection;

    #[test]
    fn test_sample_has_fourteen_projects_with_multiple_identifiers() {
        let records = FILES
            .iter()
            .filter(|(path, _)| path.ends_with("/index.json") || path.ends_with("/index.yaml") || path.ends_with("/index.md"))
            .collect::<Vec<_>>();
        assert_eq!(records.len(), 14);
        assert_eq!(records.iter().filter(|(path, _)| path.ends_with("/index.json")).count(), 11);
        assert!(ORCID::is_valid("0000-0000-0000-0001"));
        assert!(ROR::is_valid("000000001"));
        for (path, content) in records {
            let slug = path.split('/').next().expect("project folder");
            assert!(content.contains(&format!("10.5072/acorn-sample-{slug}")), "missing DOI in {path}");
            assert!(content.contains("0000-0000-0000-0001"), "missing ORCID in {path}");
            assert!(content.contains("000000001"), "missing ROR in {path}");
            if path.ends_with(".json") {
                let value: serde_json::Value = serde_json::from_str(content).expect("valid JSON sample");
                assert_eq!(
                    value.pointer("/meta/identifier").and_then(serde_json::Value::as_str),
                    Some(slug),
                    "project identity in {path}"
                );
                let activity: ResearchActivity = serde_json::from_str(content).expect("RAD sample must deserialize");
                assert!(activity.validate().is_ok(), "RAD sample must validate: {path}");
            }
        }
    }
    #[test]
    fn test_sample_images_support_multiple_slides_and_one_missing_example() {
        let mut multiple_images = 0;
        let mut missing_images = 0;
        for (path, content) in FILES.iter().filter(|(path, _)| path.ends_with("/index.json")) {
            let value: serde_json::Value = serde_json::from_str(content).expect("valid JSON sample");
            let media = value
                .pointer("/meta/media")
                .and_then(serde_json::Value::as_array)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let folder = path.split('/').next().expect("project folder");
            multiple_images += usize::from(media.len() > 1);
            missing_images += usize::from(media.is_empty());
            for image in media {
                let url = image.get("contentUrl").and_then(serde_json::Value::as_str).expect("image URL");
                let image_path = format!("{folder}/{url}");
                assert!(FILES.iter().any(|(file, _)| file == &image_path), "missing sample image: {image_path}");
            }
        }
        assert!(multiple_images >= 7, "expected carousels on most Portfolio projects");
        assert_eq!(missing_images, 1, "keep one project without an image");
    }
    #[test]
    fn test_sample_loaded_state_survives_restore_and_clears_on_replacement() {
        let loaded = Workspace::default().replace(Selection { files: files(), skipped: 0 });
        assert!(is_loaded(&loaded));
        let restored = serde_json::from_str::<Workspace>(&serde_json::to_string(&loaded).expect("serialize sample"))
            .expect("deserialize sample")
            .restored()
            .expect("restore sample");
        assert!(is_loaded(&restored));
        let other = loaded.replace(Selection {
            files: vec![SuppliedFile {
                path: "other.json".into(),
                content: "{}".into(),
            }],
            skipped: 0,
        });
        assert!(!is_loaded(&other));
    }
}
