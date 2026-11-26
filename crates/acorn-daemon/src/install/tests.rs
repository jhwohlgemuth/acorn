use super::*;

#[test]
fn test_target_paths_reject_executable_without_parent() {
    let directories = ProjectDirs::from_path(PathBuf::from("project-root")).expect("fixture project directories");
    assert!(matches!(
        TargetPaths::from_directories(directories, Path::new("acornd")),
        Err(TargetPathError::MissingExecutableParent { .. })
    ));
}
#[test]
fn test_target_paths_resolve_without_creating_directories() {
    let directories = ProjectDirs::from_path(PathBuf::from("project-root")).expect("fixture project directories");
    let expected_cache = directories.cache_dir().join(DAEMON_NAMESPACE);
    let expected_config = directories.config_dir().join(DEFAULT_CONFIG_FILENAMES[1]);
    let expected_data = directories.data_local_dir().join(DAEMON_NAMESPACE);
    let targets = TargetPaths::from_directories(directories, Path::new("bin/acornd")).expect("resolve deterministic daemon targets");
    assert_eq!(
        targets.binary(),
        Path::new("bin").join(format!("{CLI_APPLICATION}{}", env::consts::EXE_SUFFIX))
    );
    assert_eq!(targets.cache(), expected_cache);
    assert_eq!(targets.config(), expected_config);
    assert_eq!(targets.data(), expected_data);
}

#[cfg(target_os = "linux")]
mod linux_tests {
    use super::super::linux::{managed_unit, protected_token, quoted, render};
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_token_preflight_rejects_empty_public_and_symlinked_files() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("token");
        fs::write(&path, "\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(protected_token(&path).is_err());
        fs::write(&path, "test-token\n").unwrap();
        assert!(protected_token(&path).is_ok());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(protected_token(&path).is_err());
        let link = directory.path().join("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(protected_token(&link).is_err());
    }
    #[test]
    fn test_unit_rendering_escapes_special_paths() {
        let source = render(Path::new("/tmp/acorn %/acornd"), Path::new("/tmp/config $HOME \"one\".jsonc")).unwrap();
        assert!(source.starts_with("# Managed by ACORN"));
        assert!(source.contains("ExecStart=\"/tmp/acorn %%/acornd\" run \"/tmp/config $$HOME \\\"one\\\".jsonc\""));
        assert!(source.contains("UMask=0077"));
        assert!(quoted(Path::new("/tmp/bad\nname")).is_err());
    }
    #[test]
    fn test_unmanaged_unit_is_preserved() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("acornd.service");
        assert!(!managed_unit(&path).unwrap());
        fs::write(&path, "[Service]\nExecStart=/other\n").unwrap();
        assert!(managed_unit(&path).is_err());
    }
}

#[cfg(target_os = "macos")]
mod macos_tests {
    use super::super::macos::{launchd_presence, private_regular, LaunchIdentity, PropertyList};
    use crate::util::constants::app::{APPLICATION, LABEL, ORGANIZATION, PROPERTY_LIST_PATH};
    use acorn_cmd::{args, cmd};
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::PathBuf;

    #[test]
    fn test_daemon_identity_constants_align() {
        assert_eq!(LABEL, format!("org.{ORGANIZATION}.{APPLICATION}"));
        assert_eq!(PROPERTY_LIST_PATH, format!("/Library/LaunchDaemons/{LABEL}.plist"));
    }
    #[test]
    fn test_launchd_presence_distinguishes_missing_from_failure() {
        assert!(launchd_presence(Some(0), b"").unwrap());
        assert!(!launchd_presence(Some(113), b"Could not find service").unwrap());
        assert!(launchd_presence(Some(1), b"permission denied").is_err());
    }
    #[test]
    fn test_managed_property_list_rejects_foreign_and_symlinked_files() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("acornd.plist");
        assert!(!PropertyList::at(&path).managed().unwrap());
        fs::write(&path, "<plist/>").unwrap();
        assert!(PropertyList::at(&path).managed().is_err());
        let link = directory.path().join("link.plist");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(PropertyList::at(&link).managed().is_err());
    }
    #[test]
    fn test_private_regular_rejects_public_symlink_and_wrong_owner() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("token");
        fs::write(&path, "secret").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let uid = fs::metadata(&path).unwrap().uid();
        assert!(private_regular(&path, uid).is_ok());
        assert!(private_regular(&path, uid.saturating_add(1)).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(private_regular(&path, uid).is_err());
        let link = directory.path().join("link");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(private_regular(&link, uid).is_err());
    }
    #[test]
    fn test_property_list_render_serializes_paths_and_passes_plutil() {
        let identity = LaunchIdentity {
            user: "test-user".to_string(),
            home: PathBuf::from("/Users/test-user"),
            binary: PathBuf::from("/tmp/acorn & <one>/acornd"),
            config: PathBuf::from("/tmp/config & <one>.jsonc"),
        };
        let source = PropertyList::render(&identity).unwrap();
        let parsed = plist::Value::from_reader_xml(source.as_bytes()).unwrap();
        let dictionary = parsed.as_dictionary().unwrap();
        assert_eq!(dictionary.get("Label").and_then(plist::Value::as_string), Some(LABEL));
        assert_eq!(dictionary.get("Umask").and_then(plist::Value::as_unsigned_integer), Some(63));
        assert!(source.contains("/tmp/acorn &amp; &lt;one&gt;/acornd"));
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("test.plist");
        fs::write(&path, source).unwrap();
        assert!(cmd!(status "/usr/bin/plutil", args!["-lint", &path]).unwrap().success());
        let invalid = LaunchIdentity {
            binary: PathBuf::from("/tmp/bad\npath"),
            ..identity
        };
        assert!(PropertyList::render(&invalid).is_err());
    }
}
