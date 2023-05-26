//! src/handlers/logout_handlers.rs

use crate::askama::askama_tpl::LogoutTemplate;
use crate::authentication::auth_layer::JWTAuthMiddleware;
use crate::authentication::jwt::verify_jwt_token;
use crate::errors::MyAppError;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::{debug_handler, Extension};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;
use tower_cookies::cookie::time;

#[debug_handler]
pub async fn logout_page() -> LogoutTemplate {
    let title = "Se déconnecter".to_string();
    LogoutTemplate { title }
}

///
/// # Handler
/// Logs the current user out    
/// Deletes all identification cookies (auth_token, refresh_token)    
/// Sets the logged_in cookie to false   
/// Deletes data from redis session   
///
#[debug_handler]
pub async fn logout_handler(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    Extension(auth_guard): Extension<JWTAuthMiddleware>,
) -> Result<(CookieJar, Redirect), MyAppError> {
    tracing::info!("Entering LOGOUT_HANDLER");
    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            MyAppError::new(
                StatusCode::FORBIDDEN,
                "Token is invalid or session has expired",
            )
        })?;
    let refresh_token_details = verify_jwt_token(
        state.env.refresh_token_public_key.to_owned(),
        &refresh_token,
    )
    .map_err(|err| {
        MyAppError::new(
            StatusCode::UNAUTHORIZED,
            format!("Error verifying token : {:?}", err),
        )
    })?;

    let mut redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| MyAppError::from(e))?;

    redis_client
        .del(&[
            refresh_token_details.token_uuid.to_string(),
            auth_guard.auth_token_uuid.to_string(),
        ])
        .await
        .map_err(|e| MyAppError::from(e))?;
    // brings auth_token to null
    let access_cookie = Cookie::build("auth_token", "")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    // brings refresh_token to null
    let refresh_cookie = Cookie::build("refresh_token", "")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();
    // brings logged_in cookie to false
    let logged_in_cookie = Cookie::build("logged_in", "false")
        .path("/")
        .max_age(time::Duration::minutes(-1))
        .same_site(SameSite::Lax)
        .http_only(false)
        .finish();

    let cookiejar = cookie_jar
        .add(access_cookie)
        .add(refresh_cookie)
        .add(logged_in_cookie);

    Ok((cookiejar, Redirect::to("/")))
}
