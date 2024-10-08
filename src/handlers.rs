use std::sync::Arc;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{extract::State, http::{header, Response, StatusCode}, response::IntoResponse, Json, Extension};
use axum_extra::extract::cookie::{Cookie, SameSite};
use jsonwebtoken::{encode, EncodingKey, Header};
use rand_core::OsRng;
use serde_json::json;

use crate::{
    model::{LoginUserSchema, RegisterUserSchema, TokenClaims, User},
    response::FilteredUser,
    AppState,
};
use crate::model::ChangeRoleSchema;

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "RUST UCLM COOP API";

    let json_response = json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

fn filter_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        username: user.username.to_owned(),
        name: user.name.to_owned(),
        photo: user.photo.clone(),
        role: user.role.to_owned(),
        createdAt: user.created_at.unwrap(),
        updatedAt: user.updated_at.unwrap(),
    }
}

pub async fn register_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user_exists: Option<bool> =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM \"users\" WHERE email = $1 OR username = $2)")
            .bind(body.email.to_owned().to_ascii_lowercase())
            .bind(body.username.to_owned().to_ascii_lowercase())
            .fetch_one(&data.db)
            .await
            .map_err(|e| {
                let error_response = json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;

    if let Some(exists) = user_exists {
        if exists {
            let error_response = json!({
                "status": "fail",
                "message": "User with that email or username already exists",
            });
            return Err((StatusCode::CONFLICT, Json(error_response)));
        }
    }

    if let Some(role) = &body.role {
        if role != "admin" && role != "non_admin" {
            let error_response = json!({
                "status": "fail",
                "message": "Role must either be 'admin' or 'non_admin'"
            });
            return Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            let error_response = json!({
                "status": "fail",
                "message": format!("Error while hashing password: {}", e)
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
        .map(|hash| hash.to_string())?;

    let user = sqlx::query_as!(
        User,
        "INSERT INTO \"users\" (name,email,password,username,role) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        body.name.to_string(),
        body.email.to_string().to_ascii_lowercase(),
        hashed_password,
        body.username.to_string().to_ascii_lowercase(),
        body.role.unwrap_or_else(|| "non_admin".to_string()).to_ascii_lowercase()
    )
        .fetch_one(&data.db)
        .await
        .map_err(|e| {
            let error_response = json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let user_response = json!({"status": "success", "data": json!({
        "user": filter_user_record(&user)
    })});

    Ok(Json(user_response))
}

pub async fn login_user_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM \"users\" WHERE email = $1 OR username = $1",
        body.email_or_username.to_ascii_lowercase()
    )
        .fetch_optional(&data.db)
        .await
        .map_err(|e| {
            let error_response = json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });

            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?
        .ok_or_else(|| {
            let error_response = json!({
                "status": "fail",
                "message": "Invalid email or password"
            });
            (StatusCode::BAD_REQUEST, Json(error_response))
        })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false ,|_| true),

        Err(_) => false,
    };

    if !is_valid {
        let error_response = json!({
            "status": "fail",
            "message": "Invalid email or password"
        });

        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    let now = chrono::Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + chrono::Duration::minutes(60)).timestamp() as usize;
    let claims: TokenClaims = TokenClaims {
        sub: user.id.to_string(),
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(data.env.jwt_secret.as_ref()),
    )
        .unwrap();

    let cookie = Cookie::build(("token", token.to_owned()))
        .path("/")
        .max_age(time::Duration::hours(1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let mut response = Response::new(json!({"status": "success", "token": token}).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    Ok(response)
}

pub async fn logout_handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)>{
    let cookie = Cookie::build(("token", ""))
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true);

    let mut response = Response::new(json!({"status": "success"}).to_string());

    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok(response)
}

pub async fn get_me_handler(
    Extension(user): Extension<User>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let json_response = json!({
        "status": "success",
        "data": serde_json::json!({
            "user": filter_user_record(&user)
        })
    });

    Ok(Json(json_response))
}

pub async fn get_all_users_handler(
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let users: Vec<User> = sqlx::query_as!(
        User,
        "SELECT * FROM \"users\""
    )
        .fetch_all(&data.db)
        .await
        .map_err(|e| {
            let error_response = json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });

            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let filtered_users: Vec<FilteredUser> = users.into_iter().map(|user| {filter_user_record(&user)}).collect();

    let json_response = json!({
        "status": "success",
        "data": json!({
            "users": filtered_users
        })
    });

    Ok(Json(json_response))
}

pub async fn change_role_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<ChangeRoleSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Validate role
    if body.role != "admin" && body.role != "non_admin" {
        let error_response = json!({
            "status": "fail",
            "message": "Role must be either 'admin' or 'non_admin'",
        });
        return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // Update user role in database
    let updated_rows = sqlx::query!(
        "UPDATE \"users\" SET role = $1 WHERE id = $2",
        body.role,
        body.id
    )
        .execute(&data.db)
        .await
        .map_err(|e| {
            let error_response = json!({
            "status": "fail",
            "message": format!("Database error: {}", e),
        });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    if updated_rows.rows_affected() == 0 {
        let error_response = json!({
            "status": "fail",
            "message": "No user found with the provided ID",
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }

    let success_response = json!({
        "status": "success",
        "message": "User role updated successfully",
    });

    Ok(Json(success_response))
}