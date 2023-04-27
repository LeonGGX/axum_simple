//! src/authentication/auth_utils.rs

use crate::errors::{AppError, MyAppError};
use axum::http::StatusCode;
//use chrono::{Duration, Utc};

//use crate::models::user::User;
//use dotenvy_macro::dotenv;
use crate::authentication::jwt;
use crate::authentication::jwt::TokenDetails;
//use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
//use once_cell::sync::Lazy;
use password_auth::{generate_hash, verify_password};
//use pbkdf2::password_hash::{PasswordHasher, SaltString};
//use pbkdf2::Pbkdf2;
//use rand_core::OsRng;
//use serde::{Deserialize, Serialize};
/*
static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn create_jwt_token(user: User) -> Result<String, MyAppError> {
    let mut now = Utc::now();
    let iat = now.timestamp() as usize;
    let expires_in: Duration = Duration::minutes(5);
    now += expires_in;
    let exp = now.timestamp() as usize;
    let sub = user.id.to_string();

    let claims = Claims { sub, exp, iat };
    tracing::info!("created Claims : {claims:?}");

    let token = encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error Creating Token"));
    tracing::info!("created Token : {token:?}");
    Ok(token?)
}

pub fn is_token_valid(token: &str) -> Result<(), (jsonwebtoken::errors::Error, StatusCode)> {
    let token_message = decode::<Claims>(token, &KEYS.decoding, &Validation::new(Algorithm::HS256));
    match token_message {
        Ok(tm) => {
            tracing::info!("decoded Token : {tm:?}");
            Ok(())
        }
        Err(err) => match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                return Err((err, StatusCode::UNAUTHORIZED))
            }
            _ => return Err((err, StatusCode::INTERNAL_SERVER_ERROR)),
        },
    }
}

///
/// Utility function to hash passwords with pbkdf2
/// Returns a String with hashed password or anyhow::Error
///
#[allow(dead_code)]
pub async fn hash_password_pbkdf2(password: String) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Hash password to PHC string ($pbkdf2-sha256$...)
    let password_hash = Pbkdf2.hash_password(password.as_bytes(), &salt);

    let hashed_password = if let Ok(password) = password_hash {
        password.to_string()
    } else {
        return Err(AppError::SignupInvalidPassword);
    };
    Ok(hashed_password)
}
*/
///
/// Returns a String with a hashed password
/// uses crate 'password_auth'
///
#[allow(dead_code)]
pub async fn hash_password(password_clear: &str) -> String {
    let hash = generate_hash(password_clear);
    hash
}

#[allow(dead_code)]
pub async fn check_password(password_clear: &str, password_hash: &str) -> Result<(), MyAppError> {
    let result = verify_password(password_clear, password_hash);
    return match result {
        Ok(_) => Ok(()),
        Err(err) => Err(MyAppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            err.to_string(),
        )),
    };
}
/*
#[allow(dead_code)]
pub async fn verify_password_pbkdf2(
    _entered_pw: String,
    _stored_pw_hash: String,
) -> Result<(), AppError> {
    unimplemented!()
}
 */
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
        return Err(AppError::SignupInvalidUsername);
    } else {
        return Ok(s.to_string());
    }
}
