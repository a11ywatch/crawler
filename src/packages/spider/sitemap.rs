use super::utils::log;
use async_recursion::async_recursion;
use reqwest::{Client, IntoUrl};
use sitemap::{
    reader::{SiteMapEntity, SiteMapReader},
    structs::Location,
};
use tokio::sync::mpsc::Sender;

/// get the entire list of urls in a sitemap
#[async_recursion]
pub async fn get_sitemap_urls<T: IntoUrl + Send>(
    client: Client,
    url: T,
    stxx: Sender<String>,
) {
    match client.get(url).send().await {
        Ok(response) => {
            match response.text().await {
                Ok(text) => {
                    let parser = SiteMapReader::new(text.as_bytes());
                
                    for entity in parser {
                        match entity {
                            SiteMapEntity::Url(url_entry) => match url_entry.loc {
                                Location::None => {}
                                Location::Url(url) => {
                                    if let Err(_) = stxx.send(url.as_str().into()).await {
                                        log("receiver dropped", "");
                                    }
                                }
                                Location::ParseErr(error) => log("parse error entry url: ", error.to_string()),
                            },
                            SiteMapEntity::SiteMap(sitemap_entry) => match sitemap_entry.loc {
                                Location::None => {}
                                Location::Url(url) => {
                                    match get_sitemap_urls(client.clone(), url, stxx.clone()).await {
                                        _ => {},
                                    }
                                }
                                Location::ParseErr(err) => log("parse error sitemap url: ", err.to_string()),
                            },
                            SiteMapEntity::Err(err) => {
                                log("incorrect sitemap error: ", err.msg())
                            },
                        }
                    }
                },
                Err(_) => {}
            };
        },
        Err(_) => {}
    };
}
