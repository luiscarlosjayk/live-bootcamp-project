use std::env;

pub fn is_local() -> bool {
    let environment = env::var("ENVIRONMENT").expect("Missing ENVIRONMENT environment variable.");

    environment == *"local"
}

pub fn is_remote() -> bool {
    let environment = env::var("ENVIRONMENT").expect("Missing ENVIRONMENT environment variable.");

    environment == *"remote"
}

pub fn get_env(env_name: String) -> String {
    env::var(env_name).expect("Missing {env_name} environment variable.")
}