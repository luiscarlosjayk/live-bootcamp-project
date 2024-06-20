use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};
use std::hash::Hash;
use validator::ValidateEmail;

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

impl Email {
    pub fn parse(email: Secret<String>) -> Result<Email> {
        if email.expose_secret().validate_email() {
            Ok(Self(email))
        } else {
            Err(eyre!("Invalid email: {}", email.expose_secret()))
        }
    }
}

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::internet::raw::FreeEmail, locales::EN, Fake};

    #[test]
    fn empty_string_is_rejected() {
        let email = Secret::new("".to_string());
        assert!(Email::parse(email).is_err());
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = Secret::new("ursuladomain.com".to_string());
        assert!(Email::parse(email).is_err());
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = Secret::new("@domain.com".to_string());
        assert!(Email::parse(email).is_err());
    }

    #[test]
    fn should_be_able_to_convert_a_borrowed_email_to_str() {
        let valid_email: String = FreeEmail(EN).fake();
        let email = Email::parse(Secret::new(valid_email.clone())).unwrap();
        let email_str = email.as_ref();

        assert_eq!(&valid_email, email_str.expose_secret());
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(g: &mut G) -> Self {
            let email = FreeEmail(EN).fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(Secret::new(valid_email.0)).is_ok()
    }
}
