pub mod website {
    tonic::include_proto!("website");
}

pub mod crawler {
    tonic::include_proto!("crawler");
}

#[macro_use]
extern crate lazy_static;
use crate::tokio::macros::support::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status};
pub use website::website_service_server::{WebsiteService, WebsiteServiceServer};
pub use website::{Empty, ScanInitParams, ScanParams, ScanStreamResponse};

use std::env::var;
use std::net::SocketAddr;
use tokio;
use tonic::transport::Channel;
use tonic::transport::Server;

// gRPC crawler client
pub use crawler::crawler_client::CrawlerClient;
pub use crawler::{ScanReply, ScanRequest};

/// create gRPC client from the API server.
pub async fn create_client() -> Result<CrawlerClient<Channel>, tonic::transport::Error> {
    lazy_static! {
        static ref CLIENT: String = format!(
            "http://{}",
            var("GRPC_HOST_API").unwrap_or_else(|_| "0.0.0.0:50055".to_string())
        );
    };

    let client = CrawlerClient::connect(CLIENT.as_ref()).await?;

    Ok(client)
}

/// send request to server to start crawl.
pub async fn crawl_start(
    client: &mut CrawlerClient<Channel>,
    page: ScanRequest,
) -> Result<(), tonic::Status> {
    let request = tonic::Request::new(page);

    client.scan(request).await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct MyWebsiteService;

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<ScanStreamResponse, Status>> + Send>>;

#[tonic::async_trait]
impl WebsiteService for MyWebsiteService {
    type ScanStreamStream = ResponseStream;

    /// start scanning a website giving links crawled via grpc streams
    async fn scan_stream(
        &self,
        request: Request<ScanParams>,
    ) -> EchoResult<Self::ScanStreamStream> {
        let req = request.into_inner();
        println!("Received {:?}", req.pages[0]);

        let repeat = std::iter::repeat(ScanStreamResponse {
            message: req.domain,
        });

        let mut stream = Box::pin(tokio_stream::iter(repeat));
        let (tx, rx) = mpsc::channel(1);

        match stream.next().await {
            Some(item) => match tx.send(Ok(item)).await {
                _ => (),
            },
            _ => (),
        }

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }

    /// the scan started
    async fn scan_start(
        &self,
        request: Request<ScanInitParams>,
    ) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        println!("Scan starting for {:?} - id {:?}", req.domain, req.user_id);

        Ok(Response::new(website::Empty {}))
    }

    /// the scan finished
    async fn scan_end(&self, request: Request<ScanInitParams>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        println!("Scan finishing for {:?} - id {:?}", req.domain, req.user_id);

        Ok(Response::new(website::Empty {}))
    }

    /// full scan of all links
    async fn scan(&self, request: Request<ScanParams>) -> Result<Response<Empty>, Status> {
        let req = request.into_inner();
        println!("{:?}", req);

        let reply = website::Empty {};

        Ok(Response::new(reply))
    }
}

/// start the grpc server
pub async fn grpc_start() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = option_env!("GRPC_HOST")
        .unwrap_or("0.0.0.0:50051")
        .parse()?;
    let web = MyWebsiteService::default();

    println!("grpc website server listening on 0.0.0.0:50051");

    Server::builder()
        .add_service(WebsiteServiceServer::new(web))
        .serve(addr)
        .await?;

    Ok(())
}

/// gRPC web server start.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = tokio::task::spawn(async {
        match grpc_start().await {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err);
            }
        }
    });
    let args: Vec<String> = std::env::args().collect();

    let mut client = create_client().await.unwrap();
    let scan_request = ScanRequest {
        url: if args.len() > 1 {
            args[1].clone()
        } else {
            "https://jeffmendez.com".into()
        },
        ..Default::default()
    };

    tokio::task::spawn(async move {
        crawl_start(&mut client, scan_request).await.unwrap();
    });

    handle.await.unwrap();

    Ok(())
}
