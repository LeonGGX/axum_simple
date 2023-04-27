//! src/authentication/auth_middleware.rs

use crate::authentication::AUTH_TOKEN;
use crate::ctx::Ctx;
use crate::errors::{AppError, MyAppError};
use async_trait::async_trait;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::http::Request;
use axum::middleware::Next;
use axum_core::extract::FromRequestParts;
use axum_core::response::Response;

use futures::TryFutureExt;
use lazy_regex::regex_captures;
use tower_cookies::Cookies;

///
/// returns a Ctx from the token in cookies
///
pub async fn mw_ctx_resolver<B>(
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response, MyAppError> {
    println!("->> {:<12} - mw_ctx_resolver", "MIDDLEWARE");
    let auth_token = cookies.get(AUTH_TOKEN).map(|c| c.value().to_string());
    let result_ctx = match auth_token
        .ok_or(MyAppError::new(StatusCode::BAD_REQUEST, "No auth_token in Request"))
        .and_then(parse_token)
    {
        Ok((user_id, role)) => {
            let ctx = Ctx::new(user_id, role);
            println!("->> {:<12} - ctx : {ctx:?} w_ctx_resolver", "MIDDLEWARE");
            Ok(ctx)
        } // TODO token validation
        Err(e) => Err(MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error in Token validations")),
    };
    // Remove the cookie if something went wrong other than NoAuthTokenError
    //if result_ctx.is_err() && !matches!(result_ctx, Err(AppError::NoAuthTokenError)) {
    if result_ctx
        .is_err() && !matches!(
        result_ctx,
        Err(MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error in Token validations")) {
        cookies.remove(tower_cookies::Cookie::named(AUTH_TOKEN))
    }
    // store the result_ctx in the request extension
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}
///
/// Middleware used o require authorization
/// takes a Ctx (could be a Claim, much better !)    
/// It requires the use the middleware mw_ctx_resolver to make this work.
///
pub async fn mw_require_auth<B>(
    ctx: Result<Ctx, AppError>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, AppError> {
    println!("->> {:<12} - mw_require_auth - {ctx:?} ", "MIDDLEWARE");
    ctx?;
    Ok(next.run(req).await)
}
/*
/// Parse a token of format 'user-[user_id].[expiration].[signature]
/// Returns (user_id, expiration, signature)
fn parse_token(token: String) -> Result<(usize, String, String), AppError> {
    let (_whole, user_id, exp, sign) = regex_captures!(r#"^user-(\d+)\.(.+)\.(.+)"#, &token)
        .ok_or(AppError::AuthTokenParseError)?;
    let user_id = user_id.parse().map_err(|_| AppError::AuthTokenParseError)?;

    Ok((user_id, exp.to_string(), sign.to_string()))
} */
/// Parse a token of format 'user-[user_id].[role]
/// Returns (user_id, role)
fn parse_token(token: String) -> Result<(usize, String), MyAppError> {
    let (_whole, user_id, role) =
        regex_captures!(r#"^user-(\d+)\.(.+)"#, &token).ok_or(|| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error Parsing token")?;
    let user_id = user_id.parse().map_err(|_| MyAppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Error Parsing token")?;
    println!("->> {:<12} -  {user_id} + {role}", "parse token");

    Ok((user_id, role.to_string()))
}
#[allow(dead_code)]
fn parse_jwt_to_ctx(jwt_token: String) -> Result<(usize, String), MyAppError> { unimplemented!()}

// Region ---> Ctx Extractor
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    type Rejection = MyAppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");
        let result = parts
            .extensions
            .get::<Result<Ctx, MyAppError>>()
            .ok_or(MyAppError::new(
                StatusCode::BAD_REQUEST,
                "Erreur : Claim not in Request Cookies",
            ))?
            .clone();
        *result
    }
}
// End Region ----> Ctx Extractor
