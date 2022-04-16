use tokio;
use website_crawler::interface::settings::Settings;
use website_crawler::{grpc_start, rocket};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Settings::new(true);

    tokio::spawn(async move {
        rocket().launch();
    });

    grpc_start().await?;

    Ok(())
}
