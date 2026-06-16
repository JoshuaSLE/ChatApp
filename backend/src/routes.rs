use axum::{
    Json, Router,
    routing::{get, post},
};
use serde::Serialize;

use crate::AppState;

mod auth;

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
}

fn auth_routes() -> Router<AppState> {
    Router::new().route("/register", post(auth::register))
}
