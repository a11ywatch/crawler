use std::env::var;

pub mod website {
    tonic::include_proto!("website");
}

use website::{ScanParams, website_service_client::WebsiteServiceClient};

// make request to the api server to perform scan action to gather results [TODO: MOVE CONNECTION OUTSIDE]
pub fn monitor_page(link: String) -> String {
    let page = ScanParams {
        pages: [link.to_string()].to_vec()
    };
    
    let rt = tokio::runtime::Runtime::new().unwrap();

    // TODO: move connect outside
    let async_capture = async {
        let addr = format!("http://{}", var("GRPC_HOST_API").unwrap_or("[::1]:50051".to_string()));        
        let mut client = WebsiteServiceClient::connect(addr).await.unwrap();
            
        let request = tonic::Request::new(page);
        
        client.scan(request).await.unwrap();
    };

    rt.block_on(async_capture);

    link
}