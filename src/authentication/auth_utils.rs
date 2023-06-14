//! src/authentication/auth_utils.rs

use crate::errors::{MyAppError};
use axum::extract::State;
use axum::headers::HeaderMap;
use axum::http::{header, Response, StatusCode};
use axum_core::response::IntoResponse;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;

use crate::authentication::jwt;
use crate::authentication::jwt::{verify_jwt_token, TokenDetails};

use crate::authentication::redis_session::save_token_data_to_redis;
use crate::db::users::find_user_by_id;
use crate::AppState;
use password_auth::{generate_hash, verify_password};
use redis::AsyncCommands;

///
/// Returns a String with a hashed password
/// uses crate 'password_auth'
///
#[allow(dead_code)]
pub async fn hash_password(password_clear: &str) -> String {
    generate_hash(password_clear)
}

#[allow(dead_code)]
pub async fn check_password(password_clear: &str, password_hash: &str) -> bool {
    let result = verify_password(password_clear, password_hash);
    match result {
        Ok(_) => true,
        Err(_err) => false,
    }
}
///
/// Generates a token with the private key    
///
/// access_token_max_age : 10 minutes in the .env file    
/// refresh_token_max_age : 1 hour in the .env file
///
pub fn generate_token(
    user_id: uuid::Uuid,
    user_role: String,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, MyAppError> {
    jwt::generate_jwt_token(user_id, user_role, max_age, private_key).map_err(|e| {
        MyAppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("error generating token: {}", e),
        )
    })
}
///
/// # Handler
///
/// Authentication is implemented with JWT access tokens and refresh tokens.    
/// On successful authentication the API returns a short lived JWT access token that expires after 10 minutes,    
/// and a refresh token that expires after (7 days) 1 hour in an HTTP Only cookie.    
/// The JWT is used for accessing secure routes on the API and the refresh token is used    
/// for generating new JWT access tokens when (or just before) they expire.     
/// HTTP Only cookies are used for refresh tokens to increase security    
/// because they are not accessible to client-side javascript    
/// which prevents XSS (cross site scripting) attacks,     
/// and refresh tokens only have access to generate new JWT tokens (via the /users/refresh-token route)    
/// which prevents them from being used in CSRF (cross site request forgery) attacks.
///
#[allow(dead_code)]
pub async fn refresh_access_token_handler(
    cookie_jar: CookieJar,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, MyAppError> {
    tracing::info!(" ->>    Entering refresh_access_handler");
    // will be used for the logged_in cookie
    //let user_name: String;
    // we get the refresh_token from the cookie_jar
    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| MyAppError::new(StatusCode::FORBIDDEN, "Refresh Token Error"))?;

    // we verify the refresh_token with the public key :
    // if it's OK we retrieve the TokenDetails from it
    // else we return a MyAppError
    let refresh_token_details = match verify_jwt_token(
        state.env.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = format!("Token Verification Error : {e}");
            return Err(MyAppError::new(StatusCode::UNAUTHORIZED, error_response));
        }
    };
    // we fetch the user_id from the data stored in the redis session
    // first, we get a connection from the client in AppState
    let mut redis_client = state.redis_client.get_async_connection().await?;
    //.map_err(|e| MyAppError::from(e))?;

    // we get the stored token_uuid in string format
    let redis_token_user_id = redis_client
        .get::<_, String>(refresh_token_details.token_uuid.to_string())
        .await?;
    //.map_err(|e| MyAppError::from(e))?;

    let user_id_uuid = uuid::Uuid::try_parse(&redis_token_user_id).map_err(|_| {
        MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "Token is invalid or session has expired",
        )
    })?;

    let user = find_user_by_id(user_id_uuid, &state.pool).await?;

    let user = user.ok_or_else(|| {
        MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "The user belonging to this token no longer exists",
        )
    })?;
    let user_name = user.name;
    let access_token_details = generate_token(
        user.id,
        user.role,
        state.env.access_token_max_age,
        state.env.access_token_private_key.to_owned(),
    )?;

    save_token_data_to_redis(
        State(state.clone()),
        &access_token_details,
        state.env.access_token_max_age,
    )
    .await?;

    let access_cookie = Cookie::build(
        "access_token",
        access_token_details.token.clone().unwrap_or_default(),
    )
    .path("/")
    .max_age(time::Duration::minutes(state.env.access_token_max_age * 60))
    .same_site(SameSite::Lax)
    .http_only(true)
    .finish();

    let logged_in_cookie = Cookie::build("logged_in", user_name)
        .path("/")
        .max_age(time::Duration::minutes(state.env.access_token_max_age * 60))
        .same_site(SameSite::Lax)
        .http_only(false)
        .finish();
    // we build a new empty response
    let mut response: Response<String> = Response::default();

    // we build headers and add the access_cookie and the logged_in_cookie
    // to the headers
    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        access_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    // we add the headers to the response
    response.headers_mut().extend(headers);

    // we return the response
    tracing::info!("->>     auth_cookie refreshed");
    Ok(response)
}

///
/// Utility function to parse usernames
/// Returns a String with the parsed username or AppError
///
pub fn parse(s: &str) -> Result<String, MyAppError> {
    // `.trim()` returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character.
    let is_empty_or_whitespace = s.trim().is_empty();

    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', '#', '*', ' '];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    if is_empty_or_whitespace || contains_forbidden_characters {
        Err(MyAppError::new(StatusCode::BAD_REQUEST, "Invalid Entry"))
    } else {
        Ok(s.to_string())
    }
}
