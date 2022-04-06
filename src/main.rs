use website_crawler::interface::settings::Settings;
use website_crawler::rocket;

fn main() {
    let settings: Settings = Settings::new(true);
    drop(settings);

    rocket().launch();
}
