use nutype::nutype;
use once_cell::sync::OnceCell;
use regex::Regex;

use crate::UserFacingError;

#[nutype(validate(present, min_len = 3, max_len = 30))]
#[derive(AsRef, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Username(String);

impl UserFacingError for UsernameError {
    fn formatted_error(&self) -> &'static str {
        match self {
            UsernameError::Missing => "User name cannot be empty.",
            UsernameError::TooShort => "User name is too short. Must be at least 3 characters.",
            UsernameError::TooLong => "User name is too long. Must be at most 30 characters.",
        }
    }
}

#[nutype(validate(present, min_len = 8))]
#[derive(AsRef, Clone, Serialize, Deserialize, PartialEq)]
pub struct Password(String);

impl UserFacingError for PasswordError {
    fn formatted_error(&self) -> &'static str {
        match self {
            PasswordError::Missing => "Password cannot be empty.",
            PasswordError::TooShort => "Password is too short. Must be at least 8 characters.",
        }
    }
}

#[nutype(validate(max_len = 30))]
#[derive(AsRef, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DisplayName(String);

impl DisplayName {
    pub const MAX_CHARS: usize = 30;
}

impl UserFacingError for DisplayNameError {
    fn formatted_error(&self) -> &'static str {
        match self {
            DisplayNameError::TooLong => "Display name is too long. Must be at most 30 characters.",
        }
    }
}

static EMAIL_REGEX: OnceCell<EmailRegex> = OnceCell::new();

#[derive(Debug)]
pub struct EmailRegex(Regex);

impl EmailRegex {
    pub fn global() -> &'static Self {
        EMAIL_REGEX.get().expect("email regex is not initialized")
    }

    pub fn init() -> Self {
        Self(regex::Regex::new(r#"^\S+@\S+\.\S{1,64}$"#).unwrap())
    }

    pub fn is_valid<T: AsRef<str>>(&self, text: T) -> bool {
        self.0.is_match(text.as_ref())
    }
}

fn is_valid_email(email: &str) -> bool {
    let email_regex = EMAIL_REGEX.get_or_init(EmailRegex::init);

    email_regex.is_valid(email)
}

#[nutype(validate(with = is_valid_email))]
#[derive(AsRef, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Email(String);

impl UserFacingError for EmailError {
    fn formatted_error(&self) -> &'static str {
        match self {
            EmailError::Invalid => "Email is not valid. Format: your_name@example.com",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_username() {
        assert!(Username::new("john_doe").is_ok());
    }

    #[test]
    fn invalid_username() {
        assert_eq!(Username::new(""), Err(UsernameError::Missing));
        assert_eq!(Username::new("ab"), Err(UsernameError::TooShort));
        let long_name = "a".repeat(31);
        assert_eq!(Username::new(long_name), Err(UsernameError::TooLong));
    }

    #[test]
    fn valid_password() {
        assert!(Password::new("secure_password").is_ok());
    }

    #[test]
    fn invalid_password() {
        assert!(matches!(Password::new(""), Err(PasswordError::Missing)));
        assert!(matches!(Password::new("short"), Err(PasswordError::TooShort)));
    }

    #[test]
    fn email_validation() {
        assert!(Email::new("test@example.com").is_ok());
        assert!(Email::new("user.name+tag@domain.co.uk").is_ok());
        
        assert_eq!(Email::new("invalid"), Err(EmailError::Invalid));
        assert_eq!(Email::new("test@"), Err(EmailError::Invalid));
        assert_eq!(Email::new("@example.com"), Err(EmailError::Invalid));
    }
}
