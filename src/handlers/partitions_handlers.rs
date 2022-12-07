//! src/handlers/partitions_handlers.rs

use axum::debug_handler;
use axum::extract::{Form, Path, State};
use axum::response::Redirect;

use axum_flash::{Flash, IncomingFlashes};

use crate::askama::askama_tpl::{HandlePartitionsTemplate, HtmlTemplate, ListPartitionsTemplate};
use crate::AppState;
use serde::{Deserialize, Serialize};

use crate::db::{genres::*, musicians::*, partitions::update_partition, partitions::*};
use crate::errors::AppError;
use crate::models::genre::Genre;
use crate::models::musician::Person;
use crate::models::partition::ShowPartition;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Payload {
    pub name: String,
}

//***********************************************************************************
// CRUD Operations
//

///
/// Create a new partition in the partitions table
/// and shows the list of all partitions
///
/// Returns PartitionResponse or AppError
///
#[debug_handler]
pub async fn create_partition_hdl(
    State(state): State<AppState>,
    flash: Flash,
    Form(form): Form<ShowPartition>,
) -> (Flash, Redirect) {
    let partition_title = form.title;
    let person_name = form.full_name;
    let genre_name = form.name;

    if let Ok(partition) =
        add_partition(partition_title, person_name, genre_name, &state.pool).await
    {
        tracing::info!("partition added : {:?}", partition);
        let message = format!("Partition ajoutée : {}", partition.title);
        (flash.success(message), Redirect::to("/partitions"))
    } else {
        tracing::info!("error adding partition");
        let message = "Partition pas ajoutée".to_string();
        (flash.error(message), Redirect::to("/partitions"))
    }
}
#[debug_handler]
pub async fn update_partition_hdl(
    State(state): State<AppState>,
    flash: Flash,
    Path(id): Path<i32>,
    Form(form): Form<ShowPartition>,
) -> Result<(Flash, Redirect), AppError> {
    let partition_title = form.title;

    let person = find_persons_by_name_strict(form.full_name, &state.pool).await?;
    let person_id = person.id;

    let genre = find_genre_by_name(form.name, &state.pool).await?;
    let genre_id = genre[0].id;

    if let Ok(partition) =
        update_partition(id, partition_title, person_id, genre_id, &state.pool).await
    {
        tracing::info!("partition modified : {:?}", partition);
        let message = format!("Partition modifiée : {}", partition.title);
        Ok((flash.success(message), Redirect::to("/partitions")))
    } else {
        tracing::info!("error modifying partition");
        let message = "Partition pas modifiée".to_string();
        Ok((flash.error(message), Redirect::to("/partitions")))
    }
}
#[debug_handler]
pub async fn delete_partition_hdl(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    flash: Flash,
) -> (Flash, Redirect) {
    if let Ok(deleted_partition) = delete_partition(id, &state.pool).await {
        let message = format!("Partition effacée : {deleted_partition}");
        (flash.success(message), Redirect::to("/partitions"))
    } else {
        let message = "Partition pas effacée".to_string();
        (flash.error(message), Redirect::to("/partitions"))
    }
}

//*******************************************************************************
// Functions to show or print list of partitions
//

///
/// Shows the page with the list of partitions via ShowPartition
///
/// Returns a HTML Page or AppError
///
#[debug_handler]
pub async fn manage_partitions_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
) -> Result<(IncomingFlashes, HtmlTemplate<HandlePartitionsTemplate>), AppError> {
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);

    let show_partitions = list_show_partitions(&state.pool).await?;
    //set_static_vec_partitions(list_show_partitions(pool).await?);
    //let show_partitions = get_static_vec_partitions();

    let persons = list_persons(&state.pool).await?;
    let genres = list_genres(&state.pool).await?;
    let title = "Gestion des Partitions".to_string();
    let flash = Some(flash);
    let template = HandlePartitionsTemplate {
        title,
        flash,
        partitions: show_partitions,
        persons,
        genres,
    };
    Ok((in_flash, HtmlTemplate(template)))
}

///
/// Shows a printable list of all partitions in the db
/// under the form of ShowPartitions
///
/// Returns a HTML Page or AppError
///
#[debug_handler]
pub async fn print_list_partitions_hdl(
    State(state): State<AppState>,
) -> Result<HtmlTemplate<ListPartitionsTemplate>, AppError> {
    let list_partitions = list_show_partitions(&state.pool).await?;
    //let show_partitions = get_static_vec_partitions();

    let title = "liste des partitions".to_string();
    let template = ListPartitionsTemplate {
        title,
        list_partitions,
    };
    Ok(HtmlTemplate(template))
}

//*************************************************************************************
// Functions to find one or several partitions based on different criteria
//

///
/// find_partition_by_title
///
/// returns list musicians page with partition(s) found by title
///
#[debug_handler]
pub async fn find_partition_title_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
    Form(form): Form<Payload>,
) -> Result<(IncomingFlashes, HtmlTemplate<HandlePartitionsTemplate>), AppError> {
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);
    let partitions = find_partition_by_title(form.name, &state.pool).await?;
    let title = "Partition(s) trouvée(s)".to_string();

    let show_partitions = vec_showpartitions_from_vec_partitions(partitions, &state.pool).await;
    //set_static_vec_partitions(sh_partitions);
    //let show_partitions = get_static_vec_partitions();

    let persons = list_persons(&state.pool).await?;
    let genres = list_genres(&state.pool).await?;
    let flash = Some(flash);
    let template = HandlePartitionsTemplate {
        title,
        flash,
        partitions: show_partitions,
        persons,
        genres,
    };
    Ok((in_flash, HtmlTemplate(template)))
}

#[debug_handler]
pub async fn find_partition_genre_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
    Form(form): Form<Payload>,
) -> Result<(IncomingFlashes, HtmlTemplate<HandlePartitionsTemplate>), AppError> {
    let title = "Partition(s) trouvée(s)".to_string();
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);
    let partitions = find_partition_by_genre(form.name, &state.pool).await?;
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let show_part = show_one_partition(partition, &state.pool).await?;
        show_partitions.push(show_part);
    }
    let persons = list_persons(&state.pool).await?;
    let genres = list_genres(&state.pool).await?;
    let flash = Some(flash);

    let template = HandlePartitionsTemplate {
        title,
        flash,
        partitions: show_partitions,
        persons,
        genres,
    };
    Ok((in_flash, HtmlTemplate(template)))
}

#[debug_handler]
pub async fn find_partition_author_hdl(
    State(state): State<AppState>,
    in_flash: IncomingFlashes,
    Form(form): Form<Payload>,
) -> Result<(IncomingFlashes, HtmlTemplate<HandlePartitionsTemplate>), AppError> {
    let title = "Partition(s) trouvée(s)".to_string();
    let flash = in_flash
        .into_iter()
        .map(|(level, text)| format!("{:?}: {}", level, text))
        .collect::<Vec<_>>()
        .join(", ");
    tracing::info!("flash : {}", flash);
    let partitions = find_partition_by_author(form.name, &state.pool).await?;
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let show_part = show_one_partition(partition, &state.pool).await?;
        show_partitions.push(show_part);
    }
    //set_static_vec_partitions(show_partitions);
    //let show_partitions = get_static_vec_partitions();

    let persons = list_persons(&state.pool).await?;
    let genres = list_genres(&state.pool).await?;
    let flash = Some(flash);
    let template = HandlePartitionsTemplate {
        title,
        flash,
        partitions: show_partitions,
        persons,
        genres,
    };
    Ok((in_flash, HtmlTemplate(template)))
}
