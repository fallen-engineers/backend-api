use axum::{extract, http};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct User {
    id: uuid::Uuid,
    name: String,
    email: String,
    inserted_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    fn new(name: String, email: String) -> Self {
        let now = chrono::Utc::now();

        Self {
            id: uuid::Uuid::new_v4(),
            name,
            email,
            inserted_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    name: String,
    email: String,
}

pub async fn health() -> http::StatusCode {
    http::StatusCode::OK
}

pub async fn create_user(
    extract::State(pool): extract::State<PgPool>,
    axum::Json(payload): axum::Json<CreateUser>,
) -> Result<(http::StatusCode, axum::Json<User>), http::StatusCode> {
    let user = User::new(payload.name, payload.email);

    let res = sqlx::query(
        r#"
        INSERT INTO "user" (id, name, email, inserted_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        "#
    )
        .bind(&user.id)
        .bind(&user.name)
        .bind(&user.email)
        .bind(&user.inserted_at)
        .bind(&user.updated_at)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => Ok((http::StatusCode::CREATED, axum::Json(user))),
        Err(_) => Err(http::StatusCode::INTERNAL_SERVER_ERROR)
    }
}