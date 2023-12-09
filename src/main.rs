mod handlers;
use axum::routing::{get, post, put, delete, Router};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let port = dotenv::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let database_url = dotenv::var("DATABASE_URL").expect("missing DATABASE_URL env");

    println!("connecting to database");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Could not connect to database");

    println!("{}", format!("starting server at port {}", port));

    let app = Router::new()
        .route("/", get(handlers::health))
        .route("/user", post(handlers::create_user))
        .route("/user", get(handlers::read_user))
        .route("/user/:id", put(handlers::update_user))
        .route("/user/:id", delete(handlers::delete_user))
        .with_state(pool);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Server started at http://localhost:{}", port);

    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}