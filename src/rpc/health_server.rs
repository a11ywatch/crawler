pub mod healthcheck {
    tonic::include_proto!("health");
}

#[derive(Default)]
pub struct HealthChecker {}

#[tonic::async_trait]
impl HealthCheck for HealthChecker {
    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> std::result::Result<Response<HealthCheckReply>, Status> {
        Ok(Response::new(HealthCheckReply { healthy: true }))
    }
}
