use crate::config::ApplicationConfiguration;
use acorn::io::api::{self, orcid, spdx, Endpoint, EndpointSearch, RestfulInterface};
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

const ENDPOINTS: &str = r#"{
    "endpoints": [
        {
            "name": "orcid",
            "domain": "https://orcid.org",
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
            "domain": "https://api.ror.org",
            "root": "",
            "resources": [
                {
                    "name": "status",
                    "method": "get",
                    "template": "{{ base }}/heartbeat"
                }
            ]
        },
        {
            "name": "github",
            "domain": "https://api.github.com",
            "root": "",
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
            "domain": "https://code.ornl.gov",
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
            "name": "spdx",
            "domain": "https://spdx.org",
            "root": "",
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
    let resources = match endpoints.find_by_name("spdx") {
        | Some(Endpoint { resources, .. }) => resources.clone(),
        | None => vec![],
    };
    let spdx = api::Endpoint::at("spdx.org").resources(resources).build();
    let response = spdx.invoke_sync::<api::EmptyField, api::EmptyField>("licenses", None);
    let text = spdx.handle::<spdx::Licenses>(response);
    println!("Licenses: {text:#?}");
    // ORCiD API examples
    let resources = match endpoints.find_by_name("orcid") {
        | Some(Endpoint { resources, .. }) => resources.clone(),
        | None => vec![],
    };
    let data = vec![
        api::Param::of_type(api::ParamStyle::QueryPair)
            .values(vec![
                (Some("affiliation-org-name"), Some("Lyrasis")),
                (Some("ror-org-id"), Some("\"https://ror.org/01qz5mb56\"")),
            ])
            .with_key("q"),
        api::Param::of_type(api::ParamStyle::FieldList)
            .values(vec![(Some("family-name"), None)])
            .with_key("fl"),
        api::Param::of_type(api::ParamStyle::TemplateValue)
            .values(vec![(Some("two"), None)])
            .with_key("one"),
        api::Param::of_type(api::ParamStyle::TemplateValue)
            .values(vec![(None, Some("four"))])
            .with_key("three"),
    ];
    let orcid = api::Endpoint::at("pub.orcid.org").root("v3.0").resources(resources.clone()).build();
    // Search for ORCiDs
    let response = orcid.invoke_sync::<orcid::SearchField, orcid::OutputColumn>("search", Some(data));
    let text = orcid.handle::<orcid::SearchResponse>(response);
    println!("Response: {text:#?}");
    // Check the status of the API
    let response = orcid.invoke_sync::<api::EmptyField, api::EmptyField>("status", None);
    let text = orcid.handle::<orcid::StatusResponse>(response);
    println!("ORCiD Status: {text:#?}");
    // ROR API example
    let resources = match endpoints.find_by_name("ror") {
        | Some(Endpoint { resources, .. }) => resources.clone(),
        | None => vec![],
    };
    let ror = api::Endpoint::at("api.ror.org").resources(resources.clone()).build();
    let response = ror.invoke_sync::<api::EmptyField, api::EmptyField>("status", None);
    let text = ror.handle::<api::TextResponse>(response);
    println!("ROR Status: {text:#?}");
    Ok(())
}
