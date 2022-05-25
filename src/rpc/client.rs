pub mod website {
    tonic::include_proto!("website");
}

use std::env::var;
use tonic::transport::Channel;

pub use website::{website_service_client::WebsiteServiceClient, ScanParams};

use tokio;

/// get the gRPC client address for the API server.
pub fn get_client_address() -> String {
    format!(
        "http://{}",
        var("GRPC_HOST_API").unwrap_or("[::1]:50051".to_string())
    )
}

/// create gRPC client from the API server.
pub async fn create_client() -> Result<WebsiteServiceClient<Channel>, tonic::transport::Error> {
    let client = WebsiteServiceClient::connect(get_client_address()).await?;

    Ok(client)
}

/// request to the API server that scan has started.
pub async fn monitor_page_start(
    client: &mut WebsiteServiceClient<Channel>,
    page: &ScanParams,
) -> Result<(), tonic::Status> {
    let request = tonic::Request::new(page.to_owned());

    client.scan_start(request).await?;

    Ok(())
}

/// request to the API server that scan has finished.
pub async fn monitor_page_complete(
    client: &mut WebsiteServiceClient<Channel>,
    page: &ScanParams,
) -> Result<(), tonic::Status> {
    let request = tonic::Request::new(page.to_owned());

    client.scan_end(request).await?;

    Ok(())
}

/// request to the API server to perform scan action to gather results [TODO: MOVE CONNECTION OUTSIDE]
pub async fn monitor_page_async(page: ScanParams) -> Result<(), tonic::Status> {
    let mut client = create_client().await.unwrap();
    let request = tonic::Request::new(page);

    client.scan(request).await?;

    Ok(())
}


/// request to the API server to perform scan action to gather results re-using connection.
pub async fn monitor(
    client: &mut WebsiteServiceClient<Channel>,
    page: &ScanParams,
) -> Result<(), tonic::Status> {
    let request = tonic::Request::new(page.to_owned());

    client.scan(request).await?;

    Ok(())
}

/// make request to the api server to perform scan action to gather results [TODO: MOVE CONNECTION OUTSIDE]
pub async fn monitor_link_async(link: &String) -> Result<(), tonic::Status> {
    let mut client = create_client().await.unwrap();

    let page = ScanParams {
        pages: [link.to_string()].to_vec(),
        ..Default::default()
    };

    let request = tonic::Request::new(page);

    client.scan(request).await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
/// callback to run on website link find
pub async fn monitor_page(link: String) -> String {
    let link_target = link.clone();
    
    monitor_link_async(&link_target).await
        .unwrap_or_else(|e| println!("monitor task error: {:?}", e));

    link
}
