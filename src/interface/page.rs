/*
 * Copyright (c) A11yWatch, LLC. and its affiliates.
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 **/

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
	pub pages: Vec<String>,
	pub user_id: u32,
	pub domain: String,
}

// used for anon job
#[derive(Debug, Serialize, Deserialize)]
pub struct PageSingle {
	pub pages: Vec<String>
}
