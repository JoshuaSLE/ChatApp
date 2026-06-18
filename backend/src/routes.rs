use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Serialize;

use crate::AppState;

mod auth;
mod user;

#[derive(Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    status: &'static str,
}

pub fn app_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(|| async {
                Json(AppInfo {
                    name: env!("CARGO_PKG_NAME"),
                    version: env!("CARGO_PKG_VERSION"),
                    status: "healthy",
                })
            }),
        )
        .nest("/auth", auth_routes())
        .nest("/user", user_routes())
}

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .route("/logout", post(auth::logout))
}

fn user_routes() -> Router<AppState> {
    Router::new().route("/register", post(user::register))
}
