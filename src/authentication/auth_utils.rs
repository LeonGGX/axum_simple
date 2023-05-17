//! src/authentication/auth_utils.rs

use crate::errors::{AppError, MyAppError};
use axum::http::StatusCode;

use crate::authentication::jwt;
use crate::authentication::jwt::TokenDetails;

use password_auth::{generate_hash, verify_password};

///
/// Returns a String with a hashed password
/// uses crate 'password_auth'
///
#[allow(dead_code)]
pub async fn hash_password(password_clear: &str) -> String {
    generate_hash(password_clear)
}

#[allow(dead_code)]
pub async fn check_password(password_clear: &str, password_hash: &str) -> bool {
    let result = verify_password(password_clear, password_hash);
    match result {
        Ok(_) => true,
        Err(_err) => false,
    }
}

pub fn generate_token(
    user_id: uuid::Uuid,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, MyAppError> {
    jwt::generate_jwt_token(user_id, max_age, private_key).map_err(|e| {
        MyAppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("error generating token: {}", e),
        )
    })
}

///
/// Utility function to parse usernames
/// Returns a String with the parsed username or AppError
///
pub fn parse(s: &str) -> Result<String, AppError> {
    // `.trim()` returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character.
    let is_empty_or_whitespace = s.trim().is_empty();

    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', '#', '*', ' '];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    if is_empty_or_whitespace || contains_forbidden_characters {
        Err(AppError::SignupInvalidUsername)
    } else {
        Ok(s.to_string())
    }
}
