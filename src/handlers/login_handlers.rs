//! /src/handlers/login_handlers

use crate::askama::askama_tpl::LoginTemplate;
use crate::authentication::auth_utils::{check_password, generate_token};
//use crate::authentication::jwt::TokenClaims;
use crate::db::users::find_user_by_name;
//use std::time;

//use ::time::Duration;
//use crate::errors::MyAppError;
//use crate::models::user::User;
use crate::AppState;
//use anyhow::Error;
use axum::async_trait;
use axum::debug_handler;
use axum::extract::rejection::FormRejection;
use axum::extract::State;
//use axum::headers::{Authorization, Cookie, HeaderValue};
//use axum::http::header::AUTHORIZATION;
//use axum::headers::HeaderMap;
use axum::http::{Request, StatusCode};
use axum::response::Redirect;
use axum::Form;
use axum_core::extract::FromRequest;
use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::cookie::SameSite;
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_flash::{Flash, IncomingFlashes};
//use axum_sessions::extractors::WritableSession;
use serde::de::DeserializeOwned;
//use serde::de::Unexpected::Option;
use serde::Deserialize;
//use std::fmt::{Display, Formatter};
use thiserror::Error;

//use tower_cookies::Cookies;
use crate::authentication::redis_session::save_token_data_to_redis;
use crate::errors::MyAppError;
use validator::Validate;

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
) -> (IncomingFlashes, LoginTemplate) {
    let mut flash = String::new();
    for (level, message) in &in_flash {
        flash.push_str(&*format!("{:?}: {}", level, message))
    }
    let title = "Login - S'identifier".to_string();
    let flash = Some(flash);
    let template = LoginTemplate { title, flash };
    (in_flash, template)
}

/// # post_login_hdl
/// writes into a session en redirects to "/welcome" or to" auth/login" in case of error\
/// Since parsing form data requires consuming the request body, the `Form` extractor must be
/// *last* if there are multiple extractors in a handler.   
/// See ["the order of extractors"][order-of-extractors]
///
/// Normally start by verifying if the password matches before verifying
/// the presence of a matching username.
///
#[debug_handler]
pub async fn post_login_hdl(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    flash: Flash,
    ValidatedLoginForm(input): ValidatedLoginForm<CreateLoginInput>,
) -> Result<(CookieJar, Flash, Redirect), MyAppError> {
    println!("->> {:<12}  - post_login_hdl", "HANDLER");
    match find_user_by_name(input.name, &state.pool).await {
        Ok(user) => match check_password(&input.password, &user.password).await {
            Ok(_) => {
                let access_token_details = generate_token(
                    user.id,
                    state.env.access_token_max_age,
                    state.env.access_token_private_key.to_owned(),
                )?;
                let refresh_token_details = generate_token(
                    user.id,
                    state.env.refresh_token_max_age,
                    state.env.refresh_token_private_key.to_owned(),
                )?;
                save_token_data_to_redis(
                    State(state.clone()),
                    &access_token_details,
                    state.clone().env.access_token_max_age,
                )
                .await?;

                save_token_data_to_redis(
                    State(state.clone()),
                    &refresh_token_details,
                    state.clone().env.refresh_token_max_age,
                )
                .await
                .map_err(|err| {
                    MyAppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Redis Error {err:?}"),
                    )
                })?;

                let access_cookie = Cookie::build(
                    "auth_token",
                    access_token_details.token.clone().unwrap_or_default(),
                )
                .path("/")
                .max_age(::time::Duration::minutes(
                    state.env.access_token_max_age * 60,
                ))
                .same_site(SameSite::Lax)
                .http_only(true)
                .finish();

                let refresh_cookie = Cookie::build(
                    "refresh_token",
                    refresh_token_details.token.unwrap_or_default(),
                )
                .path("/")
                .max_age(::time::Duration::minutes(
                    state.env.refresh_token_max_age * 60,
                ))
                .same_site(SameSite::Lax)
                .http_only(true)
                .finish();

                let logged_in_cookie = Cookie::build("logged_in", "true")
                    .path("/")
                    .max_age(::time::Duration::minutes(
                        state.env.access_token_max_age * 60,
                    ))
                    .same_site(SameSite::Lax)
                    .http_only(false)
                    .finish();
                /*
                let mut headers = HeaderMap::new();
                headers.append(
                    header::SET_COOKIE,
                    access_cookie.to_string().parse().unwrap(),
                );
                headers.append(
                    header::SET_COOKIE,
                    refresh_cookie.to_string().parse().unwrap(),
                );
                headers.append(
                    header::SET_COOKIE,
                    logged_in_cookie.to_string().parse().unwrap(),
                );

                response.headers_mut().extend(headers);
                Ok(response)
                 */
                let cookiejar = cookie_jar
                    .add(access_cookie)
                    .add(refresh_cookie)
                    .add(logged_in_cookie);

                let message = format!("{}, vous êtres loggé !", user.name.clone());
                Ok((
                    cookiejar.clone(),
                    flash.success(message),
                    Redirect::to("/api/welcome"),
                ))
            }
            Err(e) => {
                tracing::error!("erreur de login : {:?}", e);
                let message = "Utilisateur ou Mot de Passe invalide".to_string();
                Ok((
                    cookie_jar,
                    flash.error(message),
                    Redirect::to("/auth/login"),
                ))
            }
        },
        Err(e) => {
            println!("->> {:<12}  - post_login_hdl", "HANDLER");
            tracing::error!("erreur de login : {:?}", e);
            let message = "Utilisateur ou Mot de Passe invalide".to_string();
            Ok((
                cookie_jar,
                flash.error(message),
                Redirect::to("/auth/login"),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedLoginForm<T>(pub T);

#[derive(Debug, Error)]
pub enum LoginFormError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
}

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedLoginForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, B, Rejection = FormRejection>,
    B: Send + 'static,
{
    type Rejection = LoginFormError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedLoginForm(value))
    }
}

impl IntoResponse for LoginFormError {
    fn into_response(self) -> Response {
        match self {
            LoginFormError::ValidationError(v) => {
                tracing::error!("erreur de login : {}", v);
                let template = LoginTemplate {
                    title: "App - Login|Error".to_string(),
                    flash: Some("Nom d'utilisateur ou mot de passe incorrect !".to_string()),
                };
                (StatusCode::BAD_REQUEST, template)
            }
            LoginFormError::AxumFormRejection(err) => {
                tracing::error!("erreur de login : {}", err);
                let template = LoginTemplate {
                    title: "App - Login|Error".to_string(),
                    flash: Some(format!("{}", err.to_string())),
                };
                (StatusCode::BAD_REQUEST, template)
            }
        }
        .into_response()
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateLoginInput {
    #[validate(length(min = 4, max = 20))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 6))]
    pub password: String,
}
