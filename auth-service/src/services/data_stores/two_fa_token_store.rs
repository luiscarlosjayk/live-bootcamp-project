use color_eyre::eyre::{eyre, Result};
use secrecy::ExposeSecret;

use crate::domain::{
    data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStore, TwoFACodeStoreError},
    Email,
};
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct HashmapTwoFACodeStore {
    codes: HashMap<Email, (LoginAttemptId, TwoFACode)>,
}

#[async_trait::async_trait]
impl TwoFACodeStore for HashmapTwoFACodeStore {
    async fn add_code(
        &mut self,
        email: Email,
        login_attempt_id: LoginAttemptId,
        code: TwoFACode,
    ) -> Result<(), TwoFACodeStoreError> {
        let value = (login_attempt_id, code);
        self.codes.insert(email, value);

        Ok(())
    }

    async fn remove_code(&mut self, email: &Email) -> Result<(), TwoFACodeStoreError> {
        match self.codes.remove(email) {
            Some(_) => Ok(()),
            None => Err(TwoFACodeStoreError::UnexpectedError(eyre!(format!(
                "Failed to remove email: {} from the 2FA Code Store",
                &email.as_ref().expose_secret()
            )))),
        }
    }

    async fn get_code(
        &self,
        email: &Email,
    ) -> Result<(LoginAttemptId, TwoFACode), TwoFACodeStoreError> {
        match self.codes.get(email) {
            Some(code) => Ok(code.clone()),
            None => Err(TwoFACodeStoreError::LoginAttemptIdNotFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        data_stores::{LoginAttemptId, TwoFACode, TwoFACodeStoreError},
        Email,
    };
    use secrecy::Secret;

    const DEFAULT_EMAIL: &str = "testing@email.com";

    #[tokio::test]
    async fn test_add_2fa_code() {
        let mut two_fa_code_store = HashmapTwoFACodeStore::default();
        let email = Email::parse(Secret::new(DEFAULT_EMAIL.to_string())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let two_fa_code = TwoFACode::default();

        let result = two_fa_code_store
            .add_code(email, login_attempt_id, two_fa_code)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_2fa_code() {
        let mut two_fa_code_store = HashmapTwoFACodeStore::default();
        let email = Email::parse(Secret::new(DEFAULT_EMAIL.to_string())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let result = two_fa_code_store
            .add_code(email.clone(), login_attempt_id, code)
            .await;
        assert!(result.is_ok());

        let result = two_fa_code_store.remove_code(&email).await;
        assert!(result.is_ok());

        let result = two_fa_code_store.remove_code(&email).await;
        assert_eq!(
            result,
            Err(TwoFACodeStoreError::UnexpectedError(eyre!(format!(
                "Failed to remove email: {} from the 2FA Code Store",
                &email.as_ref().expose_secret()
            ))))
        );
    }

    #[tokio::test]
    async fn test_get_2fa_code() {
        let mut two_fa_code_store = HashmapTwoFACodeStore::default();
        let email = Email::parse(Secret::new(DEFAULT_EMAIL.to_string())).unwrap();
        let login_attempt_id = LoginAttemptId::default();
        let code = TwoFACode::default();

        let result = two_fa_code_store
            .add_code(email.clone(), login_attempt_id.clone(), code.clone())
            .await;
        assert!(result.is_ok());

        let result = two_fa_code_store.get_code(&email).await.unwrap();
        assert_eq!(result.0, login_attempt_id);
        assert_eq!(result.1, code);
    }
}
