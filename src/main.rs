/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

use std::env;
use website_crawler;

fn main() {
    let mut page_url = "http://api:8080/api/website-crawl-background".to_string();

    for (key, value) in env::vars() {
        if key == "CRAWL_URL" {
            page_url = value.to_string();
        }
    }

    println!("{}", page_url);
    env::set_var("CRAWL_URL", page_url);

    website_crawler::rocket().launch();
}
