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
