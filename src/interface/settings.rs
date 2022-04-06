use std::env::var;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub crawl_url: String,
    pub crawl_url_background: String,
    pub scan_url_start: String,
    pub scan_url_complete: String,
    pub configuration_verbose: String
}

impl Settings {
    pub fn new(establish: bool) -> Settings {
        let crawl_url = var("CRAWL_URL").unwrap_or_else(|_| 
            "http:///127.0.0.1:8080/api/website-crawl".into());
        let crawl_url_background = var("CRAWL_URL_BACKGROUND").unwrap_or_else(|_| 
            "http:///127.0.0.1:8080/api/website-crawl-background".into());

        let scan_url_start = var("SCAN_URL_START").unwrap_or_else(|_| 
            "http:///127.0.0.1:8080/127.0.0.1/api/website-crawl-background-start".into());
        let scan_url_complete = var("SCAN_URL_COMPLETE").unwrap_or_else(|_| 
            "http:///127.0.0.1:8080/api/website-crawl-background-complete".into());
            
        let configuration_verbose = match var("RUST_LOG") {
            Ok(_) => "true".to_string(),
            Err(_) => "false".to_string(),
        };

        if establish {
            env::set_var("CRAWL_URL", &crawl_url);
            env::set_var("CRAWL_URL_BACKGROUND", &crawl_url_background);
            env::set_var("SCAN_URL_START", &scan_url_start);
            env::set_var("SCAN_URL_COMPLETE", &scan_url_complete);
            env::set_var("RUST_LOG", &configuration_verbose);
        }

        Self {
            crawl_url,
            crawl_url_background,
            scan_url_start,
            scan_url_complete,
            configuration_verbose
        }
    }
}

impl Drop for Settings {
    fn drop(&mut self) {}
}
