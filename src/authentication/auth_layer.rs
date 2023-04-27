//! src/authentication/auth_layer.rs

use crate::authentication::jwt;
use crate::db::users::find_user_by_id;
use crate::errors::MyAppError;
use crate::AppState;
use axum::extract::State;
//use axum::headers::authorization::Bearer;
//use axum::headers::Authorization;
use crate::models::user::User;
use axum::{
    //extract::TypedHeader,
    http::{Request, StatusCode},
    middleware::Next,
    //response::Response,
};
use axum_core::response::IntoResponse;
use axum_extra::extract::cookie::CookieJar;
use hyper::header;
use redis::AsyncCommands;
//use redis::AsyncCommands;
use crate::authentication::jwt::verify_jwt_token;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: User,
    pub auth_token_uuid: uuid::Uuid,
}

///
/// To be used with AUTHORIZATION Header or Cookies
///
#[allow(dead_code)]
pub async fn auth<B>(
    State(state): State<AppState>,
    cookie_jar: CookieJar,
    mut req: Request<B>,
    next: Next<B>,
    //) -> Result<Response, StatusCode> {
) -> Result<impl IntoResponse, MyAppError> {
    tracing::info!("AUTH LAYER -- AUTH");
    let auth_token = cookie_jar
        .get("auth_token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });
    let auth_token = auth_token.ok_or_else(|| {
        let error = MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "You are not logged in, Please provide token",
        );
        error
    })?;
    let auth_token_details =
       /* match jwt::verify_jwt_token(state.env.access_token_public_key.to_owned(), &auth_token) {
            Ok(token_details) => token_details,
            Err(err) => Err(|err| {
                MyAppError::new(
                    StatusCode::UNAUTHORIZED,
                    format!("Error verifying token : {:?}", err),
                )
            }),
        };*/
    verify_jwt_token(state.env.access_token_public_key.to_owned(), &auth_token)
        .map_err(|err|MyAppError::new(
        StatusCode::UNAUTHORIZED,
        format!("Error verifying token : {:?}", err),
    ))?;
    let auth_token_uuid = uuid::Uuid::parse_str(&auth_token_details.token_uuid.to_string())
        .map_err(|_| MyAppError::new(StatusCode::UNAUTHORIZED, "Error : Invalid token"))?;
    let mut redis_client = state
        .redis_client
        .get_async_connection()
        .await
        .map_err(|err| {
            MyAppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redis error: {}", err),
            )
        })?;
    let redis_token_user_id = redis_client
        .get::<_, String>(auth_token_uuid.clone().to_string())
        .await
        .map_err(|_| {
            MyAppError::new(
                StatusCode::UNAUTHORIZED,
                "Token is invalid or session has expired",
            )
        })?;
    let user_id_uuid = uuid::Uuid::parse_str(&redis_token_user_id).map_err(|_| {
        MyAppError::new(
            StatusCode::UNAUTHORIZED,
            "Token is invalid or session has expired",
        )
    })?;

    let user = find_user_by_id(user_id_uuid, &state.pool)
        .await
        .map_err(|e| {
            MyAppError::new(
                StatusCode::UNAUTHORIZED,
                "The user belonging to this token no longer exists",
            )
        })?;
    req.extensions_mut().insert(JWTAuthMiddleware {
        user,
        auth_token_uuid,
    });

    /*
         .map_err(|err| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")));
     let user = user.or_else({
         |_| {
             MyAppError::new(
                 StatusCode::UNAUTHORIZED,
                 "The user belonging to this token no longer exists",
             )
         }
     })?;
    */
    Ok(next.run(req).await)

    /*let claims = jwt::verify_jwt(&auth_token)
        .map_err(|e| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.message))?;
    tracing::info!("Claims verified : {claims:?}");
    let user_id = claims.sub;
    let user = find_user_by_id(user_id as i32, &state.pool)
        .await
        .map_err(|e| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)*/
}
