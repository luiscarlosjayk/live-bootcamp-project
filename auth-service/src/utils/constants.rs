use dotenvy::dotenv;
use lazy_static::lazy_static;
use secrecy::Secret;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: Secret<String> = set_token();
    pub static ref DATABASE_URL: Secret<String> = set_database_url();
    pub static ref REDIS_HOST_NAME: String = set_redis_hostname();
    // AWS
    pub static ref AWS_ACCESS_KEY_ID: Secret<String> = set_access_key_id();
    pub static ref AWS_SECRET_ACCESS_KEY: Secret<String> = set_aws_secret_access_key();
    pub static ref AWS_DEFAULT_REGION: Secret<String> = set_aws_region();
    // EmailClient
    pub static ref EMAIL_SENDER: String = set_email_client_sender();
}

fn set_token() -> Secret<String> {
    dotenv().ok();
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");

    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }

    Secret::new(secret)
}

fn set_email_client_sender() -> String {
    dotenv().ok();
    let secret = std_env::var(env::EMAIL_SENDER_NAME_ENV_VAR).expect("EMAIL_SENDER must be set.");

    if secret.is_empty() {
        panic!("EMAIL_SENDER must not be empty.");
    }

    secret
}

fn set_database_url() -> Secret<String> {
    dotenv().ok();
    let secret = std_env::var(env::DATABASE_URL_ENV_VAR).expect("DATABASE_URL must be set.");

    if secret.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }
    Secret::new(secret)
}

fn set_redis_hostname() -> String {
    dotenv().ok();
    let secret =
        std_env::var(env::REDIS_HOST_NAME_ENV_VAR).unwrap_or(DEFAULT_REDIS_HOSTNAME.to_owned());

    if secret.is_empty() {
        panic!("REDIS_HOST_NAME must not be empty.");
    }
    secret
}

fn set_access_key_id() -> Secret<String> {
    dotenv().ok();
    let secret =
        std_env::var(env::AWS_ACCESS_KEY_ID_NAME_ENV_VAR).expect("AWS_ACCESS_KEY_ID must be set.");

    if secret.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }

    Secret::new(secret)
}

fn set_aws_secret_access_key() -> Secret<String> {
    dotenv().ok();
    let secret = std_env::var(env::AWS_SECRET_ACCESS_KEY_NAME_ENV_VAR)
        .expect("AWS_SECRET_ACCESS_KEY must be set.");

    if secret.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }

    Secret::new(secret)
}

fn set_aws_region() -> Secret<String> {
    dotenv().ok();
    let secret = std_env::var(env::AWS_DEFAULT_REGION_NAME_ENV_VAR)
        .expect("AWS_DEFAULT_REGION must be set.");

    if secret.is_empty() {
        panic!("DATABASE_URL must not be empty.");
    }

    Secret::new(secret)
}

pub const JWT_COOKIE_NAME: &str = "jwt";
pub const DEFAULT_REDIS_HOSTNAME: &str = "127.0.0.1";

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const BASE_PATH_ENV_VAR: &str = "BASE_PATH";
    pub const DROPLET_IP_ENV_VAR: &str = "DROPLET_IP";
    pub const RECAPTCHA_SECRET_ENV_VAR: &str = "RECAPTCHA_SECRET";
    pub const DATABASE_URL_ENV_VAR: &str = "DATABASE_URL";
    pub const REDIS_HOST_NAME_ENV_VAR: &str = "REDIS_HOST_NAME";
    pub const AWS_ACCESS_KEY_ID_NAME_ENV_VAR: &str = "AWS_ACCESS_KEY_ID";
    pub const AWS_SECRET_ACCESS_KEY_NAME_ENV_VAR: &str = "AWS_SECRET_ACCESS_KEY";
    pub const AWS_DEFAULT_REGION_NAME_ENV_VAR: &str = "AWS_DEFAULT_REGION";
    pub const EMAIL_SENDER_NAME_ENV_VAR: &str = "EMAIL_SENDER";
}

pub mod prod {
    pub const APP_ADDRESS: &str = "0.0.0.0:3000";
}

pub mod test {
    pub const APP_ADDRESS: &str = "127.0.0.1:0";
}
