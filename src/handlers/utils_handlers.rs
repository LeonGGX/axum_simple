//! /src/handlers/utils_handlers

use crate::askama::askama_tpl::{
    AboutTemplate, HelloTemplate, HtmlTemplate, NotFoundTemplate, StartTemplate,
};
use crate::errors::AppError;
use axum::debug_handler;
use axum::extract::Path;
use axum::http::{StatusCode, Uri};
use axum_core::response::IntoResponse;

/// # Handler
/// Handler for pages not found
/// ## Argument
/// * uri : the uri of the page not found
/// ## Returns
/// StatusCode::NOT_FOUND + Askama template or AppError
pub async fn handler_404(uri: Uri) -> (StatusCode, NotFoundTemplate) {
    let title = "Page non trouvée".to_string();
    let template = NotFoundTemplate { title, uri };
    (StatusCode::NOT_FOUND, template)
}

/// # Handler
/// Handler giving information about the site
/// ## Returns
/// Askama template or AppError
pub async fn about_hdl() -> Result<AboutTemplate, AppError> {
    let title = "A propos de ...".to_string();
    let template = AboutTemplate { title };
    Ok(template)
}

/// # Handler
/// START PAGE OF THE SITE
/// ## Returns
/// Askama template or AppError
pub async fn start_hdl() -> Result<StartTemplate, AppError> {
    let title = "Login".to_string();
    let template = StartTemplate { title };
    Ok(template)
}

#[debug_handler]
pub async fn hello_name_askama_hdl(Path(name): Path<String>) -> impl IntoResponse {
    let title = "Askama".to_string();
    let template = HelloTemplate { title, name };
    //HtmlTemplate(template)
    template
}
