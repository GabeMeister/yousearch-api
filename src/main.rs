#[macro_use]
extern crate rocket;

mod cors;
use cors::CORS;
use rocket::http::Header;
use rocket::{Request, Response};

#[get("/")]
fn index() -> &'static str {
    "Hello, there 5"
}

#[shuttle_runtime::main]
async fn rocket() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build().mount("/hello", routes![index]).attach(CORS);

    Ok(rocket.into())
}
