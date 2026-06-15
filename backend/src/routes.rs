use axum::Router;

use crate::AppState;

pub fn app_routes() -> Router<AppState> {
    Router::new()
}
