//! /src/authentication/redis_session.rs

use crate::authentication::jwt::TokenDetails;
use crate::errors::MyAppError;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use redis::AsyncCommands;

pub async fn save_token_data_to_redis(
    State(state): State<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<(), MyAppError> {
    let redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            MyAppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redis error: {}", e),
            )
        });
    redis_client?
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.user_id.to_string(),
            (max_age * 60) as usize,
        )
        .await
        .map_err(|e| MyAppError::new(StatusCode::UNPROCESSABLE_ENTITY, format!("{}", e)))?;
    Ok(())
}
