use regex_automata::{meta::Regex, Input};
use validator::ValidationError;

#[derive(Debug, Clone, PartialEq)]
pub struct Password(String);

/**
 * Password regex pattern:
 * r"^(?=.*[a-z])(?=.*[A-Z])(?=.*[0-9])(?=.*[@$!%*#?&])[^\\s]{8,}$""
 *
 * This regex enforces:
 *  - Minimum length of 8 characters.
 *  - At least one lowercase letter.
 *  - At least one uppercase letter.
 *  - At least one digit.
 *  - At least one special character from a predefined set.
 *  - No whitespace characters.
 *
 * Source: Gemini
 */

impl Password {
    pub fn parse(password: String) -> Result<Password, ValidationError> {
        let regex = Regex::new(r"^[a-zA-Z0-9]{8,}$")
            // Regex::new(r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]{8,}$") // @todo: Check why this doesn't work
            .map_err(|_| ValidationError::new("Could not build regex"))?;
        let is_valid_password = regex.is_match(Input::new(password.as_str()));

        if is_valid_password {
            Ok(Password(password))
        } else {
            Err(ValidationError::new("Invalid password"))
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::internet::raw::Password as FakerPassword, locales::*, Fake};
    use std::ops::Range;

    #[test]
    fn valid_password_should_return_result() {
        let valid_password: String = FakerPassword(EN, Range { start: 8, end: 12 }).fake();
        println!("password under test: {valid_password}");

        assert!(Password::parse(valid_password).is_ok());
    }

    #[test]
    fn invalid_password_should_return_error() {
        let invalid_password: String = "invalid".to_string();

        assert!(Password::parse(invalid_password).is_err());
    }

    #[test]
    fn should_be_able_to_convert_a_borrowed_password_to_str() {
        let valid_password: String = FakerPassword(EN, Range { start: 8, end: 12 }).fake();
        let password = Password::parse(valid_password.clone()).unwrap();
        let password_str = password.as_ref();

        assert_eq!(valid_password, password_str.to_string());
    }
}
