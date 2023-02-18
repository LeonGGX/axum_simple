//! src/print_req_res.rs

use crate::askama::askama_tpl::{DebugTemplate, HtmlTemplate};
use crate::errors::AppError;
use crate::sessions::useful_sessions::MyWritableSession;
use axum::headers::authorization::Basic;
use axum::headers::Authorization;
use axum::{
    body::{Body, Bytes},
    headers::Cookie,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    TypedHeader,
};

use std::option::Option;

#[allow(dead_code)]
pub async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

#[allow(dead_code)]
async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match hyper::body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} body: {}", direction, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} body = {:?}", direction, body);
    }

    Ok(bytes)
}
#[allow(dead_code)]
pub async fn print_req_cookies_askama(
    session: MyWritableSession,
    auth: Option<TypedHeader<Authorization<Basic>>>,
    cookie: TypedHeader<Cookie>,
) -> Result<HtmlTemplate<DebugTemplate>, AppError> {
    let str_cookie_one = format!("{cookie:?}");

    let s = ";";
    let str_cookie_two = str_cookie_one.as_str();
    let str = str_cookie_two.strip_prefix("TypedHeader(Cookie(").unwrap();
    let v: Vec<&str> = str.split(s).collect();

    let cookies: Vec<String> = v.iter().map(|&x| x.into()).collect();

    let str_auth: String;
    if let Some(auth) = auth {
        str_auth = format!("{auth:?}");
    } else {
        str_auth = "Pas de Header Authorisation".to_string();
    }
    tracing::info!("{str_auth}");

    let session_user = session.get_raw("users").unwrap_or_default();
    let session_role = session.get_raw("role").unwrap_or_default();
    //let str_auth = String::from("Authorisation bearer pas fait");
    let title = "Debug ...".to_string();
    let template = DebugTemplate {
        title,
        cookies,
        str_auth,
        session_user,
        session_role,
    };
    Ok(HtmlTemplate(template))
}
/*
pub async fn print_req_headers(
    request: Box<Request<dyn Header>>,
) -> String {
    let(parts, header) = request.into_parts();

}
*/

/*
///
/// Utility function to parse usernames
/// Returns a String with the parsed username or AppError
///
pub fn parse(s: &String) -> Result<String, AppError> {
    // `.trim()` returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character.
    let is_empty_or_whitespace = s.trim().is_empty();


    // Iterate over all characters in the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}', '#', '*', ' '];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
        return Err(AppError::ValidationError);
    } else {
        return Ok(s.to_string());
    }
}
*/
