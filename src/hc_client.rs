use health::health_check_client::HealthCheckClient;
use health::HealthCheckRequest;

pub mod health {
    tonic::include_proto!("health");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = HealthCheckClient::connect("http://127.0.0.1:50055").await?;
    let response = client
        .check(tonic::Request::new(HealthCheckRequest {}))
        .await
        .unwrap();

    let inner = response.into_inner();

    println!("{:?}", inner);

    Ok(())
}
