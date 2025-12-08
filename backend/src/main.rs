use backend::run_app;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server...");
    backend::run_app().await
}
