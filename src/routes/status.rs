use rocket;
use rocket_contrib::json::JsonValue;

#[get("/_internal_/healthcheck")]
pub fn get_health() -> JsonValue {
	json!({ "status": "healthy" })
}