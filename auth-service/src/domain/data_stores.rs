use crate::domain::{Email, Password, User};
use async_trait::async_trait;
use rand::Rng;
use regex_automata::meta::Regex;

#[derive(Debug, PartialEq)]
pub enum UserStoreError {
    UserAlreadyExists,
    UserNotFound,
    InvalidCredentials,
    UnexpectedError,
}

#[async_trait]
pub trait UserStore {
    async fn add_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn delete_user(&mut self, user: User) -> Result<(), UserStoreError>;
    async fn get_user(&self, email: &Email) -> Result<User, UserStoreError>;
    async fn validate_user(&self, email: &Email, password: &Password)
        -> Result<(), UserStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum BannedTokenStoreError {
    InvalidToken,
    UnexpectedError,
}

#[async_trait]
pub trait BannedTokenStore {
    async fn add_token(&mut self, token: String) -> Result<(), BannedTokenStoreError>;
    async fn contains_token(&self, token: &str) -> Result<bool, BannedTokenStoreError>;
    async fn empty_store(&mut self) -> Result<(), BannedTokenStoreError>;
}

#[async_trait]
pub trait TwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError>;

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError>;

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError>;
}

#[derive(Debug, PartialEq)]
pub enum TwoFACodeStoreError {
    LoginAttemptIdNotFound,
    UnexpectedError,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoginAttemptId(String);

impl LoginAttemptId {
    pub fn parse(id: String) -> Result<Self, String> {
        let _ = match uuid::Uuid::parse_str(&id) {
            Ok(uuid) => uuid,
            Err(_) => return Err("Error".to_string()),
        };

        if uuid::Uuid::parse_str(&id).is_err() {
            return Err("Invalid loginAttemptId".to_string());
        }

        Ok(Self(id))
    }
}

impl Default for LoginAttemptId {
    fn default() -> Self {
        let random_id = uuid::Uuid::new_v4().to_string();
        Self(random_id)
    }
}

impl AsRef<str> for LoginAttemptId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TwoFACode(String);

impl TwoFACode {
    pub fn parse(code: String) -> Result<Self, String> {
        let regex = Regex::new(r#"(?m)^\d{6}$"#).expect("Could not build regex pattern");
        let is_valid = regex.is_match(&code);

        match is_valid {
            true => Ok(Self(code)),
            false => Err("Invalid 2FA code".to_string()),
        }
    }
}

impl std::fmt::Display for TwoFACode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for TwoFACode {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(100000..=999999);

        Self(random_number.to_string())
    }
}

impl AsRef<str> for TwoFACode {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
