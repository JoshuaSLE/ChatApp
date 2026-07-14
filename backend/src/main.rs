use axum::Router;
use dotenvy::dotenv;
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::{PgPool, migrate, postgres::PgPoolOptions};
use std::env;
use tokio::{net::TcpListener, signal};
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::routes::app_routes;

mod errors;
mod models;
mod routes;
mod tokens;
mod utils;

#[derive(Clone)]
pub struct AppState {
    pool: PgPool,
    cookie_secure: bool,
    jwt_encoding_key: EncodingKey,
    jwt_decoding_key: DecodingKey,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // INFO: Load in all the environment values
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cookie_secure = env::var("COOKIE_SECURE")
        .map(|v| v == "true")
        .unwrap_or(true);
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid number");
    let address = format!("{host}:{port}");
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let secret_bytes = hex::decode(secret).expect("Invalid hex in JWT_SECRET");

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

    let listener = TcpListener::bind(&address)
        .await
        .expect("Unable to bind the port");

    info!("Server is listening on http://{}", address);

    let app = Router::new()
        .nest("/api", app_routes())
        .with_state(AppState {
            pool,
            cookie_secure,
            jwt_encoding_key: EncodingKey::from_secret(&secret_bytes),
            jwt_decoding_key: DecodingKey::from_secret(&secret_bytes),
        });

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
