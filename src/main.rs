pub mod app_state;
pub mod config;
pub mod controllers;
pub mod middlewares;
pub mod migrations;
pub mod models;
pub mod utils;
pub mod views;

use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use axum_macros::debug_handler;
use config::initialize_database;
use controllers::{
    auth::login,
    auth::register_user,
    profile::{get_profile, update_profile},
};
use models::{profile::CacheProfile, user::CacheUser};
use serde_json::json;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    app_state::AppState, config::Config, middlewares::jwt_auth::authenticate, models::user::User,
};

async fn check_server_health() -> impl IntoResponse {
    return (StatusCode::OK, "Server is healthy!".to_string());
}

#[debug_handler]
async fn check_db_health(State(AppState { db_conn, .. }): State<AppState>) -> impl IntoResponse {
    match db_conn
        .query("SELECT COUNT(*) FROM sqlite_schema", ())
        .await
    {
        Ok(res) => (
            StatusCode::OK,
            format!("Database exist {}", res.column_count().to_string()),
        ),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

    let config = Config::init();
    let mut can_use_local_db: bool = false;
    let env_args = std::env::args().collect::<Vec<String>>();

    if env_args.len() > 1 {
        if env_args[1] == "local_db" {
            can_use_local_db = true;
        }
    }

    let db_conn = initialize_database(&config, can_use_local_db);
    let profile_cache: Arc<Mutex<HashMap<Uuid, CacheProfile>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let user_cache: Arc<Mutex<HashMap<Uuid, CacheUser>>> = Arc::new(Mutex::new(HashMap::new()));

    let app_state = AppState {
        config,
        db_conn,
        profile_cache,
        user_cache,
    };

    let router = Router::new()
        .route("/api/update_profile", post(update_profile))
        .route("/api/get_profile", get(get_profile))
        .route("/check_auth", get(check_auth_route))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            authenticate,
        ))
        .route("/api/login", post(login))
        .route("/api/register", post(register_user))
        .route("/server_health", get(check_server_health))
        .route("/db_health", get(check_db_health))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("[::]:8080").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

#[debug_handler]
pub async fn check_auth_route(
    Extension(user): Extension<User>,
) -> Result<impl IntoResponse, StatusCode> {
    let json_response = json!({
        "status": "success",
        "message": "Auth is working",
        "user_email": user.email,
    });

    return Ok(Json(json_response));
}
