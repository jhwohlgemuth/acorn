use super::*;
#[tokio::test]
async fn test_health_tracks_shared_readiness() {
    let ready = Arc::new(AtomicBool::new(false));
    let service = HealthService::new(Arc::clone(&ready));
    let response = Health::check(&service, Request::new(HealthCheckRequest::default()))
        .await
        .expect("health response")
        .into_inner();
    assert_eq!(response.status, ServingStatus::NotServing as i32);
    ready.store(true, Ordering::Release);
    let response = Health::check(&service, Request::new(HealthCheckRequest::default()))
        .await
        .expect("health response")
        .into_inner();
    assert_eq!(response.status, ServingStatus::Serving as i32);
}
