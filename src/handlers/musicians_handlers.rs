//! src/handlers/musicians_handlers

use crate::askama::askama_tpl::{HandlePersonsTemplate, HtmlTemplate, ListPersonsTemplate};
use crate::db::musicians::{
    add_person, delete_person, find_persons_by_name_parts, find_persons_by_name_strict,
    update_person,
};
use crate::errors::AppError;
use crate::{db, AppState};
use axum::debug_handler;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::Form;
use axum_flash::{Flash, IncomingFlashes};
use serde::{Deserialize, Serialize};

/// the struct must be public ...
#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub name: String,
}

#[debug_handler]
pub async fn list_persons_askama_hdl(
    State(state): State<AppState>,
) -> Result<HtmlTemplate<ListPersonsTemplate>, AppError> {
    let list_persons = db::musicians::list_persons(&state.pool).await?;
    let template = ListPersonsTemplate { list_persons };
    Ok(HtmlTemplate(template))
}

#[debug_handler]
pub async fn manage_persons_askama_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
) -> Result<(IncomingFlashes, HtmlTemplate<HandlePersonsTemplate>), AppError> {
    let title = "Gestion des Musiciens".to_string();
    let flash = in_flash
        .clone()
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    let persons = db::musicians::list_persons(&state.pool).await?;

    let flash = Some(flash);
    let template = HandlePersonsTemplate {
        title,
        flash,
        persons,
    };
    // il faut retourner le flash pour qu'il soit enlevé du cookie
    Ok((in_flash, HtmlTemplate(template)))
}

/// # Handler
/// Creates a new person (musician) in the DB
///
/// ## Arguments
/// * 'flash' - An axum_flash Flash
/// * 'state' - the AppState with PgPool
/// * 'form'  - the person name comes from a Form<Payload> where struct Payload has a field "value: String"\
///             Form must be placed as last argument because it consumes the request
/// ## Returns
/// * the flash message
/// * Redirects to /persons page with a flash message\
/// '''
#[debug_handler]
pub async fn create_person_hdl(
    flash: Flash,
    State(state): State<AppState>,
    Form(form): Form<Payload>,
) -> (Flash, Redirect) {
    let new_person = form.name;
    tracing::info!("form : {}", new_person.clone());

    if let Ok(person) = add_person(&state.pool, new_person.clone()).await {
        tracing::info!("person added : {:?}", person);
        let message = format!("Musicien ajouté : {}", new_person);
        (flash.success(message), Redirect::to("/persons"))
    } else {
        tracing::info!("error adding person");
        let message = "Musicien pas ajouté".to_string();
        (flash.error(message), Redirect::to("/persons"))
    }
}

#[debug_handler]
pub async fn update_person_hdl(
    State(state): State<AppState>,
    flash: Flash,
    Path(id): Path<i32>,
    Form(form): Form<Payload>,
) -> (Flash, Redirect) {
    let updated_person_name = form.name;
    if let Ok(person) = update_person(id, updated_person_name, &state.pool).await {
        tracing::info!("person modifyed : {:?}", person);
        let message = format!("Musicien modifié : {}", person.full_name);
        (flash.success(message), Redirect::to("/persons"))
    } else {
        tracing::info!("error modifying person");
        let message = "Musicien pas modifié".to_string();
        (flash.error(message), Redirect::to("/persons"))
    }
}

#[debug_handler]
pub async fn delete_person_hdl(
    flash: Flash,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (Flash, Redirect) {
    if let Ok(deleted_name) = delete_person(id, &state.pool).await {
        let message = format!("Musicien effacé : {}", deleted_name);
        (flash.success(message), Redirect::to("/persons"))
    } else {
        let message = "Erreur Musicien pas effacé".to_string();
        (flash.error(message), Redirect::to("/persons"))
    }
}

//****************************************************************************************
// Functions to find musicians by different criteria
//

/// # Handler
///
/// find_person_by_name_hdl
///
/// Finds person(s) with the first letter or a group of first letters\
///  e.g. two authors in the DB BREL, BRASSENS and BARTOK\
/// if you enter "B" the function returns all three\
/// if you enter "BR" the function returns BREL and BRASSENS\
/// if you enter "BRE" the function returns BREL (idem if you enter BREL)\
///
/// returns manage persons page with persons found
///
pub async fn find_person_by_name_hdl(
    State(state): State<AppState>,
    //in_flash: IncomingFlashes,
    Form(form): Form<Payload>,
) -> Result<HtmlTemplate<HandlePersonsTemplate>, AppError> {
    let person_name_to_find = form.name;
    let persons = find_persons_by_name_parts(person_name_to_find, &state.pool).await?;
    //set_static_vec_persons(find_genre_by_name(name, pool).await?);
    //let persons = get_static_vec_persons();
    let title = "Musicien(s) trouvé(s)".to_string();
    let flash = None;
    let template = HandlePersonsTemplate {
        title,
        flash,
        persons,
    };
    Ok(HtmlTemplate(template))
}
