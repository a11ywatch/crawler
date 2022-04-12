use tonic::{transport::Server, Request, Response, Status};
use std::net::SocketAddr;
use std::thread;

pub mod crawler {
    tonic::include_proto!("crawler");
}

use crawler::greeter_server::{Greeter, GreeterServer};
use crawler::{HelloReply, ScanReply, ScanRequest, HelloRequest};

use crate::scanner::scan::{scan as scanPage};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request: {:?}", request);

        let reply = crawler::HelloReply {
            message: format!("Hello {:?}!", request.into_inner().name).into(),
        };

        Ok(Response::new(reply))
    }

    async fn scan(
        &self,
        request: Request<ScanRequest>,
    ) -> Result<Response<ScanReply>, Status> {
        let req = request.into_inner();
        let url = req.url;

        let reply = crawler::ScanReply {
            message: format!("scanning {:?}!", &url).into(),
        };

        let handle = thread::spawn(move || {
            scanPage(url, req.id);
        });
      
        drop(handle);

        Ok(Response::new(reply))
    }

    async fn crawl(
        &self,
        request: Request<ScanRequest>,
    ) -> Result<Response<ScanReply>, Status> {
        let req = request.into_inner();
        let url = req.url;

        let reply = crawler::ScanReply {
            message: format!("scanning {:?}!", &url).into(),
        };

        let handle = thread::spawn(move || {
            scanPage(url, req.id);
        });
      
        drop(handle);

        Ok(Response::new(reply))
    }

}

// start the grpc server
pub async fn grpc_start() -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = option_env!("GRPC_HOST").unwrap_or("[::1]:50055").parse()?;
    let greeter = MyGreeter::default();

    Server::builder()
        .add_service(GreeterServer::new(greeter))
        .serve(addr)
        .await?;

        println!("starting");

    Ok(())
}