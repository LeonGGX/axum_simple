//! src/db/genres

use crate::errors::MyAppError;
use axum::http::StatusCode;
use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::models::genre::Genre;

//*******************************************************************************************
// CRUD Operations on genres
//

///
/// **Adds a genre to the DB**<br>
/// returns the added Genre or sqlx::Error
///
pub async fn add_genre(pool: &PgPool, genre_name: String) -> Result<Genre, MyAppError> {
    let genre = sqlx::query_as!(
        Genre,
        "INSERT INTO genres (name) VALUES ( $1 )RETURNING id, name;",
        genre_name
    )
    .fetch_one(pool)
    .await?;

    Ok(genre)
}

pub async fn update_genre(id: i32, genre_name: String, pool: &PgPool) -> Result<Genre, MyAppError> {
    let genre = sqlx::query_as!(
        Genre,
        "UPDATE genres SET name = $1 WHERE id = $2 RETURNING id, name;",
        genre_name,
        id,
    )
    .fetch_one(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;
    Ok(genre)
    /*
    let genre = sqlx::query("UPDATE genres SET name = $1 WHERE id = $2 RETURNING id, name;")
        .bind(&genre_name)
        .bind(id)
        .map(|row: PgRow| Genre {
            id: row.get(0),
            name: row.get(1),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!("db : Genre updated : {:?}", &genre);
    Ok(genre)
         */
}

pub async fn delete_genre(id: i32, pool: &PgPool) -> Result<String, MyAppError> {
    let genre = find_genre_by_id(id, pool).await?;
    if genre.is_none() {
        Err(MyAppError::new(
            StatusCode::NOT_FOUND,
            "Genre with the ID not found !",
        ))
    } else {
        let genre = genre.unwrap();
        let name = genre.name;

        let _res = sqlx::query("DELETE FROM genres WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        tracing::info!("db : Genre deleted : {}", &name);
        Ok(name)
    }
}

///
/// Returns a list of genres
/// under the form of a Vec<Genre>
/// or a sqlx Error
///
pub async fn list_genres(pool: &PgPool) -> Result<Vec<Genre>, MyAppError> {
    let genres: Vec<Genre> = sqlx::query("SELECT id, name FROM genres ORDER BY name;")
        .map(|row: PgRow| Genre {
            id: row.get(0),
            name: row.get(1),
        })
        .fetch_all(pool)
        .await?;
    Ok(genres)
}

pub async fn find_genre_by_id(id: i32, pool: &PgPool) -> Result<Option<Genre>, MyAppError> {
    let genre = sqlx::query_as!(Genre, "SELECT * FROM genres WHERE id = $1;", id,)
        .fetch_optional(pool)
        .await?;
    //.map_err(|err| MyAppError::from(err))?;
    /*
        let genre = sqlx::query("SELECT * FROM genres WHERE id = $1;")
            .bind(id)
            .map(|row: PgRow| Genre {
                id: row.get(0),
                name: row.get(1),
            })
            .fetch_one(pool)
            .await?;
    */
    //tracing::info!("db : Genre trouvé : {}", &genre.name);
    Ok(genre)
}

pub async fn find_genre_by_name_strict(name: String, pool: &PgPool) -> Result<Genre, MyAppError> {
    let genre = sqlx::query_as!(Genre, "SELECT * FROM genres WHERE name = $1;", name)
        .fetch_one(pool)
        .await?;
    Ok(genre)
}
///
/// **find genre by name parts**<br>
/// (1st letter e.g.)<br>
/// returns one genre or a vector of genres by name<br>
///
/// e.g. two genres in the DB Chanson, ChaChaCha and Classique<br>
/// if you enter "C" the function returns all three<br>
/// if you enter "CH" the function returns Chanson and ChaChaCha<br>
/// if you enter "CLA" the function returns Classique (idem if you enter CLASSIQUE)<br>
///
pub async fn find_genre_by_name_parts(
    name: String,
    pool: &PgPool,
) -> Result<Vec<Genre>, MyAppError> {
    let mut part_name = name.clone();
    part_name.push('%');

    let genre = sqlx::query("SELECT * FROM genres WHERE name LIKE $1;")
        .bind(part_name)
        .map(|row: PgRow| Genre {
            id: row.get("id"),
            name: row.get("name"),
        })
        .fetch_all(pool)
        .await?;
    Ok(genre)
}
