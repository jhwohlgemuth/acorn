use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::io::api::{self, gitlab, Configuration};
use acorn::param;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};
use tracing::info;

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
    // gitlab_example().await;
    // orcid_example().await;
    raid_example().await;
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
    let group = "24758"; // Research Enablement
    let project = "16689"; // ACORN
    let options = gitlab::Options::from_env();
    let runners = gitlab::runners(options.clone()).await;
    match &runners {
        | Ok(ref values) if !values.is_empty() => {
            let runner_id = values[0].identifier.unwrap_or_default().to_string();
            let options = options.clone().with_identifier(runner_id);
            let runner = gitlab::runner(options).await;
            info!("Runner: {runner:#?}");
            println!("# of runners: {:#?}", values.len());
        }
        | Ok(_) => println!("Runner: Ok(None)"),
        | Err(ref why) => panic!("Runner: Err({why:#?})"),
    }
    let runner_metadata = gitlab::RunnerMetadata::init()
        .runner_type("group")
        .description("TEST RUNNER — DELETE ASAP")
        .tags(&["gpu", "foo"])
        .build();
    let options = options.with_identifier(group).with_runner(runner_metadata);
    let create = gitlab::create_runner(options.clone()).await;
    println!("Create Runner Response: {create:#?}");
    let groups = gitlab::groups(options.clone()).await.unwrap();
    info!("Groups: {groups:#?}");
    println!("# of groups: {}", groups.len());
    let options = options.with_identifier(project);
    println!("Languages: {:#?}", gitlab::language_use(options).await.unwrap().entries());
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
async fn raid_example() {
    let options = api::raid::Options::from_env();
    let response = api::raid::service_points(options.clone()).await.unwrap();
    info!("Service Points: {response:#?}");
    println!("# of service points: {}", response.len());

    let ornl_service_point_id = "20000033";
    let options = options.with_identifier(ornl_service_point_id);
    let response = api::raid::service_points(options.clone()).await.unwrap();
    println!("ORNL Service Point: {response:#?}");

    let options = api::raid::Options::from_env();
    let response = api::raid::records(options.clone()).await.unwrap();
    info!("RAiD Records: {response:#?}");
    println!("# of RAiD records: {}", response.len());

    let test_raid_id = "10.83962/fb5be317";
    let options = options.with_identifier(test_raid_id);
    let record = api::raid::records(options).await.unwrap();
    println!("RAiD Record: {record:#?}");
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
