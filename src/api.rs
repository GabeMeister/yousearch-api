use rocket::Rocket;
use rocket::{get, routes, Build};

#[get("/hello")]
fn hello() -> &'static str {
    "Hello, human..."
}

#[get("/")]
fn index() -> &'static str {
    "You have reached the official yousearch api!!! Yay!"
}

pub fn build_api() -> Rocket<Build> {
    return rocket::build().mount("/", routes![index, hello]);
}
