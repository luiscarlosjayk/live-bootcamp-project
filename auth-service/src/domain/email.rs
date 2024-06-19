use color_eyre::eyre::{eyre, Result};
use validator::ValidateEmail;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(email: String) -> Result<Email> {
        if email.validate_email() {
            Ok(Self(email))
        } else {
            Err(eyre!("Invalid email: {}", email))
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
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert!(Email::parse(email).is_err());
    }
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert!(Email::parse(email).is_err());
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert!(Email::parse(email).is_err());
    }

    #[test]
    fn should_be_able_to_convert_a_borrowed_email_to_str() {
        let valid_email: String = FreeEmail(EN).fake();
        let email = Email::parse(valid_email.clone()).unwrap();
        let email_str = email.as_ref();

        assert_eq!(valid_email, email_str.to_string());
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
        Email::parse(valid_email.0).is_ok()
    }
}
