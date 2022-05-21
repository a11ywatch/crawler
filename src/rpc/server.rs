pub mod crawler {
    tonic::include_proto!("crawler");
}

pub use crawler::greeter_server::{Greeter, GreeterServer};
pub use crawler::{ScanReply, ScanRequest};
use tonic::{Request, Response, Status};

use crate::scanner::scan::scan as scanPage;
use crate::scanner::crawl::crawl as crawlPage;

#[derive(Debug, Default)]
pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    /// start scanning a website giving links crawled in realtime to gRPC API. TODO: move to stream.
    async fn scan(&self, request: Request<ScanRequest>) -> Result<Response<ScanReply>, Status> {
        let req = request.into_inner();

        let url = req.url;
        let respect_robots_txt = req.norobots == false;
        let agent = req.agent;

        let reply = crawler::ScanReply {
            message: format!("scanning - {:?}", &url).into(),
        };

        tokio::spawn(async move {
            scanPage(&url, req.id, respect_robots_txt, &agent).await.unwrap_or_default();
        });

        Ok(Response::new(reply))
    }
    /// crawl website and send all links crawled when completed to API client gRPC.
    async fn crawl(&self, request: Request<ScanRequest>) -> Result<Response<ScanReply>, Status> {
        let req = request.into_inner();
        
        let url = req.url;
        let respect_robots_txt = req.norobots == false;
        let agent = req.agent;

        let reply = crawler::ScanReply {
            message: format!("scanning - {:?}", &url).into(),
        };

        tokio::spawn(async move {
            crawlPage(&url, req.id, respect_robots_txt, &agent).await.unwrap_or_default();
        });

        Ok(Response::new(reply))
    }
}
