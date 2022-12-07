//! src/db/partitions.rs

use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::db::genres::find_genre_by_name;
use crate::db::musicians::find_persons_by_name_strict;
use crate::models::partition::{Partition, ShowPartition};

//*******************************************************************************************
// CRUD Operations on partitions
//
pub async fn add_partition(
    title: String,
    person_name: String,
    genre_name: String,
    pool: &PgPool,
) -> sqlx::Result<Partition> {
    let person_id: i32;
    let genre_id: i32;

    if let Ok(person) = find_persons_by_name_strict(person_name, &pool).await {
        tracing::info!("from db::add_partition : personne : {:?}", person);
        person_id = person.id;
    } else {
        return Err(sqlx::Error::RowNotFound);
    };

    if let Ok(genre) = find_genre_by_name(genre_name, &pool).await {
        tracing::info!("from db::add_partition : genre : {:?}", genre);
        genre_id = genre[0].id;
    } else {
        return Err(sqlx::Error::RowNotFound);
    };

    let partition = sqlx::query(
        "INSERT INTO partitions (title, person_id, genre_id)
                VALUES ( $1, $2, $3 )
                RETURNING id, title, person_id, genre_id;",
    )
    .bind(&title)
    .bind(&person_id)
    .bind(&genre_id)
    .map(|row: PgRow| Partition {
        id: row.get(0),
        title: row.get(1),
        person_id: row.get(2),
        genre_id: row.get(3),
    })
    .fetch_one(pool)
    .await?;

    tracing::info!("db : partition added : {:?}", &partition);

    Ok(partition)
}

pub async fn update_partition(
    id: i32,
    partition_title: String,
    person_id: i32,
    genre_id: i32,
    pool: &PgPool,
) -> sqlx::Result<Partition> {
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
        person_id: row.person_id.unwrap(),
        genre_id: row.genre_id.unwrap(),
    };
    Ok(partition)
}

pub async fn delete_partition(id: i32, pool: &PgPool) -> sqlx::Result<String> {
    let partition = find_partition_by_id(id.clone(), pool).await?;
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
/// Returns a list of all partitions in the db
/// under the form of a Vec<ShowPartition>
/// or a sqlx Error
///
pub async fn list_show_partitions(pool: &PgPool) -> anyhow::Result<Vec<ShowPartition>> {
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
/// Return a readable partition (ShowPartition) from a Partition
/// or sqlxError
///
pub async fn show_one_partition(
    partition: Partition,
    pool: &PgPool,
) -> sqlx::Result<ShowPartition> {
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

    Ok(show_partition)
}

pub async fn find_partition_by_id(id: i32, pool: &PgPool) -> sqlx::Result<Partition> {
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
/// find_partition_by_title
///
/// Un titre tronqué entré retourne la liste des partitions qui commencent
/// par les lettres entrées.
/// Permet une recherche lorsqu'on ne connaît pas le titre exact.
///
pub async fn find_partition_by_title(title: String, pool: &PgPool) -> sqlx::Result<Vec<Partition>> {
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
) -> sqlx::Result<Vec<Partition>> {
    let genre = find_genre_by_name(genre_name.clone(), pool).await?;
    let genre_id = genre[0].id;

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
) -> sqlx::Result<Vec<Partition>> {
    let author = find_persons_by_name_strict(author_name.clone(), pool).await?;
    //let author_id = author.id;

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

pub async fn vec_showpartitions_from_vec_partitions(
    partitions: Vec<Partition>,
    pool: &PgPool,
) -> Vec<ShowPartition> {
    let mut show_partitions: Vec<ShowPartition> = Vec::new();
    for partition in partitions {
        let one_show_partition = show_one_partition(partition, pool).await.unwrap();
        show_partitions.push(one_show_partition);
    }
    show_partitions
}
