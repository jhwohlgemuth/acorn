use super::*;

fn target_ssh() -> ContainerTarget {
    ContainerTarget::Ssh("ssh://builder".parse().unwrap())
}
fn text(values: Vec<OsString>) -> Vec<String> {
    values.into_iter().map(|value| value.to_string_lossy().into_owned()).collect()
}

#[test]
fn test_alias_remote_parses() {
    let target = ContainerTarget::from_args(Some("my-alias".parse().unwrap()), None).unwrap();
    assert_eq!(target.to_string(), "ssh://my-alias");
    let target = ContainerTarget::from_args(Some("user@builder".parse().unwrap()), None).unwrap();
    assert_eq!(target.to_string(), "ssh://user@builder");
}
#[test]
fn test_context_target_uses_context_flag() {
    let target = ContainerTarget::from_args(None, Some("prod".to_string())).unwrap();
    let base = vec![OsString::from("ps")];
    let values = text(target.to_runtime_args(&Executor::Docker, base));
    assert_eq!(values, vec!["--context", "prod", "ps"]);
    let values = text(target.to_runtime_args(&Executor::Podman, vec![OsString::from("ps")]));
    assert_eq!(values, vec!["--connection", "prod", "ps"]);
}
#[test]
fn test_local_docker_runner_arguments_are_unchanged() {
    let values = text(build_docker_runner_args(
        "runner-42",
        "gitlab/gitlab-runner:latest",
        false,
        "/srv/gitlab-runner/config",
        DOCKER_SOCKET,
        None,
        true,
    ));
    assert_eq!(values.first().map(String::as_str), Some("run"));
    assert!(values
        .windows(2)
        .any(|pair| pair == ["-v", "/srv/gitlab-runner/config:/etc/gitlab-runner"]));
    assert!(!values.iter().any(|value| value == "--host"));
}
#[test]
fn test_remote_docker_runner_uses_remote_volume_and_socket() {
    let target = target_ssh();
    let base = build_docker_runner_args(
        "runner-42",
        "gitlab/gitlab-runner:latest",
        false,
        "C:\\local\\runner",
        DOCKER_SOCKET,
        Some(&target),
        true,
    );
    let values = text(target.to_runtime_args(&Executor::Docker, base));
    assert!(values
        .iter()
        .map(String::as_str)
        .take(4)
        .eq(["--host", "ssh://builder", "run", "--detach"]));
    let socket_mount = format!("{DOCKER_SOCKET}:{DOCKER_SOCKET}");
    assert!(values.windows(2).any(|pair| pair == ["-v", "runner-42-config:/etc/gitlab-runner"]));
    assert!(values.windows(2).any(|pair| pair == ["-v", socket_mount.as_str()]));
    assert!(!values.iter().any(|value| value.contains("C:\\local")));
}
#[test]
fn test_remote_runner_registration_targets_the_requested_docker_host() {
    let template = Some(GITLAB_RUNNER_GPU_TEMPLATE_PATH.to_string());
    let token = Secret::from("token".to_string());
    let base = build_register_args(
        "runner-42",
        "https://gitlab.example.org/",
        "GitLab Runner [42]",
        &token,
        "debug",
        &template,
    );
    let values = text(target_ssh().to_runtime_args(&Executor::Docker, base));
    assert!(values
        .iter()
        .map(String::as_str)
        .take(5)
        .eq(["--host", "ssh://builder", "exec", "runner-42", "gitlab-runner"]));
    assert!(values
        .windows(2)
        .any(|pair| pair == ["--template-config", GITLAB_RUNNER_GPU_TEMPLATE_PATH]));
}
#[test]
fn test_remote_runner_validation_rejects_non_docker_config_before_creation() {
    let runner = RunnerDetails::at(Repository::GitLab {
        id: Some(42),
        location: Box::new(Location::Simple("https://gitlab.com".to_string())),
    })
    .executor(Executor::Shell)
    .build();
    let result = Executor::Docker.validate_runner_target(Some(&[runner]), Some(&target_ssh()));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires docker/podman"));
}
#[test]
fn test_remote_runner_validation_rejects_non_docker_runtime() {
    let result = Executor::Shell.validate_runner_target(None, Some(&target_ssh()));
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires the docker or podman runtime"));
}
