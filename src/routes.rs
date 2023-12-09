use axum::routing::{get, post, put, delete, Router};
use sqlx::{Pool, Postgres};
use crate::handlers;

pub fn routes(pool: Pool<Postgres>) -> Router {
    Router::new()
        .route("/", get(handlers::health))
        .route("/user", post(handlers::create_user))
        .route("/user", get(handlers::read_user))
        .route("/user/:id", put(handlers::update_user))
        .route("/user/:id", delete(handlers::delete_user))
        .with_state(pool)
}