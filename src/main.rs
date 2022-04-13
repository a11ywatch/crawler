use website_crawler::interface::settings::Settings;
use website_crawler::{ rocket, grpc_start };
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Settings::new(true);

    tokio::spawn(async move {
        rocket().launch();
    });
    
    grpc_start().await?;

    Ok(())
}
