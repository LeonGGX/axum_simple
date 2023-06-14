//! /src/handlers/utils_handlers

use crate::askama::askama_tpl::{
    AboutTemplate, HelloTemplate, ListUsersTemplate, NotFoundTemplate, StartTemplate,
    WelcomeTemplate,
};
use crate::authentication::auth_layer::JWTAuthMiddleware;
use crate::db::users::list_users;
use crate::errors::MyAppError;
use crate::AppState;
use askama_axum::Result;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode, Uri};
use axum::{debug_handler, Extension};
use axum_core::response::IntoResponse;
use axum_flash::IncomingFlashes;

/// # Handler
/// Handler for pages not found
/// ## Argument
/// * uri : the uri of the page not found
/// ## Returns
/// StatusCode::NOT_FOUND + Askama template or AppError
#[debug_handler]
pub async fn handler_404(uri: Uri) -> (StatusCode, NotFoundTemplate) {
    let title = "Page non trouvée".to_string();
    let template = NotFoundTemplate { title, uri };
    (StatusCode::NOT_FOUND, template)
}

/// # Handler
/// Handler giving information about the site
/// ## Returns
/// Askama template or AppError
#[debug_handler]
pub async fn about_hdl() -> Result<AboutTemplate, MyAppError> {
    let title = "A propos de ...".to_string();
    let template = AboutTemplate { title };
    Ok(template)
}
///
/// # Handler
/// START PAGE OF THE SITE
/// ## Returns
/// Askama template or AppError
#[debug_handler]
pub async fn start_hdl() -> Result<StartTemplate, MyAppError> {
    let title = "Bienvenue dans l'Application de Gestion des Partitions".to_string();
    let template = StartTemplate { title };
    Ok(template)
}
///
/// # Handler
/// Affiche une page d'accueil lorsque on est loggé
///
/// il faut retourner IncomningFlashes, sinon ils apparaissent sur    
/// la page suivante
///
#[debug_handler]
pub async fn welcome_hdl(
    State(_app): State<AppState>,
    in_flash: IncomingFlashes,
) -> Result<(IncomingFlashes, WelcomeTemplate), MyAppError> {
    let mut flash = String::new();
    for (level, message) in &in_flash {
        flash.push_str(&format!("{:?}: {}", level, message))
    }
    let flash = Some(flash);
    let title = "Commencer à travailler".to_string();
    let template = WelcomeTemplate { title, flash };
    Ok((in_flash, template))
}

#[debug_handler]
pub async fn hello_name_askama_hdl(Path(name): Path<String>) -> impl IntoResponse {
    let title = "Askama".to_string();
    HelloTemplate { title, name }
}

pub async fn favicon() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/png")],
        Bytes::from_static(include_bytes!(
            "D:\\Programmation\\Rust\\mes_programmes\\axum_simple\\static\\images\\rust-logo-white.png")),
    )
}

#[allow(dead_code)]
#[debug_handler]
pub async fn list_users_askama_hdl(State(state): State<AppState>) -> impl IntoResponse {
    let title = "Liste des Utilisateurs".to_string();
    let users = list_users(&state.pool).await.unwrap();
    let flash = None;
    ListUsersTemplate {
        title,
        users,
        flash,
    }
}

#[debug_handler]
pub async fn list_users_with_extension(
    State(state): State<AppState>,
    Extension(auth_jwt): Extension<JWTAuthMiddleware>,
) -> Result<ListUsersTemplate, MyAppError> {
    if auth_jwt.user.role != "Administrateur" {
        Err(MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "Hey ! Page only for Administrators",
        ))
    } else {
        let title = "Liste des Utilisateurs".to_string();
        let users = list_users(&state.pool).await.unwrap();
        let flash = None;
        Ok(ListUsersTemplate {
            title,
            users,
            flash,
        })
    }
}
