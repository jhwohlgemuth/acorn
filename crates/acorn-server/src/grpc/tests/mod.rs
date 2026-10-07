use super::*;
fn authenticated<T>(message: T) -> Request<T> {
    let mut request = Request::new(message);
    request.metadata_mut().insert(AUTHORIZATION, "Bearer secret".parse().expect("metadata"));
    request
}
#[tokio::test]
async fn test_malformed_typed_enum_is_rejected() {
    let service = GrpcService::new(AcornService, Arc::<str>::from("secret"));
    let request = proto::CheckReadabilityRequest {
        metric: 999,
        source_uri: None,
        text: "text".to_string(),
        threshold: None,
    };
    let status = proto::acorn_service_server::AcornService::check_readability(&service, authenticated(request))
        .await
        .expect_err("unknown enum");
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}
#[tokio::test]
async fn test_typed_readability_rpc_requires_authentication() {
    let service = GrpcService::new(AcornService, Arc::<str>::from("secret"));
    let request = proto::CheckReadabilityRequest {
        metric: proto::ReadabilityMetric::Ari as i32,
        source_uri: None,
        text: "A complete sentence for deterministic analysis.".to_string(),
        threshold: None,
    };
    let status = proto::acorn_service_server::AcornService::check_readability(&service, Request::new(request.clone()))
        .await
        .expect_err("missing authentication");
    assert_eq!(status.code(), tonic::Code::Unauthenticated);
    let response = proto::acorn_service_server::AcornService::check_readability(&service, authenticated(request))
        .await
        .expect("typed response")
        .into_inner();
    assert_eq!(response.metric, proto::ReadabilityMetric::Ari as i32);
    assert!(response.value.is_finite());
}
