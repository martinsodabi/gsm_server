use crate::{
    utils::{app_error::AppError, password::hash_password},
    views::user::RegisterParams,
};

use super::util::{self, byte_from_value, i64_from_value, query_get_one, row_to_value_map};
use libsql::{Connection, Value as DBV};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::error;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub id: Option<i64>,
    pub pid: Option<Vec<u8>>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub created_at: Option<i64>,
    pub updated_at: Option<i64>,
}

impl From<HashMap<String, libsql::Value>> for User {
    fn from(value_map: HashMap<String, libsql::Value>) -> Self {
        Self {
            id: util::i64_from_value("id", &value_map),
            pid: util::byte_from_value("pid", &value_map),
            email: util::string_from_value("email", &value_map),
            password: util::string_from_value("password", &value_map),
            created_at: util::i64_from_value("created_at", &value_map),
            updated_at: util::i64_from_value("updated_at", &value_map),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CacheUser {
    pub id: i32,
    pub pid: Uuid,
    pub email: String,
}

impl CacheUser {
    pub fn from(db_user: &User) -> Result<Self, AppError> {
        let id = db_user.id.ok_or(AppError::InternalServerError)? as i32;
        let pid_vec = db_user.pid.as_ref().ok_or(AppError::InternalServerError)?;

        let email = db_user
            .email
            .as_ref()
            .ok_or(AppError::InternalServerError)?
            .to_owned();

        let pid = Uuid::from_slice(pid_vec.as_slice()).map_err(|err| {
            error!("{:?}", err);
            return AppError::Unauthorized;
        })?;

        Ok(CacheUser { id, pid, email })
    }
}

pub async fn create_user(
    params: &RegisterParams,
    db_conn: &Connection,
) -> Result<(i64, String), AppError> {
    let pid = Uuid::new_v4().as_bytes().to_vec();
    let hashed_password = hash_password(&params.password)?;

    let query_statement = "INSERT INTO users (email, password, pid) values (?, ?, ?) RETURNING id";
    let query_args = vec![
        DBV::from(params.email.as_str()),
        DBV::from(hashed_password.as_str()),
        DBV::from(pid.to_owned()),
    ];

    let row = query_get_one(query_statement, query_args, &db_conn).await?;
    let value_map = row_to_value_map(row);
    let id = i64_from_value("id", &value_map).ok_or(AppError::NotFound)?;

    // TODO: Abstract this into a function!
    let uuid = Uuid::from_slice(pid.as_slice()).map_err(|err| {
        error!("{:?}", err);
        return AppError::InternalServerError;
    })?;

    Ok((id, uuid.to_string()))
}

pub async fn get_user_ids_by_email(
    email: &String,
    db_conn: &Connection,
) -> Result<(i64, String), AppError> {
    let query_statement = "SELECT id, pid FROM users WHERE email = ? LIMIT 1";
    let query_args = vec![DBV::from(email.as_str())];

    let row = match query_get_one(query_statement, query_args, db_conn).await {
        Ok(value) => value,
        Err(err) => match err {
            AppError::NotFound => return Err(AppError::UserDoesNotExist),
            err => return Err(err),
        },
    };

    let value_map = row_to_value_map(row);

    let id = i64_from_value("id", &value_map).ok_or(AppError::NotFound)?;
    let pid = byte_from_value("pid", &value_map).ok_or(AppError::NotFound)?;

    let uuid = Uuid::from_slice(pid.as_slice()).map_err(|err| {
        error!("{:?}", err);
        return AppError::InternalServerError;
    })?;

    Ok((id, uuid.to_string()))
}

pub async fn get_user_by_email(email: &String, db_conn: &Connection) -> Result<User, AppError> {
    let query_statement = "SELECT * FROM users WHERE email = ? LIMIT 1";
    let query_args = vec![DBV::from(email.as_str())];

    let row = match query_get_one(query_statement, query_args, db_conn).await {
        Ok(value) => value,
        Err(err) => match err {
            AppError::NotFound => return Err(AppError::UserDoesNotExist),
            err => return Err(err),
        },
    };

    let value_map = row_to_value_map(row);

    Ok(User::from(value_map))
}

pub async fn get_user_by_pid(pid: &String, db_conn: &Connection) -> Result<User, AppError> {
    // TODO: Abstract this into a function!
    let pid = Uuid::try_parse(pid)
        .map_err(|err| {
            error!("{:?}", err);
            return AppError::InternalServerError;
        })?
        .as_bytes()
        .to_vec();

    let query_statement = "SELECT * FROM users WHERE pid = ? LIMIT 1";
    let query_args = vec![DBV::from(pid)];

    let row = match query_get_one(query_statement, query_args, db_conn).await {
        Ok(value) => value,
        Err(err) => match err {
            AppError::NotFound => return Err(AppError::UserDoesNotExist),
            err => return Err(err),
        },
    };

    let value_map = row_to_value_map(row);

    Ok(User::from(value_map))
}
