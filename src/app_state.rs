use std::{collections::HashMap, sync::Arc};

use libsql::Connection;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{
    config::Config,
    models::{profile::CacheProfile, user::CacheUser},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db_conn: Connection,
    pub profile_cache: Arc<Mutex<HashMap<Uuid, CacheProfile>>>,
    pub user_cache: Arc<Mutex<HashMap<Uuid, CacheUser>>>,
}
