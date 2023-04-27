//! src/print_req_res.rs

use crate::askama::askama_tpl::{DebugTemplate, DebugTemplateTwo};
use crate::errors::AppError;

use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::{
    body::{Body, Bytes},
    headers::Cookie,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    TypedHeader,
};

//use crate::models::user::User;
//use axum_sessions::extractors::WritableSession;
use axum::http::header;
use axum_extra::extract::CookieJar;
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
    //session: WritableSession,
    auth: Option<TypedHeader<Authorization<Bearer>>>,
    cookie: TypedHeader<Cookie>,
) -> Result<DebugTemplate, AppError> {
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
        str_auth = "Pas de Header Authorisation Bearer".to_string();
    }
    tracing::info!("{str_auth}");
    /*
       if let Some(user) = session.get::<User>("user") {
           let title = "Debug ...".to_string();
           let session_user = user.name;
           let session_role = user.role;
           let template = DebugTemplate {
               title,
               cookies,
               str_auth,
               session_user,
               session_role,
           };
           Ok(template)
       } else {
           let title = "Debug ...".to_string();
           let session_user = "no session user".to_string();
           let session_role = "".to_string();
           let template = DebugTemplate {
               title,
               cookies,
               str_auth,
               session_user,
               session_role,
           };
    */
    let title = "Debug ...".to_string();
    let template = DebugTemplate {
        title,
        cookies,
        str_auth,
    };
    Ok(template)
}

#[allow(dead_code)]
pub async fn print_cookies_askama<B>(cookie_jar: CookieJar, req: Request<B>) -> DebugTemplateTwo {
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
    let refresh_token = cookie_jar
        .get("refresh_token")
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
    let logged_in = cookie_jar
        .get("logged_in")
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

    let title = "Liste des Cookies".to_string();
    let template = DebugTemplateTwo {
        title,
        auth_token,
        refresh_token,
        logged_in,
    };
    template
}
