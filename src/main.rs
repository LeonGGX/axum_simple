//! /src/main.rs

use axum::{
    extract::FromRef,
    routing::{get, post},
    Router,
};
use axum_flash::Key;

use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod askama;
pub mod authentication;
mod db;
mod errors;
mod globals;
mod handlers;
mod log;
mod models;
mod print_req_res;
mod sessions;

use crate::db::connect::create_pg_pool;
use crate::db::musicians::find_persons_by_name_parts;
//use crate::db::genres::find_genre_by_name;
use crate::handlers::axum_sessions_handlers::{get_session_hdl, set_session_hdl};
use crate::handlers::genres_handlers::{
    create_genre_hdl, delete_genre_hdl, find_genre_by_name_hdl, list_genres_askama_hdl,
    manage_genres_askama_hdl, update_genre_hdl,
};
use crate::handlers::login_handlers::{login_form_askama_hdl, post_login_hdl};
use crate::handlers::musicians_handlers::{create_person_hdl, delete_person_hdl};
use crate::handlers::partitions_handlers::{
    create_partition_hdl, delete_partition_hdl, find_partition_author_hdl,
    find_partition_genre_hdl, find_partition_title_hdl, manage_partitions_hdl,
    print_list_partitions_hdl, update_partition_hdl,
};
use crate::handlers::{musicians_handlers::*, utils_handlers::*};
//use crate::log::LogLayer;
use crate::models::musician::Person;
use crate::print_req_res::print_req_cookies_askama;

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
                .without_time(),
        )
        .init();

    //*****************************************************
    // les données pour ouvrir la base de données Postgresql

    // on va chercher les éléments pour ouvrir la DB dans les fichier .env
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    // on crée le pool Postgresql de sqlx
    let pool = create_pg_pool(&db_url).await?;

    //*****************************************************
    // a layer with a redis session
    // uses async_redis_session and axum_session
    let session_layer = sessions::redis_session::create_redis_session_layer()
        .await
        .unwrap();

    //******************************************************
    // axum-flash
    // vient de tower_cookies::Key
    let key = Key::generate();
    let flash_config = axum_flash::Config::new(key).use_secure_cookies(false);

    //*******************************************************
    // Our application state
    // It contains the DB pool and a configuration for flash cookies needed by axum_flash
    let state = AppState { pool, flash_config };

    //******************************************************
    // the Routers
    //

    // A simple router to the about page
    // Attention the path must be "/" !!!
    let about_route = Router::new().route("/", get(about_hdl));

    // A simple router to say hello
    let hello_routes = Router::new().route("/:name", get(hello_name_askama_hdl));

    // A router to debug app
    let debug_routes = Router::new()
        .route("/cookies", get(print_req_cookies_askama))
        .route("/set_session", get(set_session_hdl))
        .route("/get_session", get(get_session_hdl));

    // Authorisation Router
    // the route "/login" correspond to "/auth/login"
    let auth_routes =
        Router::new().route("/login", get(login_form_askama_hdl).post(post_login_hdl));

    // A router with state : a PgPool is necessary and is to be found in the state
    // with_state(AppState{}) comes at the end or if it's general later in app
    // Handles all the routes to manage persons
    // the route "/" correspond to "/persons", the route "/add" correspond to "/persons/add"
    let persons_routes = Router::new()
        .route("/", get(manage_persons_askama_hdl))
        .route("/add", post(create_person_hdl))
        .route("/delete/:id", post(delete_person_hdl))
        .route("/:id", post(update_person_hdl))
        .route("/print", get(list_persons_askama_hdl))
        .route("/find", post(find_person_by_name_hdl));

    let genres_routes = Router::new()
        .route("/", get(manage_genres_askama_hdl))
        .route("/add", post(create_genre_hdl))
        .route("/delete/:id", post(delete_genre_hdl))
        .route("/:id", post(update_genre_hdl))
        .route("/print", get(list_genres_askama_hdl))
        .route("/find", post(find_genre_by_name_hdl));

    let partitions_routes = Router::new()
        .route("/", get(manage_partitions_hdl))
        .route("/add", post(create_partition_hdl))
        .route("/delete/:id", post(delete_partition_hdl))
        .route("/:id", post(update_partition_hdl))
        .route("/print", get(print_list_partitions_hdl))
        .route("/find/title", post(find_partition_title_hdl))
        .route("/find/author", post(find_partition_author_hdl))
        .route("/find/genre", post(find_partition_genre_hdl));

    // Uniting all the routers ...
    // layer must be applied last (after fallback if all routes must have the layer)
    let app = Router::new()
        .route("/", get(start_hdl))
        //.route("/about", get(about))
        .nest("/about", about_route)
        .nest("/persons", persons_routes)
        .nest("/genres", genres_routes)
        .nest("/partitions", partitions_routes)
        .nest("/hello", hello_routes)
        .nest("/debug", debug_routes)
        .nest("/auth", auth_routes)
        .with_state(state)
        .fallback(handler_404)
        .layer(
            ServiceBuilder::new()
                //.layer(middleware::from_fn(print_request_response))
                .layer(TraceLayer::new_for_http())
                //.layer(middleware::from_fn(print_full_request))
                //.layer(LogLayer)
                .layer(session_layer),
        );

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// # Application State
/// passed to the routers where needed with _router.with_state(state)_
/// and can be used by the handlers   
///
/// ## Fields
/// - PgPool to connect to the DB
/// - flash_config needed by axum_flash
///
#[derive(Clone, FromRef)]
pub struct AppState {
    pub pool: PgPool,
    pub flash_config: axum_flash::Config,
}
