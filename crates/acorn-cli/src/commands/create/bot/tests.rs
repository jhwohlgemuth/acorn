use super::*;

#[tokio::test]
async fn test_remote_container_rejects_non_docker_runtime_before_execution() {
    let remote = "ssh://builder".parse::<Remote>().unwrap();
    let result = run(
        "42",
        &None,
        &None,
        &Some(Executor::Podman),
        &None,
        &None,
        &None,
        &None,
        EventSource::Poll,
        &None,
        false,
        &None,
        &None,
        &Some(remote),
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("requires the docker runtime"));
}
#[test]
fn test_remote_container_targets_the_requested_docker_host() {
    let remote = "ssh://builder".parse::<Remote>().unwrap();
    let command = remote
        .docker_args(create_bot_args(
            "42",
            "acorn-bot-42",
            "acorn:latest",
            None,
            Some(10),
            None,
            None,
            EventSource::Poll,
            None,
            false,
            None,
            None,
        ))
        .into_iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(command
        .iter()
        .map(String::as_str)
        .take(4)
        .eq(["--host", "ssh://builder", "run", "--detach"]));
}
#[test]
fn test_webhook_container_is_reachable_durable_and_redacts_credentials() {
    temp_env::with_vars(
        [
            ("GITLAB_TOKEN", Some("outbound-secret")),
            ("GITLAB_WEBHOOK_TOKEN", Some("inbound-secret")),
            ("GITLAB_WEBHOOK_SIGNING_TOKEN", None),
        ],
        || {
            let args = create_bot_args(
                "42",
                "acorn-bot-42",
                "acorn:latest",
                Some("localhost:8080"),
                Some(10),
                None,
                None,
                EventSource::Webhook,
                Some("https://bot.example.org"),
                true,
                Some(Path::new("/etc/deploy/.acorn")),
                None,
            )
            .into_iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
            assert!(args.windows(2).any(|values| values == ["--publish", "8080:8080"]));
            assert!(args
                .windows(2)
                .any(|values| { values == ["--mount", "type=volume,source=acorn-bot-42-state,target=/var/lib/acorn",] }));
            assert!(args.windows(2).any(|values| values == ["--bind", "0.0.0.0:8080"]));
            assert!(args.windows(2).any(|values| values == ["--env", "GITLAB_TOKEN"]));
            assert!(args.windows(2).any(|values| values == ["--env", "GITLAB_WEBHOOK_TOKEN"]));
            assert!(args
                .windows(2)
                .any(|values| { values == ["--mount", "type=bind,source=/etc/deploy/.acorn,target=/etc/acorn/.acorn,readonly"] }));
            assert!(args.windows(2).any(|values| values == ["--config", "/etc/acorn/.acorn"]));
            assert!(!args.iter().any(|value| value.contains("secret")));
        },
    );
}
