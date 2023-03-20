mod api;

#[rocket::main]
async fn main() {
    let _ = api::build_api().launch().await;
}
