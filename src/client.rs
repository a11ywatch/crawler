pub mod crawler {
    tonic::include_proto!("crawler");
}

use crawler::{HelloRequest, ScanRequest, greeter_client::GreeterClient};
use std::net::SocketAddr;

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GreeterClient::connect("dns://[::1]:50055").await?;
    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

#[allow(dead_code)] // test client connection
pub async fn scan_rpc() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = option_env!("GRPC_HOST").unwrap_or("[::1]:50055").parse()?;
    let mut client = GreeterClient::connect(format!("dns://{}", addr)).await?;
    
    let request = tonic::Request::new(ScanRequest {
        url: "https://a11ywatch.com".into(),
        id: 1,
    });

    let response = client.scan(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}

pub async fn ping() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = option_env!("GRPC_HOST").unwrap_or("[::1]:50055").parse()?;
    let mut client = GreeterClient::connect(format!("dns://{}", addr)).await?;
    
    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });
    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}