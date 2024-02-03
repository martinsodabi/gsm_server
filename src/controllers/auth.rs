use crate::{
    app_state::AppState,
    config::Config,
    middlewares::jwt_auth::create_jwt_token,
    models::{
        profile::{
            create_profile, get_profile_by_user_id, get_profile_id_by_user_id, CacheProfile,
        },
        user::{create_user, get_user_by_email, get_user_ids_by_email, CacheUser},
    },
    utils::{app_error::AppError, password::verify_password},
    views::{
        profile::ProfileParams,
        user::{LoginParams, LoginResponse, RegisterParams, RegisterResponse},
    },
};
use axum::{extract::State, Json};
use chrono::{Duration, Utc};
use libsql::Connection;
use tracing::{error, info, warn};
use uuid::Uuid;

//Production: This must only be called from validate_registration_otp
pub async fn register_user(
    State(app_state): State<AppState>,
    Json(mut params): Json<RegisterParams>,
) -> Result<Json<RegisterResponse>, AppError> {
    params.email = params.email.trim().to_lowercase();
    params.password = params.password.trim().to_string();
    params.first_name = params.first_name.trim_matches(' ').to_string();
    params.last_name = params.last_name.trim_matches(' ').to_string();

    if params.email.is_empty() || params.email.len() > 255 {
        error!("From email condition");
        return Err(AppError::WrongCredential);
    }

    if params.password.is_empty() || params.password.len() > 255 {
        error!("From password condition");
        return Err(AppError::WrongCredential);
    }

    if params.first_name.is_empty() || params.first_name.len() > 255 {
        error!("From first_name condition");
        return Err(AppError::WrongCredential);
    }

    if params.last_name.is_empty() || params.last_name.len() > 255 {
        error!("From last_name condition");
        return Err(AppError::WrongCredential);
    }

    // Typically 13 years ago
    let youngest_birth_date: i64 = (Utc::now() - Duration::days(365 * 13)).timestamp();

    // i64 not supported by libsql crate for now...
    // so more than 121 years ago is the limit!
    let oldest_birth_date: i64 = (Utc::now() - Duration::days(365 * 121)).timestamp();

    if youngest_birth_date < params.birth_date || oldest_birth_date > params.birth_date {
        error!("{:?}", "From birth_date condition");
        return Err(AppError::WrongCredential);
    }

    let db_conn = &app_state.db_conn;

    //Return UserAlreadyExist if there is a user and profile, or...
    //create profile if there is only user, otherwise...
    //continue to create user and profile.
    match get_user_ids_by_email(&params.email, db_conn).await {
        Ok((user_id, user_pid)) => {
            match get_profile_id_by_user_id(user_id, db_conn).await {
                Ok(_) => {
                    return Err(AppError::UserAlreadyExist);
                }
                Err(err) => match err {
                    AppError::NotFound => {
                        return create_profile_and_return(
                            app_state,
                            user_id,
                            &user_pid,
                            &params.first_name,
                            &params.last_name,
                            params.birth_date,
                            &params.email,
                        )
                        .await;
                    }
                    _ => return Err(err),
                },
            };
        }

        Err(err) => match err {
            AppError::UserDoesNotExist => { /* info!("Creating User and Profile"); */ }
            _ => return Err(err),
        },
    };

    let (user_id, user_pid) = create_user(&params, &db_conn).await?;

    create_profile_and_return(
        app_state,
        user_id,
        &user_pid,
        &params.first_name,
        &params.last_name,
        params.birth_date,
        &params.email,
    )
    .await
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(mut params): Json<LoginParams>,
) -> Result<Json<LoginResponse>, AppError> {
    params.email = params.email.trim().to_lowercase();
    params.password = params.password.trim().to_string();

    if params.email.is_empty() || params.password.is_empty() {
        return Err(AppError::MissingCredential);
    }

    let user = get_user_by_email(&params.email, &app_state.db_conn).await?;

    // TODO: Return a more specific error in a case of unsupported password login!
    let hashed_password = user.password.ok_or(AppError::InternalServerError)?;
    let user_pid_vec = user.pid.ok_or(AppError::InternalServerError)?;

    let user_pid_string = Uuid::from_slice(user_pid_vec.as_slice())
        .map_err(|err| {
            error!("{:?}", err);
            return AppError::InternalServerError;
        })?
        .to_string();

    let auth_token = create_jwt_token(&app_state.config, &user_pid_string)?;

    if verify_password(&params.password, &hashed_password)? {
        let user_id = user.id.ok_or(AppError::InternalServerError)?;
        let db_profile = get_profile_by_user_id(user_id, &app_state.db_conn).await?;
        let profile_pid_vec = db_profile
            .pid
            .as_ref()
            .ok_or(AppError::InternalServerError)?;

        let profile_pid = Uuid::from_slice(profile_pid_vec.as_slice()).map_err(|err| {
            error!("{:?}", err);
            return AppError::InternalServerError;
        })?;

        let cache_profile = CacheProfile::from(&db_profile)?;

        app_state
            .profile_cache
            .lock()
            .await
            .insert(profile_pid, cache_profile);

        return Ok(Json(LoginResponse {
            email: user.email,
            auth_token: Some(auth_token),
        }));
    } else {
        return Err(AppError::WrongCredential);
    }
}

async fn create_profile_and_return(
    app_state: AppState,
    user_id: i64,
    user_pid: &String,
    first_name: &String,
    last_name: &String,
    birth_date: i64,
    email: &String,
) -> Result<Json<RegisterResponse>, AppError> {
    let profile = create_profile(
        &app_state.db_conn,
        ProfileParams {
            user_id,
            birth_date,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            location: "JPN".to_string(),
            is_visible: false,
        },
    )
    .await?;

    let auth_token = create_jwt_token(&app_state.config, user_pid)?;

    let user_pid = Uuid::parse_str(user_pid.as_str()).map_err(|err| {
        error!("{:?}", err);
        return AppError::InternalServerError;
    })?;

    let cache_user = CacheUser {
        id: user_id as i32,
        pid: user_pid,
        email: email.to_owned(),
    };

    app_state
        .user_cache
        .lock()
        .await
        .insert(user_pid, cache_user);

    let profile_pid_vec = profile.pid.as_ref().ok_or(AppError::InternalServerError)?;

    let profile_pid = Uuid::from_slice(profile_pid_vec.as_slice()).map_err(|err| {
        error!("{:?}", err);
        return AppError::InternalServerError;
    })?;

    let cache_profile = CacheProfile::from(&profile)?;

    app_state
        .profile_cache
        .lock()
        .await
        .insert(profile_pid, cache_profile);

    Ok(Json(RegisterResponse {
        first_name: profile.first_name,
        last_name: profile.last_name,
        birth_date: profile.birth_date, //profile.birth_date,
        location: profile.location,
        is_visible: profile.is_visible,
        email: Some(email.to_owned()),
        auth_token: Some(auth_token),
    }))
}

pub async fn validate_registration_otp() {}
