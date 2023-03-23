#[macro_use]
extern crate rocket;
mod cors;
use cors::CORS;
use rocket::serde::json::Json;
use rocket::{response::status::BadRequest, State};
use serde::{Deserialize, Serialize};
use shuttle_runtime::CustomError;
use sqlx::{Executor, FromRow, PgPool};

#[derive(Serialize, FromRow)]
struct Todo {
    pub id: i32,
    pub note: String,
}

#[derive(Deserialize)]
struct TodoNew {
    pub note: String,
}

struct MyState {
    pool: PgPool,
}

#[get("/")]
fn index() -> &'static str {
    "It's working!"
}

#[get("/hello")]
fn hello() -> &'static str {
    "Hello world!"
}

#[get("/todo/<id>")]
async fn retrieve(id: i32, state: &State<MyState>) -> Result<Json<Todo>, BadRequest<String>> {
    let todo: Todo = sqlx::query_as("select * from todos where id = $1")
        .bind(id)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(todo))
}

#[post("/todo", data = "<data>")]
async fn add(
    data: Json<TodoNew>,
    state: &State<MyState>,
) -> Result<Json<Todo>, BadRequest<String>> {
    let todo: Todo = sqlx::query_as("insert into todos (note) values ($1) returning id, note")
        .bind(&data.note)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(todo))
}

#[shuttle_runtime::main]
async fn rocket(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_rocket::ShuttleRocket {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let state = MyState { pool };

    let rocket = rocket::build()
        .mount("/", routes![index, hello, add, retrieve])
        .manage(state)
        .attach(CORS);

    Ok(rocket.into())
}
