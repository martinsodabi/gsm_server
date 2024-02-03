use serde_json::Value;
use sqids::Sqids;
use tracing::error;

use crate::utils::app_error::AppError;

pub fn i64_from_serde_object(key: &str, map: &Value) -> Option<i64> {
    match map.get(key) {
        Some(value) => value.as_i64(),
        None => None,
    }
}

pub fn f64_from_serde_object(key: &str, map: &Value) -> Option<f64> {
    match map.get(key) {
        Some(value) => value.as_f64(),
        None => None,
    }
}

pub fn bool_from_serde_object(key: &str, map: &Value) -> Option<bool> {
    match map.get(key) {
        Some(value) => value.as_bool(),
        None => None,
    }
}

pub fn string_from_serde_object(key: &str, map: &Value) -> Option<String> {
    match map.get(key) {
        Some(value) => match value.as_str() {
            Some(value) => Some(value.to_string()),
            None => None,
        },
        None => None,
    }
}

pub fn str_from_serde_object(key: &str, map: &Value) -> Option<String> {
    match map.get(key) {
        Some(value) => serde_json::from_value(value.to_owned()).ok(),
        None => None,
    }
}

pub fn vec_from_serde_object(key: &str, map: &Value) -> Option<Vec<Value>> {
    match map.get(key) {
        Some(value) => match value.as_array() {
            Some(value) => Some(value.to_owned()),
            None => None,
        },
        None => None,
    }
}

pub fn i64_from_serde_array(index: usize, array: &Value) -> Option<i64> {
    match array.get(index) {
        Some(value) => value.as_i64(),
        None => None,
    }
}

pub fn f64_from_serde_array(index: usize, array: &Value) -> Option<f64> {
    match array.get(index) {
        Some(value) => value.as_f64(),
        None => None,
    }
}

pub fn bool_from_serde_array(index: usize, array: &Value) -> Option<bool> {
    match array.get(index) {
        Some(value) => value.as_bool(),
        None => None,
    }
}

pub fn string_from_serde_array(index: usize, array: &Value) -> Option<String> {
    match array.get(index) {
        Some(value) => match value.as_str() {
            Some(value) => Some(value.to_string()),
            None => None,
        },
        None => None,
    }
}

pub fn vec_from_serde_array(index: usize, map: &Value) -> Option<Vec<Value>> {
    match map.get(index) {
        Some(value) => match value.as_array() {
            Some(value) => Some(value.to_owned()),
            None => None,
        },
        None => None,
    }
}

pub fn id_to_sqids(id: u64, sqids: Sqids) -> Result<String, AppError> {
    let sqids = sqids.encode(&[id]).map_err(|err| {
        error!("{:?}", err);
        AppError::InternalServerError
    })?;

    Ok(sqids)
}

pub fn sqids_to_id(sqids_id: String, sqids: Sqids) -> Vec<u64> {
    sqids.decode(sqids_id.as_str())
}
