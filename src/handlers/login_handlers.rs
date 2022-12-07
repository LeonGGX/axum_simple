//! /src/handlers/login_handlers

use crate::askama::askama_tpl::{HtmlTemplate, LoginTemplate};
use crate::sessions::useful_sessions::MyWritableSession;
use crate::AppState;
use axum::debug_handler;
use axum::extract::State;
use axum::response::Redirect;
use axum::Form;
use axum_flash::{Flash, IncomingFlashes};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

/// # Handler
///
/// affiche la page de login\
/// affiche les messages flash\
///
/// the flash must be returned so the cookie is removed
///
#[debug_handler]
pub async fn login_form_askama_hdl(
    State(_app): State<AppState>,
    in_flash: IncomingFlashes,
) -> (IncomingFlashes, HtmlTemplate<LoginTemplate>) {
    /*
    let str_flash = flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", str_flash);
    */
    let mut flash = String::new();
    for (level, message) in &in_flash {
        flash.push_str(&*format!("{:?}: {}", level, message))
    }
    let title = "Login - S'identifier".to_string();
    let flash = Some(flash);
    let template = LoginTemplate { title, flash };
    (in_flash, HtmlTemplate(template))
}

/// # post_login_hdl
/// writes into a session en redirects to "/auth/login"\
/// Since parsing form data requires consuming the request body, the `Form` extractor must be
/// *last* if there are multiple extractors in a handler. See ["the order of extractors"][order-of-extractors]

#[debug_handler]
pub async fn post_login_hdl(
    State(_state): State<AppState>,
    mut session: MyWritableSession,
    flash: Flash,
    Form(login_payload): Form<LoginPayload>,
) -> (Flash, Redirect) {
    if login_payload.username.is_empty() {
        return (
            flash.error("le nom d'utilisateur manque !"),
            Redirect::to("/auth/login"),
        );
    }
    if login_payload.password.is_empty() {
        return (
            flash.error("le not de passe manque !"),
            Redirect::to("/auth/login"),
        );
    }

    let login_user = login_payload.username;
    session
        .insert("users", login_user)
        .expect("Couldn't insert in session");

    (
        flash.success("Vous êtes loggé !"),
        Redirect::to("/auth/login"),
    )
}
