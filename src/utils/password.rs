use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use tracing::error;

use crate::utils::app_error::AppError;

pub fn hash_password(password: &String) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| {
            error!("{:?}", err);
            return AppError::InternalServerError;
        })?;

    return Ok(password_hash.to_string());
}

pub fn verify_password(password: &String, hashed_password: &String) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hashed_password).map_err(|err| {
        error!("{:?}", err);
        return AppError::InternalServerError;
    })?;

    return Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok());
}
