use crate::domain::{Email, Password};

#[derive(Clone, PartialEq, Debug)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> User {
        let email = Email::parse(email).expect("Email was not able to be parsed");
        let password = Password::parse(password).expect("Password was not able to be pared");

        User {
            email: email.clone(),
            password: password.clone(),
            requires_2fa,
        }
    }
}
