use std::env::{set_var, var};

#[derive(Debug, Clone)]
pub struct Settings {
    /// gRPC website server host to send request of links found.
    pub grpc_api_host: String,
    /// Determine if logging should be enabled.
    pub configuration_verbose: String,
}

impl Settings {
    pub fn new(establish: bool) -> Settings {
        let grpc_api_host = var("GRPC_HOST_API").unwrap_or_else(|_| "0.0.0.0:50051".into());
        let configuration_verbose = match var("RUST_LOG") {
            Ok(_) => "true".to_string(),
            Err(_) => "false".to_string(),
        };

        if establish {
            set_var("GRPC_HOST_API", &grpc_api_host);
            set_var("RUST_LOG", &configuration_verbose);
        }

        Self {
            grpc_api_host,
            configuration_verbose,
        }
    }
}

impl Drop for Settings {
    fn drop(&mut self) {}
}
