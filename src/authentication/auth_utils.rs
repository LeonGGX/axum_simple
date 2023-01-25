//! src/authentication/auth_utils.rs

use crate::errors::AppError;
use pbkdf2::password_hash::{PasswordHasher, SaltString};
use pbkdf2::Pbkdf2;
use rand_core::OsRng;

///
/// Utility function to hash passwords with pbkdf2
/// Returns a String with hashed password or anyhow::Error
///
#[allow(dead_code)]
pub async fn hash_password_pbkdf2(password: String) -> Result<String, SignupError> {
    let salt = SaltString::generate(&mut OsRng);

    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt)?;

    let hashed_password = if let Ok(password) = password_hash {
        password.to_string()
    } else {
        return Err(SignupError::InvalidPassword);
    };
    Ok(hashed_password)
}

#[allow(dead_code)]
pub async fn verify_password_pbkdf2(
    entered_pw: String,
    stored_pw_hash: String,
) -> Result<(), AppError> {
}
