use tokio;
use website_crawler::grpc_start;
use website_crawler::interface::settings::Settings;

/// gRPC server start.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Settings::new(true);

    grpc_start().await?;

    Ok(())
}
