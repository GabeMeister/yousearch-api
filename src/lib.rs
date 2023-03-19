#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world! This is yousearch3"
}

#[shuttle_service::main]
pub async fn rocket() -> shuttle_service::ShuttleRocket {
    let rocket = rocket::build().mount("/hello", routes![index]);

    Ok(rocket)
}
