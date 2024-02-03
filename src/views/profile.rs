use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ProfileParams {
    pub user_id: i64, //Get this value from auth token
    pub birth_date: i64,
    pub first_name: String,
    pub last_name: String,
    pub location: String,
    pub is_visible: bool,
}

#[derive(Debug, Serialize)]
pub struct ProfileResponse {
    pub pid: Option<Uuid>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub location: Option<String>,
    pub birth_date: Option<i64>,
    pub is_visible: Option<bool>,
}
