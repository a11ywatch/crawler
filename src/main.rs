use dotenv::dotenv;
use std::env;
use website_crawler;
use website_crawler::interface::settings::Settings;

fn main() {
    dotenv().ok();
    let settings: Settings = Settings::new();

    env::set_var("CRAWL_URL", &settings.crawl_url);
    env::set_var("CRAWL_URL_BACKGROUND", &settings.crawl_url_background);
    env::set_var("SCAN_URL_START", &settings.scan_url_start);
    env::set_var("SCAN_URL_COMPLETE", &settings.scan_url_complete);
    env::set_var("RUST_LOG", &settings.configuration_verbose);

    website_crawler::rocket().launch();
}
