use chrono::prelude::*;
use serde::Serialize;

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
pub struct FilteredUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub photo: Option<String>,
    pub createdAt: DateTime<Utc>,
    pub updatedAt: DateTime<Utc>
}

#[derive(Debug, Serialize)]
pub struct UserData {
    pub user: FilteredUser
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub status: String,
    pub data: UserData
}