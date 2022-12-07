//! src/errors.rs

//use crate::AppError::Tera;
//use crate::{globals, AppState};
//use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
//use tera::Tera;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    //#[error("Page non trouvée")]
    //NotFound,
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
            //Self::NotFound => StatusCode::NOT_FOUND,
            Self::Sqlx(_) | Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Tera(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

            Self::Tera(ref e) => {
                tracing::error!("Tera error : {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }
            Self::AskamaError(ref e) => {
                tracing::error!("Askama error : {:?}", e);
                let message = e.to_string();
                (self.status_code(), message)
            }

            // Other errors get mapped normally.
            //_ => (
                //StatusCode::UNPROCESSABLE_ENTITY,
                //"une autre erreur !".to_string(),
            //),
        };

        let body = Json(json!({
            "il y a une erreur : ": error_message,
        }));

        (status, body).into_response()
    }
}
/*
pub fn handler_error(
    State(state): State<AppState>,
    error_message: String,
    //) -> Result<Html<String>, (StatusCode, &'static str)> {
) -> Result<Html<String>, AppError> {
    let title = "Erreur";

    let mut ctx = tera::Context::new();
    ctx.insert("title", title);
    ctx.insert("error_message", &error_message);
    let body = state.templates.render("error.html", &ctx)?;
    Ok(Html(body))
}
*/
