use crate::config::ApplicationConfiguration;
use acorn::io::api::{self, EndpointSearch, Param, RestfulInterface};
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

const ENDPOINTS: &str = r#"{
    "endpoints": [
        {
            "name": "github",
            "domain": "api.github.com",
            "resources": [
                {
                    "name": "tree",
                    "method": "get",
                    "template": "{{ base }}/{{ path }}/git/trees/{{ branch }}?recursive=1"
                }
            ]
        },
        {
            "name": "gitlab",
            "domain": "code.ornl.gov",
            "root": "api/v4",
            "resources": [
                {
                    "name": "tree",
                    "method": "get",
                    "template": "{{ base }}/projects/{{ project_id }}//repository/tree/{{ query }}"
                }
            ]
        },
        {
            "name": "orcid",
            "domain": "pub.orcid.org",
            "root": "v3.0",
            "resources": [
                {
                    "name": "search",
                    "method": "get",
                    "template": "{{ base }}/expanded-search/{{ query }}"
                },
                {
                    "name": "status",
                    "method": "get",
                    "template": "{{ base }}/pubStatus"
                }
            ]
        },
        {
            "name": "ror",
            "domain": "api.ror.org",
            "root": "v2",
            "resources": [
                {
                    "name": "record",
                    "method": "get",
                    "template": "{{ base }}/organizations/{{ identifier }}"
                },
                {
                    "name": "search",
                    "method": "get",
                    "template": "{{ base }}/organizations/{{ query }}"
                },
                {
                    "name": "status",
                    "method": "get",
                    "template": "{{ base }}/heartbeat"
                }
            ]
        },
        {
            "name": "spdx",
            "domain": "spdx.org",
            "resources": [
                {
                    "name": "licenses",
                    "method": "get",
                    "template": "{{ base }}/licenses/licenses.json"
                }
            ]
        }
    ]
}"#;

pub fn run(_path: &Option<PathBuf>, _ignore: &Option<String>, _offline: &bool, _verbose: &Verbosity) -> Result<(), Report> {
    let endpoints = match ApplicationConfiguration::parse(ENDPOINTS).unwrap().endpoints {
        | Some(endpoints) => endpoints,
        | None => vec![],
    };
    // Get SPDX license data
    let text = match endpoints.find_by_name("spdx") {
        | Some(endpoint) => {
            let response = endpoint.invoke_sync("licenses", None);
            endpoint.handle::<api::spdx::Licenses>(response)
        }
        | None => Err("No SPDX endpoint found".into()),
    };
    println!("Licenses: {text:#?}");
    // ORCiD API examples
    let orcid = endpoints.find_by_name("orcid");
    let text = match &orcid {
        | Some(endpoint) => {
            let data = vec![
                Param::of_type(api::ParamStyle::QueryPair)
                    .values(vec![
                        (Some("affiliation-org-name"), Some("Lyrasis")),
                        (Some("ror-org-id"), Some("\"https://ror.org/01qz5mb56\"")),
                    ])
                    .with_key("q"),
                Param::of_type(api::ParamStyle::FieldList)
                    .values(vec![(Some("family-name"), None)])
                    .with_key("fl"),
            ];
            let response = endpoint.invoke_sync_with::<api::orcid::SearchField, api::orcid::OutputColumn>("search", Some(data));
            endpoint.handle::<api::orcid::SearchResponse>(response)
        }
        | None => Err("No ORCiD endpoint found".into()),
    };
    println!("ORCiD Search Response: {text:#?}");
    let text = match &orcid {
        | Some(endpoint) => {
            let response = endpoint.invoke_sync("status", None);
            endpoint.handle::<api::orcid::StatusResponse>(response)
        }
        | None => Err("No ORCiD endpoint found".into()),
    };
    println!("ORCiD Status: {text:#?}");
    // ROR API examples
    let ror = endpoints.find_by_name("ror");
    let text = match &ror {
        | Some(endpoint) => {
            let response = endpoint.invoke_sync("status", None);
            endpoint.handle::<api::TextResponse>(response)
        }
        | None => Err("No ROR endpoint found".into()),
    };
    println!("ROR Status: {text:#?}");
    let text = match &ror {
        | Some(endpoint) => {
            let data = vec![Param::of_type(api::ParamStyle::TemplateValue)
                .values(vec![(Some("01qz5mb56"), None)])
                .with_key("identifier")];
            let response = endpoint.invoke_sync("record", Some(data));
            endpoint.handle::<api::ror::SingleRecord>(response)
        }
        | None => Err("No ROR endpoint found".into()),
    };
    println!("ROR Record: {text:#?}");
    Ok(())
}
