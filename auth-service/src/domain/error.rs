pub enum AuthAPIError {
    UserAlreadyExists,
    InvalidCredentials,
    UnexpectedError,
    InvalidRecaptcha,
    IncorrectCredentials,
    MissingToken,
    InvalidToken,
}
