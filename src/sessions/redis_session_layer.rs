//! src/sessions/redis_sessions.rs

use async_redis_session::RedisSessionStore;
use axum::http::StatusCode;
use axum_sessions::SessionLayer;
use rand::Rng;

pub async fn create_redis_session_layer(
) -> Result<SessionLayer<RedisSessionStore>, (StatusCode, String)> {
    let store = RedisSessionStore::new("redis://127.0.0.1/");
    match store {
        Ok(store) => {
            let secret = rand::thread_rng().gen::<[u8; 128]>();
            let session_layer = SessionLayer::new(store, &secret);
            Ok(session_layer)
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("une erreur s'est produite: {}", err),
            ));
        }
    }
}
