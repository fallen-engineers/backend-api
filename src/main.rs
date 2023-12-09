mod handlers;
mod routes;
use sqlx::postgres::PgPoolOptions;
use crate::routes::routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let port = dotenv::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    let database_url = dotenv::var("DATABASE_URL").expect("missing DATABASE_URL env");

    println!("Connecting to db...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Could not connect to database");

    println!("{}", format!("starting server at port {}", port));

    let app = routes(pool);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("Server started at http://localhost:{}", port);

    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}