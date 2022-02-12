/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

use dotenv::dotenv;
use std::env;
use website_crawler;

fn main() {
    dotenv().ok();
    let key = "CRAWL_URL";
    let key_complete = "CRAWL_URL_COMPLETE";

    let crawl_url = match env::var(key) {
        Ok(val) => val.to_string(),
        Err(_) => "".to_string(),
    };
    let crawl_url_complete = match env::var(key_complete) {
        Ok(val) => val.to_string(),
        Err(_) => "".to_string(),
    };

    println!("crawl message url {}", crawl_url);
    println!("crawl complete message url {}", crawl_url_complete);

    env::set_var(key, crawl_url);

    website_crawler::rocket().launch();
}
