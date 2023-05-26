//! /src/routers.rs

use crate::authentication::auth_layer;
use crate::handlers::genres_handlers::{
    create_genre_hdl, delete_genre_hdl, find_genre_by_name_hdl, list_genres_askama_hdl,
    manage_genres_askama_hdl, update_genre_hdl,
};
use crate::handlers::get_me_hld::get_me_hdl;
use crate::handlers::login_handlers::{login_form_askama_hdl, post_login_hdl};
use crate::handlers::logout_handlers::{logout_handler, logout_page};
use crate::handlers::musicians_handlers::{
    create_person_hdl, delete_person_hdl, find_person_by_name_hdl, list_persons_askama_hdl,
    manage_persons_askama_hdl, update_person_hdl,
};
use crate::handlers::partitions_handlers::{
    create_partition_hdl, delete_partition_hdl, find_partition_author_hdl,
    find_partition_genre_hdl, find_partition_title_hdl, manage_partitions_hdl,
    print_list_partitions_hdl, update_partition_hdl,
};
use crate::handlers::signup_handlers::{post_signup_hdl, signup_form_askama_hdl};
use crate::handlers::utils_handlers::{
    about_hdl, favicon, handler_404, hello_name_askama_hdl, list_users_askama_hdl,
    list_users_with_extension, start_hdl, welcome_hdl,
};
use crate::main_response_mapper;
use crate::print_req_res::print_cookies_askama;
use crate::AppState;
use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::{get, post};
use axum::{middleware, Router};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing::info_span;

///
/// # Creates all the site routes
///
/// Returns a Router (with no other arguement)    
/// needs the AppState for some routes    
/// The main routes are 'start routes', 'api routes', 'auth routes', 'debug routes'    
///
pub fn create_routers(app_state: AppState) -> Router {
    // the start page :
    let start_route = Router::new().route("/", get(start_hdl));

    // A simple router to the about page
    // Attention the path must be "/" !!!
    // because the route will be nested in api routes
    let about_route = Router::new().route("/", get(about_hdl));

    // A simple router to say hello
    let hello_routes = Router::new().route("/:name", get(hello_name_askama_hdl));

    // the administrator routes are protected by the auth layer that adds
    // a JWTAuthmiddleware struct to the request. This contains a User (authenticated with
    // access token).
    // two possibilities :
    // the easiest one : add an Extension<JWTAuthmiddleware> as argument to the handler
    // (this is the case with 'list_user_with_extension' handler
    // the less easy one : add a second layer auth_admin that comes after the auth layer
    // and adds a new condition to the request ...
    // (this is the case with 'list_user_askama')
    //
    let admin_routes = Router::new()
        .route("/users", get(list_users_with_extension))
        /*
        .route("/users", get(list_users_askama_hdl))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_layer::auth_admin,
        ))
        */
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_layer::auth,
        ))
        .with_state(app_state.clone());

    // A router to debug app
    // for all users
    let debug_routes = Router::new()
        .route(
            "/cookies",
            get(print_cookies_askama /*print_req_cookies_askama*/),
        )
        .with_state(app_state.clone());

    // Authorisation Router
    // the route "/login" correspond to "/auth/login"
    // the "/logout" route is submitted to a logged state, so not here ...
    // for all users of course.
    let auth_routes = Router::new()
        .route("/login", get(login_form_askama_hdl).post(post_login_hdl))
        .route("/signup", get(signup_form_askama_hdl).post(post_signup_hdl));

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

    let welcome_route = Router::new().route("/", get(welcome_hdl));

    // api routes only for logged users
    // whatever their role.
    // the logout route is here (one must be logged in to logout)
    let api_routes = Router::new()
        .nest("/about", about_route)
        .nest("/welcome", welcome_route)
        .nest("/persons", persons_routes)
        .nest("/genres", genres_routes)
        .nest("/partitions", partitions_routes)
        .route("/logout", get(logout_page).post(logout_handler))
        .route("/me", get(get_me_hdl))
        .route_layer(middleware::from_fn_with_state(
            app_state.clone(),
            auth_layer::auth,
        ))
        .with_state(app_state.clone());

    // Uniting all the routers ...
    // layer must be applied last (after fallback if all routes must have the layer)
    Router::new()
        .nest("/", start_route)
        .nest("/auth", auth_routes)
        .nest("/api", api_routes)
        .nest("/debug", debug_routes)
        .nest("/hello", hello_routes)
        .nest("/admin", admin_routes)
        .route("/favicon.png", get(favicon))
        .fallback(handler_404)
        .layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                        let matched_path = request
                            .extensions()
                            .get::<MatchedPath>()
                            .map(MatchedPath::as_str);

                        info_span!(
                            "http_request",
                            method = ?request.method(), // ? means use of Debug
                            matched_path,
                            some_other_field = tracing::field::Empty,
                        )
                    }),
                )
                .layer(axum::middleware::map_response(main_response_mapper)),
        )
        .with_state(app_state)
}
