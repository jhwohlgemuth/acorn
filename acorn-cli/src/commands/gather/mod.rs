use acorn::io::api::{self, Endpoint, EndpointSearch, Param, RemoteResource, ENDPOINTS};
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

pub async fn run(_path: &Option<PathBuf>, _ignore: &Option<String>, _offline: &bool, _verbose: &Verbosity) -> Result<(), Report> {
    let text = match ENDPOINTS.find_by_name("geonames") {
        | Some(endpoint) => {
            let response = endpoint.invoke("countries", None).await;
            endpoint
                .handle::<api::TextResponse>(response)
                .map(|text| api::geonames::parse_tsv(&text.to_string()))
        }
        | None => Err("No GeoNames endpoint found".into()),
    };
    println!("Countries: {text:#?}");
    spdx_example(&ENDPOINTS).await;
    orcid_example(&ENDPOINTS).await;
    ror_example(&ENDPOINTS).await;
    Ok(())
}
#[allow(dead_code)]
async fn orcid_example(endpoints: &Vec<Endpoint>) {
    // ORCiD API examples
    let orcid = endpoints.find_by_name("orcid");
    let text = match &orcid {
        | Some(endpoint) => {
            let data = vec![
                Param::of_type(api::ParamStyle::QueryPair)
                    .values(vec![
                        vec![Some("affiliation-org-name"), Some("Lyrasis")],
                        vec![Some("ror-org-id"), Some("\"https://ror.org/01qz5mb56\"")],
                    ])
                    .with_key("q"),
                Param::of_type(api::ParamStyle::FieldList)
                    .values(vec![vec![Some("family-name")]])
                    .with_key("fl"),
            ];
            let response = endpoint
                .invoke_with::<api::orcid::SearchField, api::orcid::OutputColumn>("search", Some(data))
                .await;
            endpoint.handle::<api::orcid::SearchResponse>(response)
        }
        | None => Err("No ORCiD endpoint found".into()),
    };
    println!("ORCiD Search Response: {text:#?}");
    let text = match &orcid {
        | Some(endpoint) => {
            let response = endpoint.invoke("status", None).await;
            endpoint.handle::<api::orcid::StatusResponse>(response)
        }
        | None => Err("No ORCiD endpoint found".into()),
    };
    println!("ORCiD Status: {text:#?}");
}
#[allow(dead_code)]
async fn ror_example(endpoints: &Vec<Endpoint>) {
    // ROR API examples
    let ror = endpoints.find_by_name("ror");
    let text = match &ror {
        | Some(endpoint) => {
            let response = endpoint.invoke("status", None).await;
            endpoint.handle::<api::TextResponse>(response)
        }
        | None => Err("No ROR endpoint found".into()),
    };
    println!("ROR Status: {text:#?}");
    let text = match &ror {
        | Some(endpoint) => {
            let data = vec![Param::of_type(api::ParamStyle::TemplateValue)
                .values(vec![vec![Some("01qz5mb56")]])
                .with_key("identifier")];
            let response = endpoint.invoke("record", Some(data)).await;
            endpoint.handle::<api::ror::SingleRecord>(response)
        }
        | None => Err("No ROR endpoint found".into()),
    };
    println!("ROR Record: {text:#?}");
    let text = match &ror {
        | Some(endpoint) => {
            let data = vec![
                Param::of_type(api::ParamStyle::FieldList)
                    .values(vec![vec![Some("Oak Ridge")]])
                    .with_key("query"),
                Param::of_type(api::ParamStyle::QueryPair)
                    .values(vec![vec![Some("status"), Some("inactive")]])
                    .with_key("filter"),
            ];
            let response = endpoint.invoke("search", Some(data)).await;
            endpoint.handle::<api::ror::SearchResponse>(response)
        }
        | None => Err("No ROR endpoint found".into()),
    };
    println!("ROR Search Results: {text:#?}");
}
#[allow(dead_code)]
async fn spdx_example(endpoints: &Vec<Endpoint>) {
    // Get SPDX license data
    let text = match endpoints.find_by_name("spdx") {
        | Some(endpoint) => {
            let response = endpoint.invoke("licenses", None).await;
            endpoint.handle::<api::spdx::Licenses>(response)
        }
        | None => Err("No SPDX endpoint found".into()),
    };
    println!("Licenses: {text:#?}");
}
