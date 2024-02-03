use crate::{
    utils::app_error::AppError,
    views::profile::{ProfileParams, ProfileResponse},
};

use super::util::{self, i64_from_value, query_get_many, query_get_one, row_to_value_map};
use libsql::{Connection, Value as DBV};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::error;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    pub id: Option<i64>,
    pub pid: Option<Vec<u8>>,
    pub user_id: Option<i64>,
    pub birth_date: Option<i64>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub profile_video_id: Option<i64>,
    pub location: Option<String>,
    pub is_visible: Option<bool>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl From<HashMap<String, libsql::Value>> for Profile {
    fn from(value_map: HashMap<String, libsql::Value>) -> Self {
        Self {
            id: util::i64_from_value("id", &value_map),
            pid: util::byte_from_value("pid", &value_map),
            user_id: util::i64_from_value("user_id", &value_map),
            birth_date: util::i64_from_value("birth_date", &value_map),
            first_name: util::string_from_value("first_name", &value_map),
            last_name: util::string_from_value("last_name", &value_map),
            profile_video_id: util::i64_from_value("profile_video_id", &value_map),
            location: util::string_from_value("location", &value_map),
            is_visible: util::bool_from_value("is_visible", &value_map),
            created_at: util::i64_from_value("created_at", &value_map),
            updated_at: util::i64_from_value("updated_at", &value_map),
        }
    }
}

pub struct CacheProfile {
    pub id: i32,
    pub user_id: i32,
    pub birth_date: i64,
    pub first_name: String,
    pub last_name: String,
}

impl CacheProfile {
    pub fn from(db_profile: &Profile) -> Result<Self, AppError> {
        let id = db_profile.id.ok_or(AppError::InternalServerError)? as i32;
        let user_id = db_profile.user_id.ok_or(AppError::InternalServerError)? as i32;
        let birth_date = db_profile.birth_date.ok_or(AppError::InternalServerError)?;

        let first_name = db_profile
            .first_name
            .as_ref()
            .ok_or(AppError::InternalServerError)?
            .to_owned();

        let last_name = db_profile
            .last_name
            .as_ref()
            .ok_or(AppError::InternalServerError)?
            .to_owned();

        Ok(CacheProfile {
            id,
            user_id,
            birth_date,
            first_name,
            last_name,
        })
    }
}

//Production: This should only be called from auth::register_user
pub async fn create_profile(
    db_conn: &Connection,
    params: ProfileParams,
) -> Result<Profile, AppError> {
    let pid = Uuid::new_v4().as_bytes().to_vec();

    let query_statement =
        "INSERT INTO profiles (pid, user_id, birth_date, first_name, last_name) values (?, ?, ?, ?, ?) RETURNING *";

    let query_args = vec![
        DBV::from(pid),
        DBV::from(params.user_id as i32),
        DBV::from(params.birth_date as i32),
        DBV::from(params.first_name.as_str()),
        DBV::from(params.last_name.as_str()),
    ];

    let row = query_get_one(query_statement, query_args, db_conn)
        .await
        .map_err(|err| err)?;

    let value_map = row_to_value_map(row);
    let profile = Profile::from(value_map);

    Ok(profile)
}

pub async fn get_profile_id_by_user_id(
    user_id: i64,
    db_conn: &Connection,
) -> Result<i64, AppError> {
    let query_statement = "SELECT id FROM profiles WHERE user_id = ? LIMIT 1";
    let query_args = vec![DBV::from(user_id as i32)];

    let row = query_get_one(query_statement, query_args, db_conn).await?;

    let value_map = row_to_value_map(row);

    let id = i64_from_value("id", &value_map).ok_or(AppError::InternalServerError)?;

    return Ok(id);
}

pub async fn get_profile_by_pid_string(
    pid: &String,
    db_conn: &Connection,
) -> Result<Profile, AppError> {
    // TODO: Abstract this into a function!
    let pid = Uuid::try_parse(pid)
        .map_err(|err| {
            error!("{:?}", err);
            AppError::WrongCredential
        })?
        .as_bytes()
        .to_vec();

    let query_statement = "SELECT * FROM profiles WHERE pid = ? LIMIT 1";
    let query_args = vec![DBV::from(pid)];

    return get_profile(query_statement, query_args, db_conn).await;
}

pub async fn get_profile_by_pid_vec(
    pid: Vec<u8>,
    db_conn: &Connection,
) -> Result<Profile, AppError> {
    let query_statement = "SELECT * FROM profiles WHERE pid = ? LIMIT 1";
    let query_args = vec![DBV::from(pid)];

    return get_profile(query_statement, query_args, db_conn).await;
}

pub async fn get_profile_by_user_id(
    user_id: i64,
    db_conn: &Connection,
) -> Result<Profile, AppError> {
    let query_statement = "SELECT * FROM profiles WHERE user_id = ? LIMIT 1";
    let query_args = vec![DBV::from(user_id as i32)];

    return get_profile(query_statement, query_args, db_conn).await;
}

async fn get_profile(
    query_statement: &str,
    query_args: Vec<DBV>,
    db_conn: &Connection,
) -> Result<Profile, AppError> {
    let row = query_get_one(query_statement, query_args, db_conn).await?;

    let value_map = row_to_value_map(row);
    let profile = Profile::from(value_map);

    return Ok(profile);
}

pub async fn get_profiles_by_location(
    my_location: String,
    exclude_profile_id: i64,
    db_conn: &Connection,
) -> Result<Vec<ProfileResponse>, AppError> {
    // TODO: Attempt to get profiles from cache before querying the database!
    let query_statement = "SELECT * FROM profiles WHERE location = ? LIMIT 100";
    let query_args = vec![DBV::from(my_location.as_str())];

    let rows = query_get_many(query_statement, query_args, &db_conn).await?;

    return Ok(get_profiles_as_view(rows, exclude_profile_id));
}

pub fn get_profiles_as_view(
    mut rows: libsql::Rows,
    exclude_profile_id: i64,
) -> Vec<ProfileResponse> {
    let mut profiles: Vec<ProfileResponse> = Vec::new();

    loop {
        let next_row = rows.next();

        if next_row.is_err() {
            break;
        }

        // INFO: Can safely unwrap here because Ok() is guaranteed above!
        let row = if let Some(row) = next_row.unwrap() {
            row
        } else {
            break;
        };

        let value_map = row_to_value_map(row);
        let profile = Profile::from(value_map);

        // INFO: can safely unwrap because id can't be none if profiles are from db!
        if exclude_profile_id == profile.id.unwrap_or(0) {
            continue;
        }

        let pid: Option<Uuid> = match profile.pid {
            Some(value) => Uuid::from_slice(value.as_slice()).ok(),
            _ => None,
        };

        let profile_response = ProfileResponse {
            pid,
            first_name: profile.first_name,
            last_name: profile.last_name,
            location: profile.location,
            birth_date: profile.birth_date,
            is_visible: profile.is_visible,
        };

        profiles.push(profile_response);
    }

    return profiles;
}
