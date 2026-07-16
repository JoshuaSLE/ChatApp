use crate::AppState;
use axum::{
    Json, Router,
    routing::{delete, get, patch, post},
};
use serde::Serialize;

mod auth;
mod message;
mod room;
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
            "/health",
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
        .nest("/rooms", room_routes())
}

fn auth_routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(auth::login))
        .route("/refresh", post(auth::refresh))
        .route("/logout", post(auth::logout))
}

fn user_routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(user::register))
        .route("/update", patch(user::update))
        .route("/delete", delete(user::delete))
        .route("/me", get(user::me))
        .route("/search", get(user::search))
        .route("/status", get(user::status))
}

fn room_routes() -> Router<AppState> {
    Router::new()
        .route("/", post(room::create).get(room::list))
        .route("/update/{room_id}", patch(room::update))
        .route("/delete/{room_id}", delete(room::delete))
        .route("/me/{room_id}", get(room::me))
        .nest("/{room_id}/messages", message_routes())
}

fn message_routes() -> Router<AppState> {
    Router::new().route("/", post(message::create).get(message::get))
}
