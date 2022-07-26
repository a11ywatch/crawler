pub mod website {
    tonic::include_proto!("website");
}

use std::env::var;

use tonic::transport::Channel;

use crate::spider::utils::log;
pub use website::{website_service_client::WebsiteServiceClient, ScanParams};

/// create gRPC client from the API server.
pub async fn create_client() -> Result<WebsiteServiceClient<Channel>, tonic::transport::Error> {
    lazy_static! {
        static ref CLIENT: String = format!(
            "http://{}", var("GRPC_HOST_API").unwrap_or("[::1]:50051".to_string()));
    };

    let client = WebsiteServiceClient::connect(CLIENT.as_ref()).await?;

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

/// request to the API server to perform scan action to gather results.
pub async fn monitor_page_async(page: ScanParams) -> Result<(), tonic::Status> {
    let mut client = create_client().await.unwrap();
    let request = tonic::Request::new(page);

    client.scan(request).await?;

    Ok(())
}

/// run a accessibility scan waiting for results.
pub async fn monitor(
    client: &mut WebsiteServiceClient<Channel>,
    link: String,
    user_id: u32,
) -> bool {
    let page = ScanParams {
        pages: [link.clone()].to_vec(),
        user_id,
        ..Default::default()
    };
    let request = tonic::Request::new(page);
    let mut stream = client.scan_stream(request).await.unwrap().into_inner();

    let mut perform_shutdown = false;

    while let Some(res) = stream.message().await.unwrap() {
        if res.message == "shutdown" {
            perform_shutdown = true;
        }
        log("gRPC(stream): finished -", link.clone());
    }

    // shutdown the thread
    !perform_shutdown
}
