#[macro_use]
extern crate rocket;
extern crate dotenv;

mod cors;
mod endpoints;

use cors::CORS;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use endpoints::general::ApiState;

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap();

    let pool = PgPoolOptions::new()
        .max_connections(100)
        .connect(&db_url)
        .await
        .expect("Unable to connect to Postgres");

    rocket::build()
        .manage(ApiState { pool })
        .attach(CORS)
        .mount(
            "/",
            routes![
                endpoints::general::index,
                endpoints::users::get_all_users,
                endpoints::users::get_user,
                endpoints::users::insert_user,
                endpoints::users::update_user,
                endpoints::users::delete_user
            ],
        )
}
