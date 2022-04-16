pub mod crawler {
    tonic::include_proto!("crawler");
}

pub use crawler::greeter_server::{Greeter, GreeterServer};
pub use crawler::{ScanReply, ScanRequest};
use tonic::{Request, Response, Status};

use crate::scanner::scan::scan as scanPage;

#[derive(Debug, Default)]
pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    // TODO: MOVE TO STREAM INSTEAD OF SENDING RPC FROM CLIENT BACK
    async fn scan(&self, request: Request<ScanRequest>) -> Result<Response<ScanReply>, Status> {
        let req = request.into_inner();
        let url = req.url.clone();

        let reply = crawler::ScanReply {
            message: format!("scanning {:?}!", &url).into(),
        };

        tokio::spawn(async move {
            scanPage(url, req.id).await.expect("scan failed to start");
        });

        Ok(Response::new(reply))
    }
    // used to gather all links first before sending to api [TODO]
    async fn crawl(&self, request: Request<ScanRequest>) -> Result<Response<ScanReply>, Status> {
        let req = request.into_inner();
        let url = req.url.clone();

        let reply = crawler::ScanReply {
            message: format!("scanning {:?}!", &url).into(),
        };

        tokio::spawn(async move {
            scanPage(url, req.id).await.expect("crawl failed to start");
        });

        Ok(Response::new(reply))
    }
}
