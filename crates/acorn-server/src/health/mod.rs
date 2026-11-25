//! Standard gRPC health adapter backed by process readiness.
use crate::health_proto::health_server::Health;
use crate::health_proto::{health_check_response::ServingStatus, HealthCheckRequest, HealthCheckResponse};
use alloc::sync::Arc;
use core::pin::Pin;
use core::sync::atomic::{AtomicBool, Ordering};
use futures::Stream;
use tonic::{Request, Response, Status};

/// Health service shared with HTTP readiness state.
#[derive(Clone)]
pub struct HealthService {
    ready: Arc<AtomicBool>,
}
#[tonic::async_trait]
impl Health for HealthService {
    type WatchStream = Pin<Box<dyn Stream<Item = Result<HealthCheckResponse, Status>> + Send>>;

    async fn check(&self, _request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(self.response()))
    }

    async fn watch(&self, _request: Request<HealthCheckRequest>) -> Result<Response<Self::WatchStream>, Status> {
        Ok(Response::new(Box::pin(futures::stream::once(core::future::ready(Ok(self.response()))))))
    }
}
impl HealthService {
    /// Construct a service over the process readiness flag.
    pub fn new(ready: Arc<AtomicBool>) -> Self {
        Self { ready }
    }

    fn response(&self) -> HealthCheckResponse {
        let status = match self.ready.load(Ordering::Acquire) {
            | true => ServingStatus::Serving,
            | false => ServingStatus::NotServing,
        };
        HealthCheckResponse { status: status as i32 }
    }
}
#[cfg(test)]
mod tests;
