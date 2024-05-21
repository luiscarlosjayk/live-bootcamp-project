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

impl std::fmt::Display for Paths {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
