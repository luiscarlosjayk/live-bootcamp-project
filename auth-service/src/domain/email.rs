use validator::{ValidateEmail, ValidationError};

#[derive(Debug, Clone, PartialEq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Email, ValidationError> {
        if email.validate_email() {
            Ok(Email(email))
        } else {
            Err(ValidationError::new("Invalid email"))
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::internet::raw::FreeEmail, locales::EN, Fake};

    #[test]
    fn valid_email_should_return_result() {
        let valid_email: String = FreeEmail(EN).fake();

        assert!(Email::parse(valid_email).is_ok());
    }

    #[test]
    fn invalid_email_should_return_error() {
        let invalid_email: String = "invalid".to_string();

        assert!(Email::parse(invalid_email).is_err());
    }

    #[test]
    fn should_be_able_to_convert_a_borrowed_email_to_str() {
        let valid_email: String = FreeEmail(EN).fake();
        let email = Email::parse(valid_email.clone()).unwrap();
        let email_str = email.as_ref();

        assert_eq!(valid_email, email_str.to_string());
    }
}
