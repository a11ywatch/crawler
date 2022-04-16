use health::health_check_client::HealthCheckClient;
use health::HealthCheckRequest;

use tokio;
use website_crawler::grpc_start;
use website_crawler::interface::settings::Settings;

pub mod health {
    tonic::include_proto!("health");
}

#[tokio::test]
async fn test_healthcheck() -> Result<(), Box<dyn std::error::Error>> {
    Settings::new(true);

    tokio::spawn(async move {
        let mut client = HealthCheckClient::connect("http://0.0.0.0:50055")
            .await
            .unwrap();
        let response = client
            .check(tonic::Request::new(HealthCheckRequest {}))
            .await
            .unwrap();

        let inner = response.into_inner();

        println!("{:?}", inner);

        assert_eq!(inner.healthy, true);

        std::process::exit(0);
    });

    grpc_start().await.unwrap();

    Ok(())
}
