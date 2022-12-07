//! src/errors.rs

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
   
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Tera(#[from] tera::Error),

    #[error(transparent)]
    AskamaError(#[from] askama::Error),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {           
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,           
            Self::AskamaError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Sqlx(ref e) => {
                // TODO: we probably want to use `tracing` instead
                // so that this gets linked to the HTTP request by `TraceLayer`.
                tracing::error!("SQLx error: {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }

            Self::Anyhow(ref e) => {
                // TODO: we probably want to use `tracing` instead
                // so that this gets linked to the HTTP request by `TraceLayer`.
                tracing::error!("Generic error: {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }
           
            Self::AskamaError(ref e) => {
                tracing::error!("Askama error : {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }           
        };

        let body = Json(json!({
            "il y a une erreur : ": error_message,
        }));

        (status, body).into_response()
    }
}
