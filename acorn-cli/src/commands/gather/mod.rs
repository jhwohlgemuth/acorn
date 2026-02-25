use acorn::io::api::{self, Endpoint, RemoteResource, Searchable};
use acorn::param;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

pub async fn run(_path: &Option<PathBuf>, _ignore: &Option<String>, _offline: &bool, _verbose: &Verbosity) -> Result<(), Report> {
    let countries = api::geonames::get_countries().await;
    println!("Countries: {:#?}", countries);
    println!("CiteAs API is healthy: {}", api::citeas::is_healthy().await);
    println!("ORCiD API is healthy: {}", api::orcid::is_healthy().await);
    println!("ROR API is healthy: {}", api::ror::is_healthy().await);
    // spdx_example(&INCLUDED_ENDPOINTS).await;
    // orcid_example().await;
    ror_example().await;
    Ok(())
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
    // ORCiD API examples
    let text = api::orcid::search(params).await;
    println!("ORCiD Search Response: {text:#?}");
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
#[allow(dead_code)]
async fn spdx_example(endpoints: &Vec<Endpoint>) {
    let text = match endpoints.find_by_name("spdx") {
        | Some(endpoint) => {
            let response = endpoint.invoke("licenses", None).await;
            endpoint.handle::<api::spdx::Licenses>(response)
        }
        | None => Err("No SPDX endpoint found".into()),
    };
    println!("Licenses: {text:#?}");
}
