//! /src/handlers/login_handlers

use axum::async_trait;
use axum::debug_handler;
use axum::extract::rejection::FormRejection;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Redirect;
use axum::Form;
use axum_core::extract::FromRequest;
use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::cookie::SameSite;
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_flash::{Flash, IncomingFlashes};

use serde::de::DeserializeOwned;
use serde::Deserialize;

use thiserror::Error;

use validator::Validate;

use crate::askama::askama_tpl::LoginTemplate;
use crate::authentication::auth_utils::{check_password, generate_token};
use crate::authentication::jwt::TokenDetails;
use crate::authentication::redis_session::save_token_data_to_redis;
use crate::db::users::find_user_by_name;
use crate::AppState;

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
        flash.push_str(&format!("{:?}: {}", level, message))
    }
    let title = "Login - S'identifier".to_string();
    let flash = Some(flash);
    let template = LoginTemplate { title, flash };
    (in_flash, template)
}

/// # post_login_hdl
///
/// logs the user in.
/// checks the existence of the username in the DB    
/// checks the validity of the password    
/// creates two token (access and refresh) and stores them in cookies    
/// adds the user into a redis session    
/// redirects to "api/welcome" if successful or returns to "auth/login" with failure flash message    
///
/// Since parsing form data requires consuming the request body, the `Form` extractor must be
/// *last* if there are multiple extractors in a handler.   
/// See ["the order of extractors"][order-of-extractors]
///
/// Normally start with verifying if the password matches before verifying
/// the presence of a matching username.
///
#[debug_handler]
pub async fn post_login_hdl(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    flash: Flash,
    ValidatedLoginForm(input): ValidatedLoginForm<CreateLoginInput>,
) -> Result<(CookieJar, Flash, Redirect), (Flash, Redirect)> {
    println!("->> {:<12}  - post_login_hdl", "HANDLER");

    // the following three variables must be known all along the handler
    let access_token_details: TokenDetails;
    let refresh_token_details: TokenDetails;
    let cookiejar: CookieJar;
    /*let mut access_token_details = TokenDetails {
        token: Some("".to_string()),
        ..Default::default()
    };*/
    /*let mut refresh_token_details = TokenDetails {
        token: Some("".to_string()),
        ..Default::default()
    };
    let mut cookiejar = cookie_jar.clone();*/

    let result_opt_user = find_user_by_name(input.name, &state.pool).await;
    match result_opt_user {
        // 1. the DB sent an OK Result with an Option<User>
        Ok(opt_user) => {
            // let's check if the Option<User> is None : no such user in the DB ...
            // we can't use '.ok_or_else()' to get the User in the Option here
            // because ? can't be used with (Flash, Redirect) as error.
            if opt_user.is_none() {
                let message = "The User doesn't exist ! Please sign in ...".to_string();
                return Err((flash.error(message), Redirect::to("/auth/login")));
            } else {
                // we know there is a user in the DB : the Option<User> is Some
                // let's get the User inside the Option (we can unwrap: we know there is data)
                let user = opt_user.unwrap();
                // first check password
                let verify_pw = check_password(&input.password, &user.password).await;
                if !verify_pw {
                    // is the password is wrong ...
                    let message = "Password problem !".to_string();
                    return Err((flash.error(message), Redirect::to("/auth/login")));
                } else {
                    // the password is ok
                    let access_token_dtls = generate_token(
                        user.id,
                        state.env.access_token_max_age,
                        state.env.access_token_private_key.to_owned(),
                    );
                    match access_token_dtls {
                        Err(_) => {
                            let message = "Token Error !".to_string();
                            return Err((flash.error(message), Redirect::to("/auth/login")));
                        }
                        Ok(a_t_d) => {
                            access_token_details = TokenDetails::new(
                                a_t_d.token,
                                a_t_d.token_uuid,
                                a_t_d.user_id,
                                a_t_d.expires_in,
                            );
                        }
                    }
                    let refresh_token_dtls = generate_token(
                        user.id,
                        state.env.refresh_token_max_age,
                        state.env.refresh_token_private_key.to_owned(),
                    );
                    match refresh_token_dtls {
                        Err(_) => {
                            let message = "Token Error !".to_string();
                            return Err((flash.error(message), Redirect::to("/auth/login")));
                        }
                        Ok(r_t_d) => {
                            refresh_token_details = TokenDetails::new(
                                r_t_d.token,
                                r_t_d.token_uuid,
                                r_t_d.user_id,
                                r_t_d.expires_in,
                            );
                        }
                    }
                    let save_access_token = save_token_data_to_redis(
                        State(state.clone()),
                        &access_token_details,
                        state.clone().env.access_token_max_age,
                    )
                    .await;
                    match save_access_token {
                        Err(_err) => {
                            let message = "Redis saving access token Error: {err} !".to_string();
                            return Err((flash.error(message), Redirect::to("/auth/login")));
                        }
                        Ok(_s_a_t) => (),
                    };

                    let save_refresh_token = save_token_data_to_redis(
                        State(state.clone()),
                        &refresh_token_details,
                        state.clone().env.refresh_token_max_age,
                    )
                    .await;
                    match save_refresh_token {
                        Err(_) => {
                            let message = "Redis saving refresh token Error !".to_string();
                            return Err((flash.error(message), Redirect::to("/auth/login")));
                        }
                        Ok(s) => s,
                    };
                } // end password OK
            }
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

            cookiejar = cookie_jar
                .add(access_cookie)
                .add(refresh_cookie)
                .add(logged_in_cookie);
        }
        // 1. the DB sent an Err Result (there was an error with the DB query)
        Err(err) => {
            let message = format!("DatabaseError: {err}");
            return Err((flash.error(message), Redirect::to("/auth/login")));
        }
    }
    let message = "You are logged in".to_string();
    Ok((
        cookiejar.clone(),
        flash.success(message),
        Redirect::to("/api/welcome"),
    ))
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
                    flash: Some(format!("{}", err)),
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
