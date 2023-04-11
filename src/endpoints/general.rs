use rocket::get;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use sqlx::PgPool;

pub struct ApiState {
    pub pool: PgPool,
}

#[derive(Debug, Serialize)]
pub struct SuccessFailResponse {
    pub success: bool,
}

#[get("/")]
pub fn index() -> Json<SuccessFailResponse> {
    Json(SuccessFailResponse { success: true })
}
