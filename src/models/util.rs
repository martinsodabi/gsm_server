use std::collections::HashMap;
use tracing::{error, warn};

use crate::utils::app_error::AppError;
use libsql::{Connection, Value};

pub fn i64_from_value(column_name: &str, value_map: &HashMap<String, Value>) -> Option<i64> {
    match value_map.get(column_name) {
        Some(value_type) => match value_type {
            Value::Integer(value) => Some(value.to_owned()),
            _ => None,
        },
        None => None,
    }
}

pub fn f64_from_value(column_name: &str, value_map: &HashMap<String, Value>) -> Option<f64> {
    match value_map.get(column_name) {
        Some(value_type) => match value_type {
            Value::Real(value) => Some(value.to_owned()),
            _ => None,
        },
        None => None,
    }
}

pub fn bool_from_value(column_name: &str, value_map: &HashMap<String, Value>) -> Option<bool> {
    match value_map.get(column_name) {
        Some(value_type) => match value_type {
            Value::Integer(value) => {
                if value.to_owned() == 0 {
                    Some(false)
                } else {
                    Some(true)
                }
            }
            _ => None,
        },
        None => None,
    }
}

pub fn string_from_value(column_name: &str, value_map: &HashMap<String, Value>) -> Option<String> {
    match value_map.get(column_name) {
        Some(value_type) => match value_type {
            Value::Text(value) => Some(value.to_owned()),
            _ => None,
        },
        None => None,
    }
}

pub fn byte_from_value(column_name: &str, value_map: &HashMap<String, Value>) -> Option<Vec<u8>> {
    match value_map.get(column_name) {
        Some(value_type) => match value_type {
            Value::Blob(value) => Some(value.to_owned()),
            _ => None,
        },
        None => None,
    }
}

pub async fn execute(
    statement: &str,
    args: Vec<libsql::Value>,
    db_conn: &Connection,
) -> Result<u64, AppError> {
    let rows_affected = db_conn.execute(statement, args).await.map_err(|err| {
        /* Network layer error */
        error!("{:?}", err);
        AppError::InternalServerError
    })?;

    return Ok(rows_affected);
}

pub async fn query_get_one(
    statement: &str,
    args: Vec<libsql::Value>,
    db_conn: &Connection,
) -> Result<libsql::Row, AppError> {
    let row = db_conn
        .query(statement, args)
        .await
        .map_err(|err| {
            /* Network layer error */
            error!("{:?}", err);
            AppError::InternalServerError
        })?
        .next()
        .map_err(|err| {
            /* LIBSQL sever layer error */
            error!("{:?}", err);
            AppError::InternalServerError
        })?
        .ok_or_else(|| {
            /* SQL query layer error */
            warn!(
                target = "Database event",
                warning = "Tried searching for non existent value!",
                "{:?}",
                AppError::NotFound
            );

            AppError::NotFound
        })?;

    return Ok(row);
}

pub async fn query_get_many(
    statement: &str,
    args: Vec<libsql::Value>,
    db_conn: &Connection,
) -> Result<libsql::Rows, AppError> {
    let rows = db_conn.query(statement, args).await.map_err(|err| {
        /* Network layer error */
        error!("{:?}", err);
        AppError::InternalServerError
    })?;

    return Ok(rows);
}

pub fn row_to_value_map(row: libsql::Row) -> HashMap<String, libsql::Value> {
    let mut value_map = HashMap::new();

    //A database row should not have up to a 100 columns...
    //keeping the range in case not to loop forever!!!
    for i in 0..100 {
        if let Some(column_name) = row.column_name(i) {
            let column_value = row.get_value(i).unwrap_or(libsql::Value::Null);
            value_map.insert(column_name.to_string(), column_value)
        } else {
            break;
        };
    }

    return value_map;
}
