pub mod website {
    tonic::include_proto!("website");
}

use std::env::var;

use tonic::{transport::Channel, Streaming};

use crate::spider::utils::log;

// gRPC client
pub use website::{
    website_service_client::WebsiteServiceClient, ScanInitParams, ScanParams, ScanStreamResponse,
};

/// create gRPC client from the API server.
pub async fn create_client() -> Result<WebsiteServiceClient<Channel>, tonic::transport::Error> {
    lazy_static! {
        static ref CLIENT: String = format!(
            "http://{}",
            var("GRPC_HOST_API").unwrap_or_else(|_| "[::1]:50051".to_string())
        );
    };

    let client = WebsiteServiceClient::connect(CLIENT.as_ref()).await?;

    Ok(client)
}

/// request to the API server that scan has started.
pub async fn monitor_page_start(
    client: &mut WebsiteServiceClient<Channel>,
    page: ScanInitParams,
) -> Result<(), tonic::Status> {
    let request = tonic::Request::new(page);

    client.scan_start(request).await?;

    Ok(())
}

/// request to the API server that scan has finished.
pub async fn monitor_page_complete(
    client: &mut WebsiteServiceClient<Channel>,
    page: ScanInitParams,
) -> Result<(), tonic::Status> {
    let request = tonic::Request::new(page);

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
    html: String,
) -> bool {
    let request = tonic::Request::new(ScanParams {
        pages: vec![link],
        user_id,
        html,
        ..Default::default()
    });
    let stream: Option<Streaming<ScanStreamResponse>> = match client.scan_stream(request).await {
        Ok(val) => Some(val.into_inner()),
        Err(e) => {
            log("error status-code :", &e.code().to_string());

            None
        }
    };

    let mut perform_shutdown = false; // shutdown was performed on website - terminate entire crawl

    match stream {
        Some(mut res) => {
            while let Some(r) = res.message().await.unwrap_or_default() {
                if r.message == "shutdown" {
                    perform_shutdown = true;
                }
            }
        }
        None => {
            // perform shut down true
            perform_shutdown = true;
        }
    };

    !perform_shutdown
}
