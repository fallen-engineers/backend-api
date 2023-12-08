use axum::{extract, http};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(Serialize, FromRow)]
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

pub async fn read_user(
    extract::State(pool): extract::State<PgPool>,
) -> Result<axum::Json<Vec<User>>, http::StatusCode> {
    let query_string: String = "SELECT * FROM \"user\";".to_string();
    let res = sqlx::query_as::<_, User>(&query_string)
        .fetch_all(&pool)
        .await;


    match res {
        Ok(user) => Ok(axum::Json(user)),
        Err(e) => {
            println!("{}", e.to_string());
            Err(http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn update_user(
    extract::State(pool): extract::State<PgPool>,
    extract::Path(id): extract::Path<uuid::Uuid>,
    axum::Json(payload): axum::Json<CreateUser>
) -> http::StatusCode {
    let now = chrono::Utc::now();

    let res = sqlx::query(
        r#"
        UPDATE "user"
        SET name = $1, email = $2, updated_at = $3
        WHERE id = $4
        "#,
    )
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(&now)
        .bind(&id)
        .execute(&pool)
        .await
        .map(|res| match res.rows_affected() {
            0 => http::StatusCode::NOT_FOUND,
            _ => http::StatusCode::OK,
        });

    match res {
        Ok(status) => status,
        Err(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}