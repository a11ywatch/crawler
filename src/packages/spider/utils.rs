use log::{info, log_enabled, Level};
use reqwest::Client;
use reqwest::StatusCode;

/// Perform a network request to a resource extracting all content as text streaming.
pub async fn fetch_page_html(url: &str, client: &Client) -> Option<String> {
    use tokio_stream::StreamExt;

    match client.get(url).send().await {
        Ok(res) if res.status() == StatusCode::OK => {
            let mut stream = res.bytes_stream();
            let mut data: String = String::new();

            while let Some(item) = stream.next().await {
                match item {
                    Ok(text) => {
                        data += &String::from_utf8_lossy(&text);
                    }
                    _ => (),
                }
            }

            Some(data)
        }
        Ok(_) => None,
        Err(_) => {
            log("- error parsing html text {}", &url);
            None
        }
    }
}


/// log to console if configuration verbose.
pub fn log(message: &'static str, data: impl AsRef<str>) {
    if log_enabled!(Level::Info) {
        info!("{message} - {}", data.as_ref());
    }
}
