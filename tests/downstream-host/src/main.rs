use acorn_core::{Location, Scheme};
use acorn_host::{
    config::{read, resolve_config_path, ConfigError},
    download::DownloadExpectation,
    fs::{file_checksum, file_uri_to_path, project_cache_directory, FsError, SafePath, TempDir},
    http::{AllowedAuthorities, HttpClient, HttpError, HttpPolicy},
    source::SourcePolicy,
    terminal::Label,
};
use renamed_command::{args, cmd, CancellationToken, CaptureLimits, CommandSpec, ExecutionOptions};
use std::{error::Error, fs, path::Path};

fn compose_async_apis(client: &HttpClient) {
    let location = Location::from("https://example.org/data.json");
    let _source = acorn_host::source::read_text(client, &location, SourcePolicy::new());
    let _download = acorn_host::download::download_to(
        client,
        "https://example.org/archive.tar",
        "archive.tar",
        DownloadExpectation::new().with_maximum_bytes(1024),
    );
}
fn exercise_config(directory: &Path) -> Result<(), ConfigError> {
    let path = directory.join(".xylem.jsonc");
    fs::write(&path, "[// comment\n\"downstream\",]\n")
        .map_err(|source| ConfigError::Read { path: path.clone(), source })
        .and_then(|()| read::<Vec<String>>(&path))
        .map(|values| {
            assert_eq!(values, ["downstream"]);
            assert_eq!(
                resolve_config_path(None, directory, &["missing.json", ".xylem.jsonc"]),
                Some(path.clone())
            );
            let missing = Path::new("explicit-missing.jsonc");
            assert_eq!(
                resolve_config_path(Some(missing), directory, &[".xylem.jsonc"]),
                Some(missing.to_path_buf())
            );
        })
}
fn exercise_filesystem() -> Result<(), Box<dyn Error>> {
    TempDir::create("downstream-fixture")
        .map_err(|error: FsError| Box::new(error) as Box<dyn Error>)
        .and_then(|temporary| {
            SafePath::new("nested/config.json")
                .and_then(|relative| relative.materialize_under(temporary.path()))
                .and_then(|_| file_uri_to_path("file:///tmp/acorn.json"))
                .and_then(|_| file_checksum(file!(), None))
                .map(|_| {
                    let _project_cache_call =
                        || SafePath::new("runs").and_then(|namespace| project_cache_directory("org", "ornl", "xylem", &namespace));
                })
                .map_err(|error| Box::new(error) as Box<dyn Error>)
                .and_then(|()| exercise_config(temporary.path()).map_err(|error| Box::new(error) as Box<dyn Error>))
        })
}
fn exercise_http() -> Result<HttpClient, HttpError> {
    AllowedAuthorities::parse(["example.org", "localhost:8080"])
        .and_then(|allowed| HttpClient::new(HttpPolicy::new().with_allowed_authorities(allowed)))
}
fn main() -> Result<(), Box<dyn Error>> {
    let location = Location::from("ssh://git@example.org:2222/group/project.git");
    assert_eq!(location.scheme(), Scheme::SSH);
    exercise_filesystem()
        .and_then(|()| exercise_http().map_err(|error| Box::new(error) as Box<dyn Error>))
        .map(|client| {
            compose_async_apis(&client);
            let arguments = args!["--version"];
            let specification = CommandSpec::new("git").args(arguments);
            let cancellation = CancellationToken::new();
            let options = ExecutionOptions::new()
                .with_cancellation(cancellation)
                .with_capture_limits(CaptureLimits::unlimited());
            let _managed_call = || specification.execute(&options);
            let _managed_macro_call = || cmd!(exec "git", ["--version"]);
            let _macro_result = cmd!("git" "--version");
            println!("{} downstream boundary compiled", Label::pass());
        })
}
