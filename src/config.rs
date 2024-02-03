use libsql::{Connection, Database};
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct Config {
    pub sqids_alphabet: String,
    pub turso_db_url: String,
    pub turso_auth_token: String,
    pub jwt_secret: String,
    pub jwt_expiry_minute: u64,
    pub jwt_maxage: u64,
}

impl Config {
    pub fn init() -> Config {
        let sqids_alphabet = std::env::var("SQIDS_ALPHABET").expect("SQIDS_ALPHABET must be set");
        let turso_db_url = std::env::var("TURSO_URL").expect("DATABASE_URL must be set");
        let turso_auth_token = std::env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH must be set");
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_expiry_minute =
            std::env::var("JWT_EXPIRY_MINUTE").expect("JWT_EXPIRY_MINUTE must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");

        return Config {
            sqids_alphabet,
            turso_db_url,
            turso_auth_token,
            jwt_secret,
            jwt_expiry_minute: jwt_expiry_minute
                .parse::<u64>()
                .expect("Should parse jwt_expiry_minute"),
            jwt_maxage: jwt_maxage.parse::<u64>().expect("Should parse jwt_maxage"),
        };
    }
}

//Tried to use libsql crate but open_remote conn works even with empty url ???
//Leaving it here until I figure it out!!!
pub fn initialize_database(config: &Config, can_use_local_db: bool) -> Connection {
    let url: &str;
    let token = &config.turso_auth_token;

    let db = if can_use_local_db {
        warn!("Using sqlite local file db!");
        url = "file:////Users/martinsodabi/Developer/RustProjects/gsm/gsm.db";
        Database::open(url).unwrap()
    } else {
        url = config.turso_db_url.as_str();
        Database::open_remote(url, token).unwrap()
    };

    match db.connect() {
        Ok(db_conn) => {
            info!("CONNECTED TO TURSO DB {:?}", url);
            db_conn
        }

        Err(err) => {
            error!("{:?}", err);
            info!("UNABLE TO CONNECT TO TURSO DB {:?}", url);
            info!("REVERTING TO IN_MEMORY DB");
            Database::open_in_memory()
                .unwrap()
                .connect()
                .expect("UNABLE TO OPEN IN_MEMORY DB")
        }
    }
}
