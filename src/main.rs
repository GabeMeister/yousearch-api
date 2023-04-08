#[macro_use]
extern crate rocket;
extern crate dotenv;

mod cors;

use cors::CORS;
use dotenv::dotenv;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use rocket::State;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, FromRow, PgPool};
use std::env;

#[derive(Debug, FromRow, Serialize)]
struct User {
    id: i32,
    name: String,
}
#[derive(Debug, Deserialize)]
struct NewUser {
    name: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUserBody {
    name: String,
}

struct ApiState {
    pool: PgPool,
}

#[derive(Debug, Serialize)]
struct SuccessFailResponse {
    success: bool,
}

#[derive(Debug, Serialize)]
struct NewUserIdResponse {
    id: i32,
}

#[get("/")]
fn index() -> String {
    let db_url = env::var("RUST_ENV").unwrap();

    format!("It works! {db_url}")
}

#[get("/user/<id>")]
async fn get_user(id: i32, state: &State<ApiState>) -> Json<Option<User>> {
    let user = sqlx::query_as::<_, User>("select id, name from users where id=$1")
        .bind(id)
        .fetch_one(&state.pool)
        .await;

    match user {
        Ok(u) => Json(Some(u)),
        Err(e) => {
            dbg!(e);
            Json(None)
        }
    }
}

#[get("/user/all")]
async fn get_all_users(state: &State<ApiState>) -> Json<Vec<User>> {
    let result = sqlx::query_as::<_, User>("select id, name from users")
        .fetch_all(&state.pool)
        .await;

    match result {
        Ok(all_users) => Json(all_users),
        Err(_) => Json(Vec::<User>::new()),
    }
}

#[post("/user", data = "<user>")]
async fn insert_user(user: Json<NewUser>, state: &State<ApiState>) -> Json<NewUserIdResponse> {
    dbg!(user.name.clone());

    let result: Result<i32, Error> =
        sqlx::query_scalar("insert into users (name, password) values ($1, $2) returning id")
            .bind(user.name.clone())
            .bind(user.password.clone())
            .fetch_one(&state.pool)
            .await;

    match result {
        Ok(id) => Json(NewUserIdResponse { id }),
        Err(_) => Json(NewUserIdResponse { id: -1 }),
    }
}

#[patch("/user/<id>", data = "<user>")]
async fn update_user(
    id: i32,
    user: Json<UpdateUserBody>,
    state: &State<ApiState>,
) -> Json<SuccessFailResponse> {
    let result = sqlx::query("update users set name=$1 where id=$2")
        .bind(user.name.clone())
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => Json(SuccessFailResponse { success: true }),
        Err(_) => Json(SuccessFailResponse { success: false }),
    }
}

#[delete("/user/<id>")]
async fn delete_user(id: i32, state: &State<ApiState>) -> Json<SuccessFailResponse> {
    let result = sqlx::query("delete from users where id=$1")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => Json(SuccessFailResponse { success: true }),
        Err(_) => Json(SuccessFailResponse { success: false }),
    }
}

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
                index,
                get_user,
                get_all_users,
                insert_user,
                update_user,
                delete_user
            ],
        )
}
