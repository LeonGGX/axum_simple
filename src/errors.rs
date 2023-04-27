//! src/errors.rs

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use password_auth::VerifyError;
use serde::Serialize;
//use sqlx::Error;
use std::error::Error;

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

impl IntoResponse for MyAppError {
    fn into_response(self) -> Response {
        /* (
            self.code,
            Json(ResponseMessage {
                message: self.message,
            }),
        )
            .into_response() */
        let body = ErrorTemplate {
            title: "Error".to_string(),
            error_message: self.message,
        };
        (self.code, body).into_response()
    }
}

#[derive(Serialize)]
struct ResponseMessage {
    message: String,
}

#[derive(Debug, Clone, Error)]
pub enum AppError {
    //#[error(transparent)]
    //Sqlx(#[from] sqlx::Error),

    //#[error(transparent)]
    //Anyhow(#[from] anyhow::Error),

    //   #[error(transparent)]
    //   AskamaError(#[from] askama::Error),
    #[error("Signup Error: user name exists")]
    SignupUsernameExists,

    #[error("Signup Error : invalid user name")]
    SignupInvalidUsername,

    #[error("Signup Error : empty password")]
    SignupEmptyPassword,

    #[error("Signup Error : the two passwords don't match")]
    SignupPasswordDoNotMatch,

    #[error("Signup Error : invalid password")]
    SignupInvalidPassword,

    #[error("Authentication Error : no auth token")]
    NoAuthTokenError,

    #[error("Authentication Error : error parsing token")]
    AuthTokenParseError,

    #[error("Authentication Error : Ctx not in Request Extension")]
    AuthFailCtxNotInRequestExtensionError,
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            //Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            //Self::AskamaError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SignupUsernameExists => StatusCode::BAD_REQUEST,
            Self::SignupInvalidUsername => StatusCode::BAD_REQUEST,
            Self::SignupEmptyPassword => StatusCode::BAD_REQUEST,
            Self::SignupPasswordDoNotMatch => StatusCode::BAD_REQUEST,
            Self::SignupInvalidPassword => StatusCode::BAD_REQUEST,
            Self::AuthTokenParseError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NoAuthTokenError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AuthFailCtxNotInRequestExtensionError => StatusCode::BAD_REQUEST,
            //_ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            /*Self::Sqlx(ref e) => {
                tracing::error!("SQLx error: {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }
            Self::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }
            Self::AskamaError(ref e) => {
                tracing::error!("Askama error : {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }*/
            Self::SignupUsernameExists => {
                (self.status_code(), Self::SignupUsernameExists.to_string())
            }
            Self::SignupInvalidUsername => {
                (self.status_code(), Self::SignupInvalidUsername.to_string())
            }
            Self::SignupEmptyPassword => {
                (self.status_code(), Self::SignupEmptyPassword.to_string())
            }
            Self::SignupPasswordDoNotMatch => (
                self.status_code(),
                Self::SignupPasswordDoNotMatch.to_string(),
            ),
            Self::SignupInvalidPassword => {
                (self.status_code(), Self::SignupInvalidPassword.to_string())
            }
            Self::AuthTokenParseError => {
                (self.status_code(), Self::AuthTokenParseError.to_string())
            }
            Self::NoAuthTokenError => (self.status_code(), Self::NoAuthTokenError.to_string()),
            Self::AuthFailCtxNotInRequestExtensionError => (
                self.status_code(),
                Self::AuthFailCtxNotInRequestExtensionError.to_string(),
            ), // Other errors get mapped normally.
        };

        /*
        let body = Json(json!({
            "il y a une erreur : ": error_message,
        }));
        */
        let body = ErrorTemplate {
            title: "Error".to_string(),
            error_message,
        };
        (status, body).into_response()
    }
}
