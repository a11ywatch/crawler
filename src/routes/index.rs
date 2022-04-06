use rocket;

#[get("/")]
pub fn landing() -> String {
	format!("Welcome to web crawler!")
}
