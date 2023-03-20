mod api;

#[shuttle_service::main]
pub async fn rocket() -> shuttle_service::ShuttleRocket {
    let rocket = api::build_api();

    Ok(rocket)
}
