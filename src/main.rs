mod handlers;
mod routes;
mod config;
mod model;
mod response;
mod jwt_auth;

use std::sync::Arc;
use axum::http::{HeaderValue, Method};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use sqlx::{Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use crate::config::Config;
use crate::routes::create_router;

pub struct AppState {
    db: Pool<Postgres>,
    env: Config,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let config = Config::init();
    let addr = format!("0.0.0.0:{}", config.port);

    println!("Connecting to db...");

    let pool = match PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("Could not connect to database {:?}", err);
            std::process::exit(1);
        }
    };

    println!("{}", format!("starting server at port {}", config.port));

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState {
        db: pool.clone(),
        env: config.clone(),
    }))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 Server started at http://localhost:{}", config.port);

    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}