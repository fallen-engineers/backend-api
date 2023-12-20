use std::sync::Arc;
use axum::middleware;
use axum::routing::{get, post, Router, put};
use crate::handlers;
use crate::AppState;
use crate::handlers::{get_me_handler, logout_handler};
use crate::jwt_auth::{admin_auth, auth};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(handlers::health_checker_handler))
        .route("/api/auth/register", post(handlers::register_user_handler))
        .route("/api/auth/login", post(handlers::login_user_handler))
        .route("/api/users/all", get(handlers::get_all_users_handler))
        .route(
            "/api/users/change_role",
            put(handlers::change_role_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), admin_auth))
        )
        .route(
            "/api/auth/logout",
            get(logout_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
        )
        .route(
            "/api/users/me",
            get(get_me_handler)
                .route_layer(middleware::from_fn_with_state(app_state.clone(), auth))
        )
        .with_state(app_state)
}
