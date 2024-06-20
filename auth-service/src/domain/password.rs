use color_eyre::eyre::{eyre, Result};
use regex_automata::{meta::Regex, Input};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

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
    pub fn parse(password: Secret<String>) -> Result<Password> {
        if validate_password(&password) {
            Ok(Self(password))
        } else {
            Err(eyre!("Failed to parse string to a Password type"))
        }
    }
}

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

fn validate_password(password: &Secret<String>) -> bool {
    password_has_lowercase(password.expose_secret())
        && password_has_uppercase(password.expose_secret())
        && password_has_required_length(password.expose_secret())
        && password_has_numbers(password.expose_secret())
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

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::Secret;

    #[test]
    fn valid_password_should_return_result() {
        let valid_password = Secret::new("abcDEF123".to_string());

        assert!(Password::parse(valid_password).is_ok());
    }

    #[test]
    fn invalid_password_should_return_error() {
        let invalid_password = Secret::new("ABCDEF123".to_string()); // Misses lowercases
        assert!(Password::parse(invalid_password).is_err());

        let invalid_password = Secret::new("abcdef123".to_string()); // Misses uppercases
        assert!(Password::parse(invalid_password).is_err());

        let invalid_password = Secret::new("abcdEFGH".to_string()); // Misses numbers
        assert!(Password::parse(invalid_password).is_err());
    }

    #[test]
    fn should_be_able_to_convert_a_borrowed_password_to_str() {
        let valid_password = Secret::new("abcDEF123".to_string());
        let password = Password::parse(valid_password.clone()).expect("Couldn't parse password");
        let password_str = password.as_ref();

        assert_eq!(valid_password.expose_secret(), password_str.expose_secret());
    }
}
