//! /src/main.rs
//!
//! Note on State : use Arc<Mutex<Appstate>>> when you have to mutate the Appstate
//! It's not necessary to mutate a DB connection pool
//!

pub use ::time;
use axum::extract::FromRef;
use axum_flash::Key;

use crate::config::Config;
use crate::db::connect::{create_pg_pool, create_redis_client};
use crate::routers::create_routers;
use axum_core::response::Response;
use redis::Client;
use sqlx::PgPool;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod askama;
pub mod authentication;
mod config;
mod ctx;
mod db;
mod errors;
mod globals;
mod handlers;
mod log;
mod models;
mod print_req_res;
mod routers;
//mod sessions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //******************************************************
    // initiating the tracing system
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "axum_simple=debug,tower_http=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time()
                .pretty(),
        )
        .init();
    //*****************************************************************
    // Getting configuration data
    // using the Config struct from config.rs
    dotenvy::dotenv().ok();
    let env = Config::init();

    //*****************************************************
    // Opening DB connections (Postgresql and Redis)

    // creating Postgresql pool from sqlx
    let pool = create_pg_pool(env.clone().database_url).await?;

    // creating redis client
    let redis_client = create_redis_client(env.clone().redis_url).await?;

    //*****************************************************
    // a layer with a redis session
    // uses async_redis_session and axum_session
    //let session_layer = sessions::redis_session::create_redis_session_layer()
    //.await
    //.unwrap();

    //******************************************************
    // axum-flash
    // vient de tower_cookies::Key
    let key = Key::generate();
    let flash_config = axum_flash::Config::new(key).use_secure_cookies(false);

    //*******************************************************
    // Our application state
    // It contains the DB pool,
    // a configuration for flash cookies needed by axum_flash (flash_config),
    // the configuration data (env)
    // a redis client used for the session management,
    let state = AppState {
        pool,
        flash_config,
        env,
        redis_client,
    };

    //*******************************************************
    // Creating the routers
    let app = create_routers(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("🚀 Server started successfully");
    println!("-- Listening on 127.0.0.1:3000");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn main_response_mapper<B>(res: Response<B>) -> Response<B> {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    println!();
    res
}

/// # Application State
/// passed to the routers where needed with _router.with_state(state)_
/// and can be used by the handlers   
///
/// ## Fields
/// - PgPool to connect to the DB
/// - flash_config needed by axum_flash
/// - env : data from the config file
/// - redis_client : the redis client for the session
///
#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: PgPool,
    pub flash_config: axum_flash::Config,
    pub env: Config,
    pub redis_client: Client,
}
