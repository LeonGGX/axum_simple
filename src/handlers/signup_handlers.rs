//! src/handlers/signup_handlers.rs

use crate::askama::askama_tpl::SignupTemplate;
//use crate::errors::MyAppError;
use crate::AppState;
use axum::extract::rejection::FormRejection;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Redirect;
use axum::{async_trait, debug_handler, Form};
use axum_core::extract::FromRequest;
use axum_core::response::{IntoResponse, Response};
use axum_extra::extract::cookie::SameSite;
use axum_extra::extract::{cookie::Cookie, CookieJar};
use axum_flash::{Flash, IncomingFlashes};
//use axum_sessions::extractors::WritableSession;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;
use validator::{Validate, ValidationError};

use crate::authentication::auth_utils::{generate_token, hash_password};
//use crate::authentication::jwt::TokenClaims;
use crate::db::users::{add_user, find_user_by_email /*find_user_by_name*/};
//use crate::handlers::login_handlers::LoginFormError::ValidationError;
use crate::models::user::NewUser;
//use crate::sessions::useful_sessions::MyWritableSession;

// # Handler
///
/// affiche la page de login\
/// affiche les messages flash\
///
/// the flash must be returned so the cookie is removed
///
#[debug_handler]
pub async fn signup_form_askama_hdl(
    State(_app): State<AppState>,
    in_flash: IncomingFlashes,
) -> (IncomingFlashes, SignupTemplate) {
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
    let title = "Signup - S'inscrire comme utilisateur".to_string();
    let flash = Some(flash);
    let template = SignupTemplate { title, flash };
    (in_flash, template)
}

#[debug_handler]
pub async fn post_signup_hdl(
    State(state): State<AppState>,
    //mut session: WritableSession,
    flash: Flash,
    cookie_jar: CookieJar,
    ValidatedSignupForm(input): ValidatedSignupForm<CreateSignupInput>,
) -> (CookieJar, Flash, Redirect) {
    tracing::debug!("{:#?}", input);

    // we start by hashing the password
    let hash = hash_password(&input.password).await;
    tracing::debug!("{hash}");

    // we construct a NewUser with password hashed
    let new_user = NewUser {
        name: input.name,
        email: input.email,
        password: hash,
        role: input.role,
    };
    tracing::info!("{:#?}", new_user);

    // check if the user already exists
    // if so signup terminated and return to login
    if let Ok(user) = find_user_by_email(new_user.email.clone(), &state.pool).await {
        let message = format!("L'Utilisateur {} existe déjà !", user.name);
        (
            cookie_jar,
            flash.error(message),
            Redirect::to("/auth/signup"),
        )
    } else {
        // if the user doesn't exist we add him to the DB
        // and invite him to log in
        match add_user(&new_user, &state.pool).await {
            Ok(user) => {
                let message = format!(
                    "Bonjour {}, vous êtes enregistré, prière de vous logger",
                    user.name
                );
                (
                    cookie_jar,
                    flash.success(message),
                    Redirect::to("/auth/login"),
                )
                /*
                let user_clone = user.clone();
                let auth_token = generate_token(
                    user_clone.id,
                    state.env.access_token_max_age,
                    state.env.access_token_private_key.to_owned(),
                )
                .unwrap();
                let cloned_token = auth_token.token.clone();
                println!(
                    "->> {:<12}  - post_signup_hdl : {cloned_token:?}",
                    "token created"
                );

                let cookie =
                    Cookie::build("auth_token", auth_token.token.clone().unwrap_or_default())
                        .path("/")
                        .max_age(::time::Duration::minutes(
                            state.env.access_token_max_age * 60,
                        ))
                        .same_site(SameSite::Lax)
                        .http_only(true)
                        .finish();
                let cookiejar = cookie_jar.add(cookie);
                let message = format!("Bonjour {}, vous êtes enregistré et loggé", user.name);
                (
                    cookiejar,
                    flash.success(message),
                    Redirect::to("/api/welcome"),
                )
                 */
            }
            Err(e) => {
                let message = format!("vous n'êtes pas enregistré {:?}!", e);
                (
                    cookie_jar,
                    flash.error(message),
                    Redirect::to("/auth/signup"),
                )
            }
        }
    }
}

pub fn validate_username(s: &str) -> Result<(), ValidationError> {
    // `.trim()` returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character.
    let is_empty_or_whitespace = s.trim().is_empty();

    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', '#', '*', ' '];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    if is_empty_or_whitespace || contains_forbidden_characters {
        return Err(ValidationError::new(
            "Le nom d'utilisateur est vide ou contient des caractères interdits",
        ));
    } else {
        return Ok(());
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateSignupInput {
    #[validate(length(min = 4, max = 10), custom = "validate_username")]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(must_match = "confirm_pwd")]
    #[validate(length(min = 6))]
    pub password: String,
    #[validate(must_match(other = "confirm_pwd"))]
    pub confirm_pwd: String,
    pub role: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedSignupForm<T>(pub T);

#[derive(Debug, Error)]
pub enum SignupFormError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    AxumFormRejection(#[from] FormRejection),
}

#[async_trait]
impl<T, S, B> FromRequest<S, B> for ValidatedSignupForm<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Form<T>: FromRequest<S, B, Rejection = FormRejection>,
    B: Send + 'static,
{
    type Rejection = SignupFormError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Form(value) = Form::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedSignupForm(value))
    }
}

impl IntoResponse for SignupFormError {
    fn into_response(self) -> Response {
        match self {
            SignupFormError::ValidationError(v) => {
                let template = SignupTemplate {
                    title: "App - Signup|Error".to_string(),
                    flash: Option::from(v.to_string()),
                };
                (StatusCode::BAD_REQUEST, template)
            }
            SignupFormError::AxumFormRejection(err) => {
                let template = SignupTemplate {
                    title: "App - Signup|Error".to_string(),
                    flash: Option::from(err.to_string()),
                };
                (StatusCode::BAD_REQUEST, template)
            }
        }
        .into_response()
    }
}
