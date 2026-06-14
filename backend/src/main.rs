use axum::{Router, routing::get};
use dotenvy::{dotenv, var};
use sqlx::{migrate, postgres::PgPoolOptions};
use tokio::{net::TcpListener, signal};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Unable to connect to the database");

    migrate!()
        .run(&pool)
        .await
        .expect("Failed to run the migration");

    info!("Database migrated and ready!");

    let host = var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

    let port = var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid number");

    let address = format!("{host}:{port}");

    let listener = TcpListener::bind(&address)
        .await
        .expect("Unable to bind the port");

    info!("Server is listening on http://{}", address);

    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to run the server");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Server gracefully shutting down!")
}
