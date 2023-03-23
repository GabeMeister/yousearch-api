#[macro_use]
extern crate rocket;
mod cors;
use cors::CORS;

// #[derive(Serialize, FromRow)]
// struct Todo {
//     pub id: i32,
//     pub note: String,
// }

// #[derive(Deserialize)]
// struct TodoNew {
//     pub note: String,
// }

// struct MyState {
//     pool: PgPool,
// }

#[get("/")]
fn index() -> &'static str {
    "It's working!"
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hello world!"
}

#[shuttle_runtime::main]
async fn rocket() -> shuttle_rocket::ShuttleRocket {
    let rocket = rocket::build()
        .mount("/", routes![index, hello])
        .attach(CORS);

    Ok(rocket.into())
}
