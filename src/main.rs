use pretiola::startup::run;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let default_bind = if std::env::var("APP_ENV").as_deref() == Ok("production") {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    };
    let host = std::env::var("BIND_ADDR").unwrap_or_else(|_| default_bind.into());
    let addr = format!("{}:{}", host, port);
    let listener =
        TcpListener::bind(&addr).unwrap_or_else(|_| panic!("Failed to bind port {}", port));
    log::info!("Server started on port {}", port);
    run(listener)?.await
}
