//! Extractors for sessions.

use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use axum::{async_trait, extract::FromRequestParts, Extension};

use axum::http::request::Parts;
//use axum::http::Request;
use axum_sessions::async_session;
use tokio::sync::{OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock};

/// A type alias which provides a handle to the underlying session.
///
/// This is provided via [`http::Extensions`](axum::http::Extensions). Most
/// applications will use the
/// [`ReadableSession`](crate::extractors::ReadableSession) and
/// [`WritableSession`](crate::extractors::WritableSession) extractors rather
/// than using the handle directly. A notable exception is when using this
/// library as a generic Tower middleware: such use cases will consume the
/// handle directly.
pub type SessionHandle = Arc<RwLock<async_session::Session>>;

/// An extractor which provides a readable session. Sessions may have many
/// readers.
#[derive(Debug)]
pub struct MyReadableSession {
    session: OwnedRwLockReadGuard<async_session::Session>,
}

impl Deref for MyReadableSession {
    type Target = OwnedRwLockReadGuard<async_session::Session>;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}
/*
#[async_trait]
impl<S, B> FromRequest<S, B> for MyReadableSession
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request(request: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session_handle): Extension<SessionHandle> =
            Extension::from_request(request, state)
                .await
                .expect("Session extension missing. Is the session layer installed?");
        let session = session_handle.read_owned().await;

        Ok(Self { session })
    }
}
*/
#[async_trait]
impl<S> FromRequestParts<S> for MyReadableSession
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session_handle): Extension<SessionHandle> =
            Extension::from_request_parts(parts, state)
                .await
                .expect("Session extension missing. Is the session layer installed?");
        let session = session_handle.read_owned().await;

        Ok(Self { session })
    }
}

/// An extractor which provides a writable session. Sessions may have only one
/// writer.
#[derive(Debug)]
pub struct MyWritableSession {
    session: OwnedRwLockWriteGuard<async_session::Session>,
}

impl Deref for MyWritableSession {
    type Target = OwnedRwLockWriteGuard<async_session::Session>;

    fn deref(&self) -> &Self::Target {
        &self.session
    }
}

impl DerefMut for MyWritableSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.session
    }
}
/*
#[async_trait]
impl<S, B> FromRequest<S, B> for MyWritableSession
where
    B: Send + 'static,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request(request: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session_handle): Extension<SessionHandle> =
            Extension::from_request(request, state)
                .await
                .expect("Session extension missing. Is the session layer installed?");
        let session = session_handle.write_owned().await;

        Ok(Self { session })
    }
}
*/
#[async_trait]
impl<S> FromRequestParts<S> for MyWritableSession
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(session_handle): Extension<SessionHandle> =
            Extension::from_request_parts(parts, state)
                .await
                .expect("Session extension missing. Is the session layer installed?");
        let session = session_handle.write_owned().await;

        Ok(Self { session })
    }
}
