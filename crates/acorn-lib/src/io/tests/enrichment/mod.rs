use crate::io::api::{orcid, ror};
use crate::io::enrichment::{orcid_decision, Enrich, OrcidBatch, OrcidBatchResult, OrcidEnrichment, OrcidResolution, ResearchActivityEnrichment};
use crate::io::workflow::EnrichmentOrigin;
use acorn_core::Location;
use acorn_schema::enrichment::Provider;
use acorn_schema::research_activity::output::{Contributor as OutputContributor, ResearchOutput};
use acorn_schema::research_activity::{ActivityContributor, ResearchActivity, ResearchActivityMetadata};
use acorn_schema::standard::cff::{Agent as CffAgent, Cff, Person as CffPerson, Reference as CffReference};
use acorn_schema::standard::{datacite, invenio};
use acorn_schema::ContactPoint;

fn contact(given_name: &str, family_name: &str, identifier: Option<&str>) -> ContactPoint {
    ContactPoint {
        given_name: given_name.to_string(),
        family_name: family_name.to_string(),
        identifier: identifier.map(ToString::to_string),
        ..ContactPoint::default()
    }
}
fn profile(identifier: &str, given_name: Option<&str>, family_name: Option<&str>) -> orcid::SearchResult {
    orcid::SearchResult {
        orcid_id: Some(identifier.to_string()),
        given_names: given_name.map(ToString::to_string),
        family_names: family_name.map(ToString::to_string),
        credit_name: None,
        emails: None,
        institution_names: None,
        other_name: None,
    }
}
fn resolved(path: &str, profile: orcid::SearchResult) -> OrcidBatchResult {
    OrcidBatchResult {
        profiles: vec![(path.to_string(), profile)],
        ..OrcidBatchResult::default()
    }
}
fn response(results: impl IntoIterator<Item = orcid::SearchResult>) -> orcid::SearchResponse {
    let results = results.into_iter().collect::<Vec<_>>();
    orcid::SearchResponse {
        num_found: results.len(),
        namespace: String::new(),
        results,
    }
}
fn ror_record(identifier: &str, domain: &str) -> ror::SingleRecord {
    ror::SingleRecord::builder()
        .identifier(identifier.to_string())
        .domains(vec![domain.to_string()])
        .external_ids(Vec::new())
        .links(Vec::new())
        .locations(Vec::new())
        .names(vec![ror::NameField::builder()
            .value("Example Laboratory".to_string())
            .types(vec![ror::OrganizationNameType::RorDisplay])
            .build()])
        .relationships(Vec::new())
        .status(ror::Status::Active)
        .types(Vec::new())
        .build()
}
mod orcid_cases;
mod ror_cases;
