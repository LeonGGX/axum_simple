//! /src/handlers/axum_sessions_handlers

use crate::models::user;
use crate::models::user::Role;
use crate::sessions::useful_sessions::{MyReadableSession, MyWritableSession};
use axum::debug_handler;
use axum::response::Redirect;
//use uuid::Uuid;

#[debug_handler]
pub async fn set_session_hdl(mut session: MyWritableSession) -> Redirect {
    //let id = Uuid::new_v4();
    let role = Role::Admin;
    let user = user::User {
        id: 123456789,
        name: "Léon GENGOUX".to_string(),
        password_hash: "password".to_string(),
        role,
    };
    session
        .insert("users", user)
        .expect("Couldn't insert in session");

    Redirect::to("/debug/cookies")
}

#[debug_handler]
pub async fn get_session_hdl(session: MyReadableSession) -> String {
    let str_rep = session.get_raw("users").unwrap_or_default();
    str_rep
}
