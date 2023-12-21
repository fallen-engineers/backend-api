use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;
use crate::AppState;
use crate::record_model::{CreateRecordSchema, DeleteRecordSchema, Record, UpdateRecordSchema};

pub async fn create_record_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CreateRecordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let record = sqlx::query_as!(
        Record,
        "INSERT INTO \"records\" (last_updated_by,first_name,last_name,mi,course,year_level,payment_for,amount,received_by) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING *",
        body.last_updated_by.to_string(),
        body.first_name.to_string(),
        body.last_name.to_string(),
        body.mi.to_string(),
        body.course.to_string(),
        body.year_level.to_string(),
        body.payment_for.to_string(),
        body.amount,
        body.received_by.to_string()
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

    let record_response = json!({"status": "success", "data": json!({
        "record": record
    })});

    Ok(Json(record_response))
}

pub async fn get_all_records(
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let records: Vec<Record> = sqlx::query_as!(
        Record,
        "SELECT * FROM \"records\""
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

    let json_response = json!({
        "status": "success",
        "data": json!({
            "records": records
        })
    });

    Ok(Json(json_response))
}

pub async fn update_record_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateRecordSchema>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let record = sqlx::query_as!(
        Record,
        "UPDATE \"records\" SET last_updated_by = $1, first_name = $2, last_name = $3, mi = $4, course = $5, year_level = $6, payment_for = $7, amount = $8, received_by = $9, updated_at = NOW() WHERE id = $10 RETURNING *",
        body.last_updated_by.to_string(),
        body.first_name.to_string(),
        body.last_name.to_string(),
        body.mi.to_string(),
        body.course.to_string(),
        body.year_level.to_string(),
        body.payment_for.to_string(),
        body.amount,
        body.received_by.to_string(),
        body.id
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

    let record_response = json!({"status": "success", "data": json!({
        "record": record
    })});

    Ok(Json(record_response))
}

pub async fn delete_record_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<DeleteRecordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let rows_deleted = sqlx::query!(
        "DELETE FROM \"records\" WHERE id = $1",
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

    if rows_deleted.rows_affected() == 0 {
        let error_response = json!({
            "status": "fail",
            "message": "No record found with the provided ID"
        });

        return Err((StatusCode::NOT_FOUND, Json(error_response)))
    }

    let response = json!({
        "status": "success",
        "message": "Record Successfully Deleted"
    });

    Ok(Json(response))
}