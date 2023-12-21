use std::sync::Arc;
use axum::body::Body;
use axum::extract::State;
use axum::http::{StatusCode, Response};
use axum::Json;
use axum::response::{IntoResponse};
use serde_json::json;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio::fs::File;
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

async fn fetch_all_records(
    db: &sqlx::PgPool,
) -> Result<Vec<Record>, (StatusCode, Json<serde_json::Value>)> {
    sqlx::query_as!(
        Record,
        "SELECT * FROM \"records\""
    )
        .fetch_all(db)
        .await
        .map_err(|e| {
            let error_response = json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })
}

pub async fn get_all_records(
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let records: Vec<Record> = fetch_all_records(&data.db).await?;

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

fn handle_xlsx_error<T>(result: Result<T, xlsxwriter::XlsxError>) -> Result<T, (StatusCode, Json<serde_json::Value>)> {
    match result {
        Ok(val) => Ok(val),
        Err(e) => {
            let error_response = json!({
                "status": "fail",
                "message": format!("Excel error: {}", e),
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

pub async fn create_excel_all_record(
    State(data): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let records: Vec<Record> = fetch_all_records(&data.db).await?;

    let wb = handle_xlsx_error(xlsxwriter::Workbook::new("/tmp/records.xlsx"))?;

    let mut sheet = handle_xlsx_error(wb.add_worksheet(None))?;

    let headers = &["Id", "Created At", "Updated At", "Last Updated By", "First Name", "Last Name", "MI", "Course", "Year Level", "Payment For", "Amount", "Received By"];

    for (i, header) in headers.iter().enumerate() {
        sheet.write_string(0, i as u16, header, None)
            .map_err(|e| {
                let error_response = json!({
                    "status": "fail",
                    "message": format!("Excel error: {}", e)
                });

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
    }

    for (i, record) in records.iter().enumerate() {
        let row = (i + 1) as u32; // Start writing data from the second row

        let mut write_to_sheet = |col: u16, data: &str| -> Result<(), (StatusCode, Json<serde_json::Value>)> {
            sheet.write_string(row, col, data, None).map_err(|e| {
                let error_response = json!({
                "status": "fail",
                "message": format!("Excel error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })
        };

        write_to_sheet(0, &record.id.to_string())?;
        write_to_sheet(1, &record.created_at.unwrap().to_string())?;
        write_to_sheet(2, &record.updated_at.unwrap().to_string())?;
        write_to_sheet(3, &record.last_updated_by)?;
        write_to_sheet(4, &record.first_name)?;
        write_to_sheet(5, &record.last_name)?;
        write_to_sheet(6, &record.mi)?;
        write_to_sheet(7, &record.course)?;
        write_to_sheet(8, &record.year_level)?;
        write_to_sheet(9, &record.payment_for)?;
        write_to_sheet(10, &record.amount)?;
        write_to_sheet(11, &record.received_by)?;
    }

    let close_result = wb.close();
    if let Err(e) = close_result {
        let error_response = json!({
            "status": "fail",
            "message": format!("Excel error: {}", e),
        });
        return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    };

    let response = json!({
        "status": "success",
        "message": "successfully created excel file"
    });

    Ok(Json(response))
}

pub async fn download_excel_handler() -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let file = File::open("/tmp/records.xlsx").await.map_err(|e| {
        let error_response = json!({
            "status": "fail",
            "message": format!("File error: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })?;

    let stream = FramedRead::new(file, BytesCodec::new());

    let body = Body::from_stream(stream);

    let response = Response::builder()
        .header("Content-Type", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
        .header("Content-Disposition", "attachment; filename=records.xlsx")
        .body(body)
        .unwrap();

    Ok(response)
}