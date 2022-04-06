use num_cpus;
use rocket;
use rocket_contrib::json::JsonValue;

#[get("/_internal_/healthcheck")]
pub fn get_health() -> JsonValue {
	json!({ "status": "healthy" })
}

#[get("/cpu")]
pub fn get_cpu() -> String {
	format!("Number of logic cores is {}", num_cpus::get())
}