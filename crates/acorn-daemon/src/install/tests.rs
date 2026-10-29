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
