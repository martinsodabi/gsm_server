use axum::{body::Body, extract::State, http::Request, middleware::Next, response::IntoResponse};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::{
    app_state::AppState,
    config::Config,
    models::user::{get_user_by_pid, CacheUser},
    utils::app_error::AppError,
};

const AUTHORIZATION: &str = "Authorization";
const BEARER: &str = "Bearer";

pub async fn authenticate(
    State(app_state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, AppError> {
    // Get authorization header from the http request
    let authorization_header = match request.headers().get(AUTHORIZATION) {
        Some(auth_header) => auth_header,
        None => return Err(AppError::Unauthorized),
    };

    let authorization = authorization_header.to_str().map_err(|err| {
        error!("{:?}", err);
        return AppError::Unauthorized;
    })?;

    if !authorization.starts_with(BEARER) {
        return Err(AppError::Unauthorized);
    }

    let jwt_token = authorization.trim_start_matches(BEARER).trim();

    let token_header = jsonwebtoken::decode_header(jwt_token).map_err(|err| {
        error!("{:?}", err);
        return AppError::Unauthorized;
    })?;

    let user_claims = jsonwebtoken::decode::<UserClaims>(
        jwt_token,
        &DecodingKey::from_secret(app_state.config.jwt_secret.as_bytes()),
        &Validation::new(token_header.alg),
    )
    .map_err(|err| {
        error!("{:?}", err);
        return AppError::Unauthorized;
    })?;

    let user_ref = user_claims.claims.sub;

    let user_pid = Uuid::parse_str(user_ref.as_str()).map_err(|err| {
        error!("{:?}", err);
        return AppError::InternalServerError;
    })?;

    let user = if let Some(value) = app_state.user_cache.lock().await.get(&user_pid) {
        value.to_owned()
    } else {
        let db_conn = app_state.db_conn;
        let db_user = get_user_by_pid(&user_ref, &db_conn).await.map_err(|err| {
            error!("{:?}", err);
            return AppError::Unauthorized;
        })?;

        let cache_user = CacheUser::from(&db_user)?;

        app_state
            .user_cache
            .lock()
            .await
            .insert(cache_user.pid, cache_user.to_owned());

        cache_user
    };

    request.extensions_mut().insert(user);
    return Ok(next.run(request).await);
}

pub fn create_jwt_token(config: &Config, user_pid: &String) -> Result<String, AppError> {
    let now = chrono::Utc::now();
    let jwt_expiry_minute = config.jwt_expiry_minute;
    let exp = (now + chrono::Duration::minutes(jwt_expiry_minute as i64)).timestamp() as u64;
    let claims = UserClaims {
        sub: user_pid.to_owned(),
        exp,
    };
    let jwt_secret = &config.jwt_secret;
    let jwt_token = encode_user_claims(&claims, jwt_secret)?;

    return Ok(jwt_token);
}

fn encode_user_claims(user_claims: &UserClaims, secret: &String) -> Result<String, AppError> {
    let token = encode(
        &Header::default(),
        user_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|err| {
        error!("{:?}", err);
        return AppError::InternalServerError;
    })?;

    return Ok(token);
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub sub: String,
    pub exp: u64,
}
