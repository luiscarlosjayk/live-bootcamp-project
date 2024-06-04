use std::env;

pub fn is_local() -> bool {
    let environment = env::var("ENVIRONMENT").expect("Missing ENVIRONMENT environment variable.");

    environment == *"local"
}

pub fn is_remote() -> bool {
    let environment = env::var("ENVIRONMENT").expect("Missing ENVIRONMENT environment variable.");

    environment == *"remote"
}

pub fn get_env<Env>(env_name: Env) -> String
where
    Env: Into<String>,
{
    let env_name = env_name.into();

    env::var(&env_name)
        .unwrap_or_else(|_| panic!("{}", format!("Missing {} environment variable.", &env_name)))
}
