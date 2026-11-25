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
        Path::new("bin").join(format!("{APPLICATION}{}", env::consts::EXE_SUFFIX))
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
