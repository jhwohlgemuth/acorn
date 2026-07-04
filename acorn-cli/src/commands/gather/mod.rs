#![allow(clippy::panic)]
use crate::cli::CommandOptions;
use crate::commands::preflight;
use acorn::io::api::{self, gitlab, Configuration};
use acorn::param;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};
use tracing::{error, info};
use validator::Validate;

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
    // raid_example().await;
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
    let response = gitlab::merge_request_note(&options).await;
    println!("Merge Request Note Response: {:#?}", response);
}
#[allow(dead_code)]
async fn gitlab_example() {
    let group = "24758"; // Research Enablement
    let project = "16689"; // ACORN
    let options = gitlab::Options::from_env();
    let runners = gitlab::runners(&options).await;
    match &runners {
        | Ok(ref values) if !values.is_empty() => {
            let runner_id = values.first().map(|v| v.identifier.unwrap_or_default().to_string()).unwrap_or_default();
            let runner = gitlab::runner(&options.clone().with_identifier(runner_id)).await;
            info!("Runner: {runner:#?}");
            println!("# of runners: {:#?}", values.len());
        }
        | Ok(_) => println!("Runner: Ok(None)"),
        | Err(ref why) => error!("Runner: Err({why:#?})"),
    }

    let groups = gitlab::groups(&options.clone().with_identifier(group)).await.unwrap_or_default();
    info!("Groups: {groups:#?}");
    println!("# of groups: {}", groups.len());

    let options = &options.clone().with_identifier(project).with_params(vec![
        param!(KeyValuePair, "target_type", "note"),
        param!(KeyValuePair, "after", "2026-06-27"),
        param!(KeyValuePair, "before", "2026-##-27"),
    ]);
    let events = gitlab::events(options).await.unwrap_or_default();
    info!("Events: {events:#?}");
    println!("# of events: {}", events.len());

    // match gitlab::language_use(&options.with_identifier(project)).await {
    //     | Ok(response) => println!("Languages: {:#?}", response.entries()),
    //     | Err(why) => println!("Languages request failed: {why:#?}"),
    // }
}
#[allow(dead_code)]
async fn orcid_example() {
    let options = api::orcid::Options::from_env().with_params(vec![
        param!(
            QueryPair,
            "q",
            (("affiliation-org-name", "Lyrasis"), ("ror-org-id", "\"https://ror.org/01qz5mb56\""),)
        ),
        param!(FieldList, "fl", "family-name"),
    ]);
    println!("ORCiD Search Response: {:#?}", api::orcid::search(&options).await);
}
#[allow(dead_code)]
async fn raid_example() {
    // let options = api::raid::Options::from_env();
    // let response = api::raid::service_point(options.clone()).await.unwrap();
    // info!("Service Points: {response:#?}");
    // println!("# of service points: {}", response.len());

    // let ornl_service_point_id = "20000033";
    // let options = options.with_identifier(ornl_service_point_id);
    // let response = api::raid::service_point(options.clone()).await.unwrap();
    // println!("ORNL Service Point: {response:#?}");

    // let options = api::raid::Options::from_env();
    // let response = api::raid::record(options.clone()).await.unwrap();
    // info!("RAiD Records: {response:#?}");
    // println!("# of RAiD records: {}", response.len());

    // let test_raid_id = "10.83962/fb5be317";
    // let options = options.with_identifier(test_raid_id);
    // let record = api::raid::record(options).await.unwrap();
    // println!("RAiD Record: {record:#?}");

    use acorn::schema::pid::raid::{
        Access, AccessIdentifier, AccessType, Contributor, ContributorPosition, CreditRole, Description, DescriptionIdentifier, DescriptionType,
        Metadata, PositionType, Role, Title, TitleIdentifier, TitleType,
    };
    use acorn::schema::Date;

    let orcid = "https://sandbox.orcid.org/0000-0003-0021-3068";
    let access_type = AccessIdentifier::init().id(AccessType::OpenAccess).build();
    let description_type = DescriptionIdentifier::init().id(DescriptionType::Primary).build();
    let title_type = TitleIdentifier::init().id(TitleType::Primary).build();
    let position = ContributorPosition::init()
        .id(PositionType::PrincipalInvestigator)
        .date(Date::init().start_date("2025-08-28").build())
        .build();
    let role = Role::init().id(CreditRole::WritingReviewEditing).build();
    let access = Access::init().access_type(access_type).build();
    let contributor = Contributor::init()
        .id(orcid)
        .schema_uri("https://orcid.org/")
        .position(vec![position])
        .role(vec![role])
        .leader(true)
        .contact(true)
        .build();
    let date = Date::init().start_date("2024-01-01").build();
    let description = Description::init()
        .text("Super informative description text")
        .description_type(description_type)
        .build();
    let title = Title::init()
        .text("RAiD Record created from ACORN API")
        .title_type(title_type)
        .start_date("2026-05-05")
        .build();
    let metadata = Metadata::init()
        .access(access)
        .contributor(vec![contributor])
        .date(date)
        .description(vec![description])
        .title(vec![title])
        .build();
    match serde_json::to_string_pretty(&metadata) {
        | Ok(serialized) => info!("Metadata: {serialized}"),
        | Err(why) => info!("Metadata serialization failed: {why:#?}"),
    }
    println!("Validation: {:#?}", metadata.validate());
    // let options = api::raid::Options::from_env().with_metadata(metadata);
    // let response = api::raid::create_record(options).await;
    // match response {
    //     | Ok(record) => println!("Created RAiD Record: {record:#?}"),
    //     | Err(why) => println!("Failed to create RAiD record:\n{why}"),
    // }
}
#[allow(dead_code)]
async fn ror_example() {
    let identifier = "01qz5mb56";
    let record_options = api::ror::Options::from_env().with_identifier(identifier);
    println!("ROR Record: {:#?}", api::ror::record(&record_options).await);
    let search_options = api::ror::Options::from_env().with_params(vec![
        param!(FieldList, "query", "Oak Ridge"),
        param!(QueryPair, "filter", ("status", "inactive")),
    ]);
    println!("ROR Search Results: {:#?}", api::ror::search(&search_options).await);
}
