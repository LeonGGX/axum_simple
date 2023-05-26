//! src/authentication/auth_layer.rs

use crate::authentication::jwt::verify_jwt_token;
use crate::db::users::find_user_by_id;
use crate::errors::MyAppError;
use crate::models::user::User;
use crate::AppState;
use axum::extract::State;
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    Extension,
};
use axum_core::response::IntoResponse;
use axum_extra::extract::cookie::CookieJar;
use hyper::header;
use redis::{AsyncCommands, FromRedisValue};
use serde::{Deserialize, Serialize};

///
/// Struct that contains :    
/// - user : User    
/// - auth_token_uuid
///
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: User,
    pub auth_token_uuid: uuid::Uuid,
}

///
/// # Layer
/// To be used with AUTHORIZATION Header or Cookies    
/// Used to check if the authorization cookie exists in the request   
/// send to the route.    
/// It checks if the cookie or Authorization header exists    
/// in the request cookie jar or header.    
/// It verifies if the token is valid
///
#[allow(dead_code)]
pub async fn auth<B>(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, MyAppError> {
    tracing::info!("Entering auth middleware");
    // Fetching an auth_token among the cookies in cookiejar and stringify it
    // if no cookie is found, we check the header AUTHORIZATION and parse it
    let auth_token = cookie_jar
        .get("auth_token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    //if auth_value.starts_with("Bearer ") {
                    //Some(auth_value[7..].to_owned())
                    if let Some(auth_value) = auth_value.strip_prefix("Bearer ") {
                        Some(auth_value.to_owned())
                    } else {
                        None
                    }
                })
        });
    // if there is no auth_token in cookies or header AUTHORIZATION
    // we show an error
    let auth_token = auth_token.ok_or_else(|| {
        MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "You are not logged in, Please provide token",
        )
    })?;

    // if an auth_token is found, let's verify it and return a TokenDetails struct :
    // pub struct TokenDetails {
    //    pub token: Option<String>,
    //    pub token_uuid: uuid::Uuid,
    //    pub user_id: uuid::Uuid,
    //    pub user_role: String,
    //    pub expires_in: Option<i64>,
    // }
    let auth_token_details =
        verify_jwt_token(state.env.access_token_public_key.to_owned(), &auth_token).map_err(
            |err| {
                MyAppError::new(
                    StatusCode::UNAUTHORIZED,
                    format!("Error verifying token : {:?}", err),
                )
            },
        )?;
    // let's get the token_uuid from the retrieved TokenDetails
    let auth_token_uuid = uuid::Uuid::try_parse(&auth_token_details.token_uuid.to_string())
        .map_err(|_| MyAppError::new(StatusCode::UNAUTHORIZED, "Error : Invalid token"))?;

    // get a connection to the redis DB with our redis_client from the AppState
    let mut redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .map_err(|err| {
            // we can use ::from with MyAppError because it is implemented for RedisError
            MyAppError::from(err)
        })?;
    // get the token_user_uuid from the redis DB using the auth_token_uuid
    let redis_token_user_id = redis_client
        .get::<_, String>(auth_token_uuid.clone().to_string())
        .await
        .map_err(|err| MyAppError::from(err))?;
    // transform the redis_token_id which is a string to an uuid
    let user_id_uuid = uuid::Uuid::try_parse(&redis_token_user_id).map_err(|_| {
        MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "Token is invalid or session has expired",
        )
    })?;
    // let's fetch a user with the uuid
    // we must return an Option ...
    let user = find_user_by_id(user_id_uuid, &state.pool)
        .await
        .map_err(|e| MyAppError::from(e))?;
    // Si l'Option n'est pas None et contient un User on continue
    // sinon, on renvoie une erreur précisant que le token n'existe plus
    let user = user.ok_or_else(|| {
        MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "Error : The user belonging to this token no longer exists".to_string(),
        )
    })?;
    req.extensions_mut().insert(JWTAuthMiddleware {
        user,
        auth_token_uuid,
    });

    Ok(next.run(req).await)
}

///
/// # Layer
/// restricts the route to admin routes
/// to be used AFTER the auth layer    
/// it takes the request already filtered by the auth layer    
/// filtering the authenticated user ans checks if the user is administrator   
///
/// This layer could be replaced by using Extension(JWTAuthMiddelware)as argument
/// in the handler that only allows an administrator and check the user.role.   
///
/// Here two possibilities :    
/// use Extension(JWTAuthMiddleware) in the arguments and check its user.role    
/// or :    
/// let role = req    
///        .extensions()    
///         .get::<JWTAuthMiddleware>()    
///         .map(|ext| ext.user.role.clone())    
///         .ok_or_else(|| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, "No Role found"))?;    
/// and the Extension argument is not necessary
///
#[allow(dead_code)]
pub async fn auth_admin<B>(
    State(state): State<AppState>,
    //cookie_jar: CookieJar,
    Extension(auth_ext): Extension<JWTAuthMiddleware>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, MyAppError> {
    tracing::info!("Entering auth_admin middleware");
    /*let role = req
        .extensions()
        .get::<JWTAuthMiddleware>()
        .map(|ext| ext.user.role.clone())
        .ok_or_else(|| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, "No Role found"))?;
    */
    let role = auth_ext.user.role;
    if role == "Administrateur" {
        Ok(next.run(req).await)
    } else {
        Err(MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "Page Only For Administrators",
        ))
    }
}
