//! src/handlers/genres_hdl.rs

use axum::debug_handler;
use axum::extract::{Form, Path, State};
use axum::response::Redirect;
use axum_flash::{Flash, IncomingFlashes};

use crate::askama::askama_tpl::{HandleGenresTemplate, HtmlTemplate, ListGenresTemplate};
use crate::AppState;
use serde::{Deserialize, Serialize};

use crate::db::genres::*;
use crate::errors::AppError;
//use crate::models::genre::Genre;

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
        (flash.error(message), Redirect::to("/genres"))
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
        (flash.success(message), Redirect::to("/genres"))
    } else {
        tracing::info!("error modifying genre");
        let message = "Genre pas modifié".to_string();
        (flash.error(message), Redirect::to("/genres"))
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
        (flash.success(message), Redirect::to("/genres"))
    } else {
        let message = "Genre pas effacé".to_string();
        (flash.error(message), Redirect::to("/genres"))
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
pub async fn manage_genres_askama_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
) -> Result<(IncomingFlashes, HtmlTemplate<HandleGenresTemplate>), AppError> {
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    let genres = list_genres(&state.pool).await?;
    //set_static_vec_genres(list_genres(pool).await?);
    //let genres = get_static_vec_genres();
    let title = "Gestion des Genres".to_string();
    let flash = Some(flash);

    let template = HandleGenresTemplate {
        title,
        flash,
        genres,
    };

    Ok((in_flash, HtmlTemplate(template)))
}

/// # Handler
///
/// Shows a printable list of Genres
///
/// Returns a HTML Page or AppError
///
pub async fn list_genres_askama_hdl(
    State(state): State<AppState>,
) -> Result<HtmlTemplate<ListGenresTemplate>, AppError> {
    let list_genres = list_genres(&state.pool).await?;
    //let list_genres = get_static_vec_genres();
    let template = ListGenresTemplate { list_genres };
    Ok(HtmlTemplate(template))
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
pub async fn find_genre_by_name_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
    Form(form): Form<Payload>,
) -> Result<HtmlTemplate<HandleGenresTemplate>, AppError> {
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");

    let name = form.name;
    tracing::debug!("name : {}", name);

    let genres = find_genre_by_name(name, &state.pool).await?;
    //set_static_vec_genres(find_genre_by_name(name, pool).await?);
    //let genres = get_static_vec_genres();
    let title = "Genre(s) trouvé(s)".to_string();
    let flash = Some(flash);
    //let flash = None;

    let template = HandleGenresTemplate {
        title,
        flash,
        genres,
    };
    Ok(HtmlTemplate(template))
}
