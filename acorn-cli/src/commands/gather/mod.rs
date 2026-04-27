use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::io::api::{self, gitlab};
use acorn::param;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

pub async fn run(
    path: &Option<PathBuf>,
    filter: &Option<String>,
    ignore: &Option<String>,
    merge_request: &bool,
    _database_path: &Option<PathBuf>,
    verbose: &Verbosity,
    offline: bool,
) -> Result<(), Report> {
    let options = CommandOptions::init()
        .maybe_path(path.clone())
        .maybe_filter(filter.clone())
        .maybe_ignore(ignore.clone())
        .merge_request(*merge_request)
        .offline(offline)
        .quiet(verbose.is_silent())
        .build();
    preflight!(&options);
    println!("CiteAs API is healthy: {}", api::citeas::is_healthy().await);
    println!("ORCiD API is healthy: {}", api::orcid::is_healthy().await);
    println!("ROR API is healthy: {}", api::ror::is_healthy().await);
    // let langs = gitlab::languages().await?;
    // dbg!(langs);
    // gitlab_merge_request_note_example().await;
    gitlab_example().await;
    // orcid_example().await;
    // ror_example().await;
    Ok(())
}
#[allow(dead_code)]
async fn gitlab_merge_request_note_example() {
    let comment = r#"# :seedling: Test Comment
> Here is a quote from @o9w

Here is a list
- one
- two
- three

( o 0 )
    "#;
    let options = gitlab::Options::from_env().with_body(comment);
    let response = gitlab::merge_request_note(options).await;
    println!("Merge Request Note Response: {:#?}", response);
}
#[allow(dead_code)]
async fn gitlab_example() {
    let group = "24758";
    let options = gitlab::Options::from_env();
    let runners = gitlab::runners(options.clone()).await;
    println!("Runners: {:#?}", runners);
    match runners {
        | Ok(ref values) if !values.is_empty() => {
            let runner_id = values[0].identifier.to_string();
            let options = options.clone().with_identifier(runner_id);
            println!("Runner: {:#?}", gitlab::runner(options).await);
        }
        | Ok(_) => println!("Runner: Ok(None)"),
        | Err(ref why) => panic!("Runner: Err({why:#?})"),
    }
    let options = options.with_identifier(group);
    println!("Groups: {:#?}", gitlab::groups(options).await);
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
