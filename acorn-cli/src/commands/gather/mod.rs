use acorn::io::api::orcid;
use acorn::io::network_get_request;
use acorn::prelude::PathBuf;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{Report, Result};

pub fn run(_path: &Option<PathBuf>, _ignore: &Option<String>, _offline: &bool, _verbose: &Verbosity) -> Result<(), Report> {
    let scheme = "https";
    let domain = "pub.orcid.org";
    let root = "v3.0";
    let resource = "expanded-search";

    let data = vec![("given-names", "Jason"), ("family-name", "Wohlgemuth")];
    let query = orcid::search_query(data, vec![], vec![]);
    let path = [domain, root, resource, &query].join("/");
    println!("Query: {scheme}://{path}");
    let content = match network_get_request(format!("{scheme}://{path}")).send() {
        | Err(e) => return Err(Report::new(e)),
        | Ok(response) => response.text().unwrap_or_default(),
    };
    println!("Response: {:#?}", content);
    Ok(())
}
