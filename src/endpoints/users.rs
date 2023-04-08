use super::general::ApiState;
use super::general::SuccessFailResponse;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use sqlx::{Error, FromRow};
use rocket::State;

#[derive(Debug, FromRow, Serialize)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct NewUserIdResponse {
    pub id: i32,
}

#[get("/user/<id>")]
pub async fn get_user(id: i32, state: &State<ApiState>) -> Json<Option<User>> {
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
pub async fn get_all_users(state: &State<ApiState>) -> Json<Vec<User>> {
    let result = sqlx::query_as::<_, User>("select id, name from users")
        .fetch_all(&state.pool)
        .await;

    match result {
        Ok(all_users) => Json(all_users),
        Err(_) => Json(Vec::<User>::new()),
    }
}

#[derive(Debug, Deserialize)]
pub struct NewUser {
    pub name: String,
    pub password: String,
}

#[post("/user", data = "<user>")]
pub async fn insert_user(user: Json<NewUser>, state: &State<ApiState>) -> Json<NewUserIdResponse> {
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

#[derive(Debug, Deserialize)]
pub struct UpdateUserBody {
    pub name: String,
}

#[post("/user/<id>/update", data = "<user>")]
pub async fn update_user(
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

#[post("/user/<id>/delete")]
pub async fn delete_user(id: i32, state: &State<ApiState>) -> Json<SuccessFailResponse> {
    let result = sqlx::query("delete from users where id=$1")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(_) => Json(SuccessFailResponse { success: true }),
        Err(_) => Json(SuccessFailResponse { success: false }),
    }
}