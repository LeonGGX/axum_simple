//! /src/authentication/redis_session.rs
//!
//! Only contains the function "save_token_data_to_redis"
//!
//! to check redis session with windows 10
//! make sure the redis server is running : it should be or the server
//! would have sent an error at the start
//!
//! in your Powershell type "redis-cli" and then "keys *" and you should
//! get the list of keys  sent to our redis session.
//! to get the data you should type "get + key value" you want to find
//! now it returns the user ID (Uuid in String form)
//!

use crate::authentication::jwt::TokenDetails;
use crate::errors::MyAppError;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use redis::{AsyncCommands, Commands};

///
/// Saves the token ID and the User ID to Redis    
/// Returns an empty Ok Result (()) or MyappError    
/// Args :    
/// - the AppState
/// - token_details : a TokenDetails struct
/// - max_age : i64
///
pub async fn save_token_data_to_redis(
    State(state): State<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<(), MyAppError> {
    // first get an async connection from the Redis Client in AppState
    let mut redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| MyAppError::from(e))?;
    redis_client
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            (max_age * 60) as usize, // 15 minutes
        )
        .await
        .map_err(|err| MyAppError::from(err))?;

    Ok(())
}
