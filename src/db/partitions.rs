//! src/db/partitions.rs

use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::db::genres::find_genre_by_name_strict;
use crate::db::musicians::find_persons_by_name_strict;
use crate::errors::MyAppError;
use crate::models::partition::{Partition, ShowPartition};

//*******************************************************************************************
// CRUD Operations on partitions
//
///
/// **Adds a partition to the DB**<br>
/// requires a title, the musician name and the genre<br>
/// requires a PgPool<br>
/// uses sqlx::query_as! macro<br>
/// in the sql query when returning the id to build the Person struct,<br>
/// it's necessary to write RETURNING id as "id?", ... or we get an error
/// from the DB
///
pub async fn add_partition(
    title: String,
    person_name: String,
    genre_name: String,
    pool: &PgPool,
) -> Result<Partition, MyAppError> {
    //let person_id: i32;
    //let genre_id: i32;

    let person = find_persons_by_name_strict(person_name, pool).await?;
    let person_id = person.id;

    let genre = find_genre_by_name_strict(genre_name, pool).await?;
    let genre_id = genre.id;

    let partition: Partition = sqlx::query_as!(
        Partition,
        r#"INSERT INTO partitions (title, person_id, genre_id)
                VALUES ( $1, $2, $3 )
                RETURNING id as "id?", title, person_id, genre_id;"#,
        title,
        person_id,
        genre_id,
    )
    .fetch_one(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;

    tracing::info!("db : partition added : {:?}", &partition);
    Ok(partition)
}

pub async fn update_partition(
    id: i32,
    partition_title: String,
    person_id: i32,
    genre_id: i32,
    pool: &PgPool,
) -> Result<Partition, MyAppError> {
    let row = sqlx::query!(
        r#"
        UPDATE partitions
        SET title = $1, person_id = $2, genre_id = $3
        WHERE id = $4
        RETURNING id, title, person_id, genre_id
        "#,
        partition_title,
        person_id,
        genre_id,
        id,
    )
    .fetch_one(pool)
    .await?;
    let partition = Partition {
        id: Some(row.id),
        title: row.title,
        person_id: row.person_id,
        genre_id: row.genre_id,
    };
    Ok(partition)
}

pub async fn delete_partition(id: i32, pool: &PgPool) -> Result<String, MyAppError> {
    let partition = find_partition_by_id(id, pool).await?;
    let name = partition.title;

    let _res = sqlx::query("DELETE FROM partitions WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    tracing::info!("db : Partition deleted : {}", &name);

    Ok(name)
}

//******************************************************************************************
// Retrieve lists from partitions tables
//

///
/// **Returns a list of all partitions in the db**<br>
/// under the form of a Vec<ShowPartition>
/// or a MyAppError
///
pub async fn list_show_partitions(pool: &PgPool) -> Result<Vec<ShowPartition>, MyAppError> {
    let rep: Vec<ShowPartition> = sqlx::query(
        "
    SELECT partitions.id, partitions.title, persons.full_name, genres.name
    FROM partitions
    INNER JOIN persons
    ON partitions.person_id = persons.id
    INNER JOIN genres
    ON partitions.genre_id = genres.id
    ORDER BY partitions.title
        ",
    )
    .map(|row: PgRow| ShowPartition {
        id: row.get(0),
        title: row.get(1),
        full_name: row.get(2),
        name: row.get(3),
    })
    .fetch_all(pool)
    .await?;
    Ok(rep)
}

///
/// **Returns a readable partition (ShowPartition) from a Partition Struct**<br>
/// or MyAppError
///
pub async fn show_one_partition(
    partition: Partition,
    pool: &PgPool,
) -> Result<ShowPartition, MyAppError> {
    let show_partition = sqlx::query(
        "
    SELECT partitions.id, partitions.title, persons.full_name, genres.name
    FROM partitions
    INNER JOIN persons
    ON partitions.person_id = persons.id
    INNER JOIN genres
    ON partitions.genre_id = genres.id
    WHERE partitions.title = $1
        ",
    )
    .bind(partition.title)
    .map(|row: PgRow| ShowPartition {
        id: row.get(0),
        title: row.get(1),
        full_name: row.get(2),
        name: row.get(3),
    })
    .fetch_one(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;

    Ok(show_partition)
}

pub async fn find_partition_by_id(id: i32, pool: &PgPool) -> Result<Partition, MyAppError> {
    let partition = sqlx::query("SELECT * FROM partitions WHERE id = $1;")
        .bind(id)
        .map(|row: PgRow| Partition {
            id: row.get("id"),
            title: row.get("title"),
            person_id: row.get("person_id"),
            genre_id: row.get("genre_id"),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!("db : partition trouvée (titre) : {}", &partition.title);
    Ok(partition)
}

///
/// **Find_partition_by_title**
///
/// Un titre tronqué entré retourne la liste des partitions qui commencent
/// par les lettres entrées.
/// Permet une recherche lorsqu'on ne connaît pas le titre exact.
///
pub async fn find_partition_by_title(
    title: String,
    pool: &PgPool,
) -> Result<Vec<Partition>, MyAppError> {
    let mut part_title = title.clone();
    part_title.push('%');

    let partitions = sqlx::query("SELECT * FROM partitions WHERE title LIKE $1;")
        .bind(part_title)
        .map(|row: PgRow| Partition {
            id: row.get("id"),
            title: row.get("title"),
            person_id: row.get("person_id"),
            genre_id: row.get("genre_id"),
        })
        .fetch_all(pool)
        .await?;

    Ok(partitions)
}

pub async fn find_partition_by_genre(
    genre_name: String,
    pool: &PgPool,
) -> Result<Vec<Partition>, MyAppError> {
    let genre = find_genre_by_name_strict(genre_name.clone(), pool).await?;
    let genre_id = genre.id;

    let partitions = sqlx::query(
        "SELECT * FROM partitions \
        WHERE genre_id = $1 \
        ORDER BY partitions.title",
    )
    .bind(genre_id)
    .map(|row: PgRow| Partition {
        id: row.get("id"),
        title: row.get("title"),
        person_id: row.get("person_id"),
        genre_id: row.get("genre_id"),
    })
    .fetch_all(pool)
    .await?;

    tracing::info!("db : partition(s) trouvée(s) pour genre : {}", &genre_name);
    Ok(partitions)
}

pub async fn find_partition_by_author(
    author_name: String,
    pool: &PgPool,
) -> Result<Vec<Partition>, MyAppError> {
    let author = find_persons_by_name_strict(author_name.clone(), pool).await?;
    let partitions = sqlx::query(
        "SELECT * FROM partitions \
        WHERE person_id = $1\
        ORDER BY partitions.title",
    )
    .bind(author.id)
    .map(|row: PgRow| Partition {
        id: row.get("id"),
        title: row.get("title"),
        person_id: row.get("person_id"),
        genre_id: row.get("genre_id"),
    })
    .fetch_all(pool)
    .await?;

    tracing::info!(
        "db : partition(s) trouvée(s) pour auteur : {}",
        &author_name
    );
    Ok(partitions)
}

#[allow(dead_code)]
pub async fn vec_showpartitions_from_vec_partitions(
    partitions: Vec<Partition>,
    pool: &PgPool,
) -> Result<Vec<ShowPartition>, MyAppError> {
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let one_show_partition = show_one_partition(partition, pool).await?;
        show_partitions.push(one_show_partition);
    }
    Ok(show_partitions)
}
