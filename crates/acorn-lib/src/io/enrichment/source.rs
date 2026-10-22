use super::{
    async_trait, stream, warn, Enrich, InputOutput, Location, Pipeline, ResearchActivity, SourceDocument, SourceDocuments, StreamExt, String,
    ToString, Vec,
};

#[async_trait]
impl Enrich<()> for SourceDocument {
    type Output = (Self, Option<String>, Vec<String>);
    async fn enrich(&self, (): ()) -> Self::Output {
        match ResearchActivity::parse(&self.content) {
            | Some(data) => match Pipeline::default().enrich((Location::from(self.source.as_str()), data)).await {
                | Ok(result) => {
                    result.conflicts.iter().for_each(|conflict| warn!("Enrichment conflict — {conflict}"));
                    match serde_json::to_string(&result.data) {
                        | Ok(content) => (
                            Self::init()
                                .content(content.clone())
                                .format(self.format.clone())
                                .source(self.source.clone())
                                .build(),
                            Some(content),
                            result.failures,
                        ),
                        | Err(why) => (self.clone(), None, result.failures.into_iter().chain([why.to_string()]).collect()),
                    }
                }
                | Err(why) => (self.clone(), None, vec![why.to_string()]),
            },
            | None => (self.clone(), None, Vec::new()),
        }
    }
}
#[async_trait]
impl Enrich<bool> for SourceDocuments {
    type Output = (Self, Vec<String>, Vec<String>);
    async fn enrich(&self, enabled: bool) -> Self::Output {
        match enabled {
            | false => (self.clone(), Vec::new(), Vec::new()),
            | true => stream::iter(self.0.iter())
                .then(|document| document.enrich(()))
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .fold(
                    (Self::default(), Vec::new(), Vec::new()),
                    |(documents, enriched, failures), (document, content, errors)| {
                        (
                            Self(documents.0.into_iter().chain([document]).collect()),
                            enriched.into_iter().chain(content).collect(),
                            failures.into_iter().chain(errors).collect(),
                        )
                    },
                ),
        }
    }
}
