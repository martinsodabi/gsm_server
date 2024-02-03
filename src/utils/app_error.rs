use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    InvalidToken,
    WrongCredential,
    MissingCredential,
    TokenCreation,
    Unauthorized,
    InternalServerError,
    UserDoesNotExist,
    UserAlreadyExist,
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, err_msg) = match self {
            Self::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid Token"),
            Self::WrongCredential => (StatusCode::NOT_ACCEPTABLE, "Wrong Credentials"),
            Self::MissingCredential => (StatusCode::NOT_ACCEPTABLE, "Missing Credentials"),
            Self::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create Token"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized Request"),
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::UserDoesNotExist => (StatusCode::NOT_FOUND, "User does not Exist"),
            Self::UserAlreadyExist => (StatusCode::CONFLICT, "User already exist"),
            Self::NotFound => (StatusCode::NOT_FOUND, "Could not find resource"),
        };
        return (status, Json(json!({ "error": err_msg}))).into_response();
    }
}
