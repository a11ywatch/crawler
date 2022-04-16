pub mod health {
    tonic::include_proto!("health");
}

pub use health::health_check_server::{HealthCheck, HealthCheckServer};
pub use health::{HealthCheckReply, HealthCheckRequest};

use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct HealthChecker;

#[tonic::async_trait]
impl HealthCheck for HealthChecker {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckReply>, Status> {
        Ok(Response::new(HealthCheckReply { healthy: true }))
    }
}
