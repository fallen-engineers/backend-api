use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, FromRow, Serialize, Clone)]
pub struct Record {
    pub id: i32,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
    pub last_updated_by:String,
    pub first_name: String,
    pub last_name: String,
    pub mi: String,
    pub course: String,
    pub year_level: String,
    pub payment_for: String,
    pub amount: String,
    pub received_by: String,
}
#[derive(Debug, Deserialize)]
pub struct CreateRecordSchema {
    pub last_updated_by: String,
    pub first_name: String,
    pub last_name: String,
    pub mi:String,
    pub course: String,
    pub year_level: String,
    pub payment_for: String,
    pub amount: String,
    pub received_by: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRecordSchema {
    pub id: i32,
    pub last_updated_by: String,
    pub first_name: String,
    pub last_name: String,
    pub mi:String,
    pub course: String,
    pub year_level: String,
    pub payment_for: String,
    pub amount: String,
    pub received_by: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteRecordSchema {
    pub id: i32
}