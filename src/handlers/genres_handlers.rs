//! src/handlers/genres_hdl.rs

use axum::debug_handler;
use axum::extract::{Form, Path, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum_flash::{Flash, IncomingFlashes};

use crate::askama::askama_tpl::{HandleGenresTemplate, ListGenresTemplate};
use crate::{/*db,*/ globals, AppState};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::db::genres::*;
use crate::errors::MyAppError;
//use crate::handlers::musicians_handlers::get_filtered_list_persons_once_cell;

use crate::models::genre::Genre;
//use crate::models::musician::Person;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Payload {
    pub name: String,
}

//***********************************************************************************
// CRUD Operations
//

/// # Handler
/// Creates a new genre in the DB
///
/// ## Arguments
/// * 'flash' - An axum_flash Flash
/// * 'state' - the AppState with PgPool
/// * 'form'  - the person name comes from a Form<Demande> where struct Payload has a field "value: String"\
///             Form must be placed as last argument because it consumes the request
/// ## Returns
/// * the flash message
/// * Redirects to /genres page with a flash message\
/// '''
#[debug_handler]
pub async fn create_genre_hdl(
    flash: Flash,
    State(state): State<AppState>,
    Form(form): Form<Payload>,
) -> (Flash, Redirect) {
    let new_genre = form.name;

    if let Ok(genre) = add_genre(&state.pool, new_genre.clone()).await {
        tracing::info!("genre added : {:?}", genre);
        let message = format!("Genre ajouté : {}", new_genre);
        (flash.success(message), Redirect::to("/genres"))
    } else {
        tracing::info!("Error adding genre");
        let message = "Genre pas ajouté".to_string();
        (flash.error(message), Redirect::to("/api/genres"))
    }
}

#[debug_handler]
pub async fn update_genre_hdl(
    State(state): State<AppState>,
    flash: Flash,
    Path(id): Path<i32>,
    Form(form): Form<Payload>,
) -> (Flash, Redirect) {
    let updated_genre_name = form.name;
    if let Ok(genre) = update_genre(id, updated_genre_name, &state.pool).await {
        tracing::info!("genre modifyed : {:?}", genre);
        let message = format!("Genre modifié : {}", genre.name);
        (flash.success(message), Redirect::to("/api/genres"))
    } else {
        tracing::info!("error modifying genre");
        let message = "Genre pas modifié".to_string();
        (flash.error(message), Redirect::to("/api/genres"))
    }
}

#[debug_handler]
pub async fn delete_genre_hdl(
    flash: Flash,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (Flash, Redirect) {
    if let Ok(deleted_name) = delete_genre(id, &state.pool).await {
        let message = format!("Genre effacé : {}", deleted_name);
        (flash.success(message), Redirect::to("/api/genres"))
    } else {
        let message = "Genre pas effacé".to_string();
        (flash.error(message), Redirect::to("/api/genres"))
    }
}

//*******************************************************************************
// Functions to show or print list of genres
//

/// # Handler
///
/// Shows the page to manage the genres
///
/// Returns a HTML Page (askama template) or AppError
///
#[debug_handler]
pub async fn manage_genres_askama_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
) -> Result<(IncomingFlashes, HandleGenresTemplate), MyAppError> {
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    // populates the static vector of Genre with the list of all genres
    //set_static_vec_genres(list_genres(&state.pool).await?);
    // get the values in the static vector of genres
    //let genres = get_static_vec_genres();

    let genres = get_list_all_genres_one_cell(&state.pool)
        .await
        .map_err(|_| {
            MyAppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting global list genres",
            )
        })?;

    let title = "Gestion des Genres".to_string();
    let flash = Some(flash);

    let template = HandleGenresTemplate {
        title,
        flash,
        genres,
    };

    Ok((in_flash, template))
}

/// # Handler
///
/// Shows a printable list of Genres
///
/// Returns a HTML Page or AppError
///
#[debug_handler]
pub async fn list_genres_askama_hdl(//State(state): State<AppState>,
) -> Result<ListGenresTemplate, MyAppError> {
    //let list_genres = list_genres(&state.pool).await?;
    //let list_genres = get_static_vec_genres();
    let list_genres = get_existing_list_genres_one_cell().await.unwrap();
    let template = ListGenresTemplate { list_genres };
    Ok(template)
}

//****************************************************************************************
// Functions to find genres by different criteria
//

/// # Handler
///
/// find_genre_by_name
///
/// returns list genre page with genre found
///
#[debug_handler]
pub async fn find_genre_by_name_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
    Form(form): Form<Payload>,
) -> Result<HandleGenresTemplate, MyAppError> {
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");

    let name = form.name;
    tracing::debug!("name : {}", name);

    //let genres = find_genre_by_name(name, &state.pool).await?;
    //set_static_vec_genres(find_genre_by_name(name, pool).await?);
    //let genres = get_static_vec_genres();
    let genres = get_filtered_list_genres_once_cell(&state.pool, name)
        .await
        .unwrap();

    let title = "Genre(s) trouvé(s)".to_string();
    let flash = Some(flash);
    //let flash = None;

    let template = HandleGenresTemplate {
        title,
        flash,
        genres,
    };
    Ok(template)
}

///
/// Functions with OneCell crate
///
///
pub async fn get_filtered_list_genres_once_cell(
    pool: &PgPool,
    genre_name: String,
) -> Result<Vec<Genre>, MyAppError> {
    globals::once_cell::set_static_vec_genres(find_genre_by_name(genre_name, pool).await.unwrap());
    let genres = globals::once_cell::get_static_vec_genres();
    Ok(genres)
}

pub async fn get_list_all_genres_one_cell(pool: &PgPool) -> Result<Vec<Genre>, MyAppError> {
    globals::once_cell::set_static_vec_genres(list_genres(pool).await.unwrap());
    let genres = globals::once_cell::get_static_vec_genres();
    Ok(genres)
}

pub async fn get_existing_list_genres_one_cell() -> Result<Vec<Genre>, MyAppError> {
    let genres = globals::once_cell::get_static_vec_genres();
    Ok(genres)
}
