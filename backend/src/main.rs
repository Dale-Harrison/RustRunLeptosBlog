use backend::run_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run_app().await
}
