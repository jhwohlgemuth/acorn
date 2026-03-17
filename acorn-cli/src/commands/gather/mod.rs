use acorn::io::api::{self};
use acorn::io::database::schema::Table;
use acorn::io::database::{Database, Operations};
use acorn::param;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

pub async fn run(
    _path: &Option<PathBuf>,
    _ignore: &Option<String>,
    _offline: &bool,
    database_path: &Option<PathBuf>,
    _verbose: &Verbosity,
) -> Result<(), Report> {
    let countries = api::geonames::countries().await;
    println!("Countries: {:#?}", countries);
    println!("CiteAs API is healthy: {}", api::citeas::is_healthy().await);
    println!("ORCiD API is healthy: {}", api::orcid::is_healthy().await);
    println!("ROR API is healthy: {}", api::ror::is_healthy().await);
    let licenses = api::spdx::download().await.ok();
    let database = Database::<Table>::from_path(database_path.clone());
    let inserted = database.persist(licenses).await;
    dbg!(&inserted);
    // gitlab_example().await;
    // orcid_example().await;
    // ror_example().await;
    Ok(())
}
#[allow(dead_code)]
async fn gitlab_example() {
    let _ = dotenvy::from_filename(".env");
    match std::env::var("GITLAB_TOKEN") {
        | Ok(token) => {
            let project = "34619";
            let group = "24758";
            println!("GitLab Token: {}", token);
            println!("Runners: {:#?}", api::gitlab::runners(&token).await);
            println!("Runner: {:#?}", api::gitlab::runner(project, &token).await);
            println!("Groups: {:#?}", api::gitlab::groups(group, &token).await);
        }
        | Err(_) => println!("GitLab Token not found in environment variables"),
    }
}
#[allow(dead_code)]
async fn orcid_example() {
    let params = vec![
        param!(
            QueryPair,
            "q",
            (("affiliation-org-name", "Lyrasis"), ("ror-org-id", "\"https://ror.org/01qz5mb56\""),)
        ),
        param!(FieldList, "fl", "family-name"),
    ];
    println!("ORCiD Search Response: {:#?}", api::orcid::search(params).await);
}
#[allow(dead_code)]
async fn ror_example() {
    let identifier = "01qz5mb56";
    println!("ROR Record: {:#?}", api::ror::record(identifier).await);
    let params = vec![
        param!(FieldList, "query", "Oak Ridge"),
        param!(QueryPair, "filter", ("status", "inactive")),
    ];
    println!("ROR Search Results: {:#?}", api::ror::search(params).await);
}
