use std::fmt;

mod login;
mod logout;
mod signup;
mod verify_2fa;
mod verify_token;

// Re-export items from sub-modules;
pub use login::*;
pub use logout::*;
pub use signup::*;
pub use verify_2fa::*;
pub use verify_token::*;

#[derive(Debug)]
pub enum Paths {
    Root,
    Signup,
    Login,
    Logout,
    Verify2FA,
    VerifyToken,
}

impl Paths {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Root => "/",
            Self::Signup => "/signup",
            Self::Login => "/login",
            Self::Logout => "/logout",
            Self::Verify2FA => "/verify-2fa",
            Self::VerifyToken => "/verify-token",
        }
    }
}

impl fmt::Display for Paths {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = match self {
            Self::Root => "/",
            Self::Signup => "/signup",
            Self::Login => "/login",
            Self::Logout => "/logout",
            Self::Verify2FA => "/verify-2fa",
            Self::VerifyToken => "/verify-token",
        };
        write!(f, "{}", output)
    }
}
