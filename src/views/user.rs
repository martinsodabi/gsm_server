use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RegisterParams {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: i64,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub birth_date: Option<i64>,
    pub location: Option<String>,
    pub is_visible: Option<bool>,
    pub email: Option<String>,
    pub auth_token: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginParams {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub email: Option<String>,
    pub auth_token: Option<String>,
}
