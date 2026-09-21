use acorn_core::{Location, Scheme};
use acorn_host::{
    download::DownloadExpectation,
    fs::{file_checksum, file_uri_to_path, FsError, SafePath, TempDir},
    http::{AllowedAuthorities, HttpClient, HttpError, HttpPolicy},
    terminal::Label,
};
use renamed_command::{args, cmd, CommandSpec};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let location = Location::from("ssh://git@example.org:2222/group/project.git");
    assert_eq!(location.scheme(), Scheme::SSH);
    exercise_filesystem()
        .map_err(|error| Box::new(error) as Box<dyn Error>)
        .and_then(|()| exercise_http().map_err(|error| Box::new(error) as Box<dyn Error>))
        .map(|client| {
            compose_async_apis(&client);
            let arguments = args!["--version"];
            let _specification = CommandSpec::new("git").args(arguments);
            let _macro_result = cmd!("git" "--version");
            println!("{} downstream boundary compiled", Label::pass());
        })
}
fn compose_async_apis(client: &HttpClient) {
    let location = Location::from("https://example.org/data.json");
    let _source = acorn_host::source::read_text(client, &location);
    let _download = acorn_host::download::download_to(
        client,
        "https://example.org/archive.tar",
        "archive.tar",
        DownloadExpectation::new().with_maximum_bytes(1024),
    );
}
fn exercise_filesystem() -> Result<(), FsError> {
    TempDir::create("downstream-fixture").and_then(|temporary| {
        SafePath::new("nested/config.json")
            .and_then(|relative| relative.materialize_under(temporary.path()))
            .and_then(|_| file_uri_to_path("file:///tmp/acorn.json"))
            .and_then(|_| file_checksum(file!(), None))
            .map(|_| {
                let _config_result = acorn_host::config::read::<String>("config.json");
            })
    })
}
fn exercise_http() -> Result<HttpClient, HttpError> {
    AllowedAuthorities::parse(["example.org", "localhost:8080"])
        .and_then(|allowed| HttpClient::new(HttpPolicy::new().with_allowed_authorities(allowed)))
}
