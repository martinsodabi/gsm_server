use std::{collections::HashMap, i32};

use axum::{extract::State, Extension, Json};
use axum_macros::debug_handler;
use libsql::Value as DBV;
use serde_json::from_value;
use tracing::warn;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    models::{
        profile::{get_profile_by_user_id, Profile},
        user::User,
        util::{query_get_one, row_to_value_map},
    },
    utils::app_error::AppError,
    views::profile::ProfileResponse,
};

pub async fn get_profile(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
) -> Result<Json<ProfileResponse>, AppError> {
    let user_id = user.id.ok_or(AppError::NotFound)?;
    let db_conn = app_state.db_conn;

    let profile = get_profile_by_user_id(user_id, &db_conn).await?;

    // TODO: Abstract this into a function!
    let pid: Option<Uuid> = match profile.pid {
        Some(value) => Uuid::from_slice(value.as_slice()).ok(),
        _ => None,
    };

    return Ok(Json(ProfileResponse {
        pid,
        first_name: profile.first_name,
        last_name: profile.last_name,
        location: profile.location,
        birth_date: profile.birth_date,
        is_visible: profile.is_visible,
    }));
}

#[debug_handler]
pub async fn update_profile(
    State(app_state): State<AppState>,
    Extension(user): Extension<User>,
    Json(params): Json<serde_json::Value>,
) -> Result<Json<ProfileResponse>, AppError> {
    // TODO: Return an appropriate error!
    let user_id = user.id.ok_or(AppError::NotFound)?;

    let mut query_map: HashMap<&str, DBV> = HashMap::new();

    if let Some(location) = params.get("location") {
        if let Some(location) = from_value::<String>(location.to_owned()).ok() {
            // TODO: Check if location exist in a location_map!
            if location.is_empty() || location.len() > 255 {
                warn!("From location condition");
                return Err(AppError::WrongCredential);
            }

            query_map.insert(
                "location",
                DBV::from(location.trim().to_uppercase().as_str()),
            );
        }
    }

    if let Some(first_name) = params.get("first_name") {
        if let Some(first_name) = from_value::<String>(first_name.to_owned()).ok() {
            if first_name.is_empty() || first_name.len() > 24 {
                warn!("From first_name condition");
                return Err(AppError::WrongCredential);
            }

            query_map.insert("first_name", DBV::from(first_name.trim()));
        }
    }

    if let Some(last_name) = params.get("last_name") {
        if let Some(last_name) = from_value::<String>(last_name.to_owned()).ok() {
            if last_name.is_empty() || last_name.len() > 24 {
                warn!("From last_name condition");
                return Err(AppError::WrongCredential);
            }

            query_map.insert("last_name", DBV::from(last_name.trim()));
        }
    }

    if let Some(is_visible) = params.get("is_visible") {
        if let Some(is_visible) = from_value::<bool>(is_visible.to_owned()).ok() {
            // TODO: Query to check if this value can be updated safely!
            if is_visible {
                query_map.insert("is_visible", DBV::from(1));
            } else {
                query_map.insert("is_visible", DBV::from(0));
            }
        }
    }

    if let Some(birth_date) = params.get("birth_date") {
        if let Some(birth_date) = from_value::<i64>(birth_date.to_owned()).ok() {
            // Typically 18 years ago
            let youngest_birth_date: i64 = 1136098230;

            // i64 not supported by libsql crate for now...
            // so more than 121 years ago is the limit!
            let oldest_birth_date: i64 = i32::MIN.into();

            if youngest_birth_date < birth_date || oldest_birth_date > birth_date {
                warn!("From birth_date condition");
                return Err(AppError::WrongCredential);
            }

            query_map.insert("birth_date", DBV::from(birth_date as i32));
        }
    }

    let mut query_statement: String = "UPDATE profiles SET".to_string();
    let mut query_args: Vec<DBV> = Vec::new();

    for (k, v) in query_map {
        query_statement.push_str(" ");
        query_statement.push_str(k);
        query_statement.push_str(" = ?,");
        query_args.push(v);
    }

    if query_args.is_empty() {
        warn!("From query_args condition");
        return Err(AppError::WrongCredential);
    }

    query_statement.pop();
    query_statement.push_str(" WHERE user_id = ? RETURNING *");
    query_args.push(DBV::from(user_id as i32));

    let db_conn = app_state.db_conn;
    let row = query_get_one(query_statement.as_str(), query_args, &db_conn).await?;

    let profile = Profile::from(row_to_value_map(row));
    let pid = Uuid::from_slice(profile.pid.unwrap_or(Vec::new()).as_slice()).ok();

    Ok(Json(ProfileResponse {
        pid,
        first_name: profile.first_name,
        last_name: profile.last_name,
        location: profile.location,
        birth_date: profile.birth_date,
        is_visible: profile.is_visible,
    }))
}
