use website_crawler::interface::settings::Settings;
use website_crawler::{ rocket, grpc_start };
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings: Settings = Settings::new(true);
    drop(settings);

    tokio::spawn(async move {
        rocket().launch();
    });
    
    grpc_start().await?;

    Ok(())
}
