//! src/errors.rs

use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum_flash::Flash;
use password_auth::VerifyError;
use redis::RedisError;
use serde::Serialize;
use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::askama::askama_tpl::ErrorTemplate;
use thiserror::Error;
use validator::ValidationError;

#[derive(Debug)]
pub struct MyAppError {
    pub code: StatusCode,
    pub message: String,
}

impl MyAppError {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<anyhow::Error> for MyAppError {
    fn from(value: anyhow::Error) -> Self {
        let error_message = value.source().unwrap().to_string();
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: error_message,
        }
    }
}

impl From<sqlx::Error> for MyAppError {
    fn from(value: sqlx::Error) -> Self {
        let error_message = value.to_string();
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: error_message,
        }
    }
}

impl From<password_auth::VerifyError> for MyAppError {
    fn from(value: VerifyError) -> Self {
        let error_message = value.source().unwrap().to_string();
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: error_message,
        }
    }
}

impl From<validator::ValidationError> for MyAppError {
    fn from(value: ValidationError) -> Self {
        let error_message = value.source().unwrap().to_string();
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            message: error_message,
        }
    }
}
impl From<redis::RedisError> for MyAppError {
    fn from(value: RedisError) -> Self {
        let error_message = value.detail().unwrap().to_string();
        Self {
            code: StatusCode::UNPROCESSABLE_ENTITY,
            message: error_message,
        }
    }
}

impl Display for MyAppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.message)
    }
}

impl std::error::Error for MyAppError {}

impl IntoResponse for MyAppError {
    fn into_response(self) -> Response {       
        let body = ErrorTemplate {
            title: "Error".to_string(),
            error_message: self.message,
        };
        (self.code, body).into_response()
    }
}


