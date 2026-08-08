//! Module for downloading, parsing, and using GeoNames data
//!
//! See [Geonames web services documentation](https://www.geonames.org/export/web-services.html) for more information
use crate::io::api::{ApiResult, RemoteResource, TextResponse, INCLUDED_ENDPOINTS};
use crate::schema::geonames::Countries;
use crate::util::Searchable;
use color_eyre::eyre::eyre;

/// Type alias for GeoNames API reponse for country data, which is returned as TSV text that can be parsed into structured data
pub type SearchResponse = TextResponse;
/// Pull GeoNames country data from the API and parse it into structured data
pub async fn download() -> ApiResult<Countries> {
    let name = "GeoNames";
    match INCLUDED_ENDPOINTS.find_by_name(name) {
        | Some(endpoint) => {
            let response = endpoint.invoke("countries", None).await;
            endpoint.handle::<SearchResponse>(response).map(|text| text.to_string().into())
        }
        | None => Err(eyre!("{name} endpoint not found")),
    }
}
