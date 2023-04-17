#[macro_use]
extern crate rocket;
extern crate dotenv;

mod cors;
mod endpoints;
mod utils;

use cors::CORS;
use dotenv::dotenv;
use endpoints::general::ApiState;
use sqlx::postgres::PgPoolOptions;
use std::env;

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
                endpoints::users::delete_user,
                endpoints::videos::get_videos,
                endpoints::videos::create_video,
                endpoints::videos::search_video_captions,
                endpoints::videos::test_video,
            ],
        )
}
