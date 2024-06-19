use color_eyre::eyre::{eyre, Result};
use regex_automata::{meta::Regex, Input};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Password(String);

/**
 * Password regex pattern: (NOT WORKING)
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
    pub fn parse(password: String) -> Result<Password> {
        let is_valid_password = validate_password(&password);

        if is_valid_password {
            Ok(Self(password))
        } else {
            Err(eyre!("Invalid password"))
        }
    }
}

fn validate_password(password: &str) -> bool {
    password_has_lowercase(password)
        && password_has_uppercase(password)
        && password_has_required_length(password)
        && password_has_numbers(password)
}

fn password_has_uppercase(password: &str) -> bool {
    let regex = Regex::new(r"[A-Z]+").expect("Could'n build regex pattern");
    regex.is_match(Input::new(password))
}

fn password_has_lowercase(password: &str) -> bool {
    let regex = Regex::new(r"[a-z]+").expect("Could'n build regex pattern");
    regex.is_match(Input::new(password))
}

fn password_has_numbers(password: &str) -> bool {
    let regex = Regex::new(r"[0-9]+").expect("Could'n build regex pattern");
    regex.is_match(Input::new(password))
}

fn password_has_required_length(password: &str) -> bool {
    password.len() > 8
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_password_should_return_result() {
        let valid_password = "abcDEF123";

        assert!(Password::parse(valid_password.to_string()).is_ok());
    }

    #[test]
    fn invalid_password_should_return_error() {
        let invalid_password: String = "ABCDEF123".to_string(); // Misses lowercases
        assert!(Password::parse(invalid_password).is_err());

        let invalid_password: String = "abcdef123".to_string(); // Misses uppercases
        assert!(Password::parse(invalid_password).is_err());

        let invalid_password: String = "abcdEFGH".to_string(); // Misses numbers
        assert!(Password::parse(invalid_password).is_err());
    }

    #[test]
    fn should_be_able_to_convert_a_borrowed_password_to_str() {
        let valid_password = "abcDEF123".to_string();
        let password = Password::parse(valid_password.clone()).expect("Couldn't parse password");
        let password_str = password.as_ref();

        assert_eq!(valid_password, password_str.to_string());
    }
}
