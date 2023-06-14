//! src/handlers/musicians_handlers

use crate::askama::askama_tpl::{HandlePersonsTemplate, /*HtmlTemplate, */ ListPersonsTemplate,};
use crate::db::musicians::{
    add_person, delete_person, find_persons_by_name_parts, /*find_persons_by_name_strict,*/
    list_persons, update_person,
};
use crate::errors::MyAppError;
use crate::globals;
use crate::models::musician::Person;
use crate::AppState;
use axum::debug_handler;
use axum::extract::{Path, State};
use axum::response::Redirect;
use axum::Form;
use axum_flash::{Flash, IncomingFlashes};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// the struct must be public ...
#[derive(Serialize, Deserialize)]
pub struct Payload {
    pub name: String,
}
///
/// # Handler
///
/// **Manages the Printable Musician List Page**  
/// Initiates the static Vec(Person) to show the list of musicians
/// filtered or not
///
/// ## Arguments
/// * 'state' - the AppState with PgPool
///
/// ## Returns
/// * Result with the Askama Template that handles
/// the Musician Page or AppError
///
/// '''
#[debug_handler]
pub async fn list_persons_askama_hdl(//State(state): State<AppState>,
) -> Result<ListPersonsTemplate, MyAppError> {
    let list_persons = get_existing_list_persons_one_cell().await.unwrap();
    let template = ListPersonsTemplate { list_persons };
    Ok(template)
}

///
/// # Handler
///
/// **Manages the Musician Page**  
/// Initiates the static Vec<Person> to show the list of musicians
///
/// ## Arguments
/// * 'state' - the AppState with PgPool
/// * 'in_flash' - An axum_flash IncomingFlash
/// ## Returns
/// * Result with the IncomingFlashes and the Askama Template that handles
/// the Musician Page
/// * Redirects to '/persons' page with a flash message\
/// '''
#[debug_handler]
pub async fn manage_persons_askama_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
) -> Result<(IncomingFlashes, HandlePersonsTemplate), MyAppError> {
    let title = "Gestion des Musiciens".to_string();
    let flash = in_flash
        .clone()
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    let persons = get_list_all_persons_one_cell(&state.pool).await.unwrap();

    let flash = Some(flash);
    let template = HandlePersonsTemplate {
        title,
        flash,
        persons,
    };
    // il faut retourner le flash pour qu'il soit enlevé du cookie
    Ok((in_flash, template))
}
///
/// # Handler
///
/// **Creates a new person (musician) in the DB**
///
/// ## Arguments
/// * 'flash' - An axum_flash Flash
/// * 'state' - the AppState with PgPool
/// * 'form'  - the person name comes from a Form(Payload) where struct Payload has a field "value: String"<br>
/// <br>
///             *Form must be placed as last argument because it consumes the request*
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
        (flash.success(message), Redirect::to("/api/persons"))
    } else {
        tracing::info!("error adding person");
        let message = "Musicien pas ajouté".to_string();
        (flash.error(message), Redirect::to("/api/persons"))
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
        tracing::info!("person modified : {:?}", person);
        let message = format!("Musicien modifié : {}", person.full_name);
        (flash.success(message), Redirect::to("/api/persons"))
    } else {
        tracing::info!("error modifying person");
        let message = "Musicien pas modifié".to_string();
        (flash.error(message), Redirect::to("/api/persons"))
    }
}

#[debug_handler]
pub async fn delete_person_hdl(
    flash: Flash,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> (Flash, Redirect) {
    if let Ok(deleted_person) = delete_person(id, &state.pool).await {
        let message = format!("Musicien effacé : {}", deleted_person.full_name);
        (flash.success(message), Redirect::to("/api/persons"))
    } else {
        let message = "Erreur Musicien pas effacé".to_string();
        (flash.error(message), Redirect::to("/api/persons"))
    }
}

//****************************************************************************************
// Functions to find musicians by different criteria
//

#[doc = include_str!("../docs/find_person_by_name_hdl.md")]
///
/// # Handler
///
/// ## pub async fn find_person_by_name_hdl
///
/// **Finds person(s) with the first letter or a group of first letters**<br>
///  e.g. two authors in the DB BREL, BRASSENS and BARTOK<br>
/// if you enter "B" the function returns all three<br>
/// if you enter "BR" the function returns BREL and BRASSENS<br>
/// if you enter "BRE" the function returns BREL (idem if you enter BREL)<br>
/// <br>
/// returns manage persons page with persons found
///
#[debug_handler]
pub async fn find_person_by_name_hdl(
    State(state): State<AppState>,
    Form(form): Form<Payload>,
) -> Result<HandlePersonsTemplate, MyAppError> {
    let person_name_to_find = form.name;
    let persons = get_filtered_list_persons_once_cell(&state.pool, person_name_to_find).await?;
    //.unwrap();
    tracing::info!("liste personnes trouvées :{:?}", persons);

    let title = "Musicien(s) trouvé(s)".to_string();
    let flash = None;
    let template = HandlePersonsTemplate {
        title,
        flash,
        persons,
    };
    Ok(template)
}

///
/// Functions with OneCell crate
///
///
pub async fn get_filtered_list_persons_once_cell(
    pool: &PgPool,
    person_name: String,
) -> Result<Vec<Person>, MyAppError> {
    globals::once_cell::set_static_vec_persons(
        find_persons_by_name_parts(person_name, pool).await?,
    );
    let persons = globals::once_cell::get_static_vec_persons();
    Ok(persons)
}

pub async fn get_list_all_persons_one_cell(pool: &PgPool) -> Result<Vec<Person>, MyAppError> {
    globals::once_cell::set_static_vec_persons(list_persons(pool).await.unwrap());
    let persons = globals::once_cell::get_static_vec_persons();
    Ok(persons)
}

pub async fn get_existing_list_persons_one_cell() -> Result<Vec<Person>, MyAppError> {
    let persons = globals::once_cell::get_static_vec_persons();
    Ok(persons)
}
