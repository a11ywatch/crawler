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

    let crawl_url = match env::var(key) {
        Ok(val) => val.to_string(),
        Err(_) => "".to_string(),
    };

    println!("crawl message url {}", crawl_url);

    env::set_var(key, crawl_url);

    website_crawler::rocket().launch();
}
