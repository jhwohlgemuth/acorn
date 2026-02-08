use acorn::io::api::{self, orcid};
use acorn::io::network_get_request;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

pub fn run(_path: &Option<PathBuf>, _ignore: &Option<String>, _offline: &bool, _verbose: &Verbosity) -> Result<(), Report> {
    // let response = api.invoke::<orcid::SearchResponse>("search", Some(data));
    let orcid = api::Endpoint::init().domain("pub.orcid.org").root("v3.0").build();
    let resource = "expanded-search";
    let data = vec![("given-names", "Jason"), ("family-name", "Wohlgemuth")];
    let query = orcid::query_string(data, vec![], vec![]);
    let path = [&orcid.base(), resource, &query].join("/");
    println!("Query: {path}");
    let content = match network_get_request(path.to_string()).send() {
        | Err(e) => return Err(Report::new(e)),
        | Ok(response) => response.text().unwrap_or_default(),
    };
    println!("Raw response: {content}");
    let data = api::parse::<orcid::SearchResponse>(&content).unwrap_or_else(|e| panic!("Failed to parse ORCiD search response: {e}"));
    println!("Parsed data: {:#?}", data);

    Ok(())
}
