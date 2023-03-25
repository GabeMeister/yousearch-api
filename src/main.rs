#[macro_use]
extern crate rocket;
mod cors;
use cors::CORS;
use rocket::serde::json::Json;
use rocket::{response::status::BadRequest, State};
use serde::{Deserialize, Serialize};
use shuttle_runtime::CustomError;
use shuttle_secrets::SecretStore;
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

struct Secrets {
    pub db_pass: String,
    pub gabe: String,
}

struct YousearchState {
    pool: PgPool,
    secrets: Secrets,
}

#[get("/")]
fn index() -> &'static str {
    "It's working!"
}

#[get("/hello")]
fn hello(state: &State<YousearchState>) -> String {
    let gabe_var = state.secrets.gabe.clone();
    dbg!(&gabe_var);
    dbg!(&state.secrets.db_pass);

    gabe_var
}

#[get("/todo/<id>")]
async fn retrieve(
    id: i32,
    state: &State<YousearchState>,
) -> Result<Json<Todo>, BadRequest<String>> {
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
    state: &State<YousearchState>,
) -> Result<Json<Todo>, BadRequest<String>> {
    let todo: Todo = sqlx::query_as("insert into todos (note) values ($1) returning id, note")
        .bind(&data.note)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| BadRequest(Some(e.to_string())))?;

    Ok(Json(todo))
}

#[shuttle_runtime::main]
async fn rocket(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:{secrets.DB_PASS}@localhost:5432/yousearch"
    )]
    pool: PgPool,
    #[shuttle_secrets::Secrets] secret_store: SecretStore,
) -> shuttle_rocket::ShuttleRocket {
    pool.execute(include_str!("../schema.sql"))
        .await
        .map_err(CustomError::new)?;

    let state = YousearchState {
        pool,
        secrets: Secrets {
            db_pass: secret_store.get("DB_PASS").unwrap(),
            gabe: secret_store.get("GABE").unwrap(),
        },
    };

    let rocket = rocket::build()
        .mount("/", routes![index, hello, add, retrieve])
        .manage(state)
        .attach(CORS);

    Ok(rocket.into())
}
