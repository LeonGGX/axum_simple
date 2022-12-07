//! src/db/musicians.rs

use sqlx::postgres::PgRow;
use sqlx::{PgPool, Row};

use crate::models::musician::Person;

//*******************************************************************************************
// CRUD Operations on persons - musicians
//
/// Adds a person (musician) to the DB based on the person name
/// * pool : &PgPool
/// * full_name: String\
/// \
/// returns a Person or sqlx Error
///
#[allow(dead_code)]
pub async fn add_person(pool: &PgPool, full_name: String) -> sqlx::Result<Person> {
    let person = sqlx::query_as!(
        Person,
        r#"
    INSERT INTO persons ( full_name )
    VALUES ( $1 )
    RETURNING id, full_name
            "#,
        full_name
    )
    .fetch_one(pool)
    .await?;

    tracing::info!("db : person added : {:?}", &person);
    Ok(person)
}

#[allow(dead_code)]
pub async fn update_person(id: i32, person_name: String, pool: &PgPool) -> sqlx::Result<Person> {
    let person =
        sqlx::query("UPDATE persons SET full_name = $1 WHERE id = $2 RETURNING id, full_name;")
            .bind(&person_name)
            .bind(id)
            .map(|row: PgRow| Person {
                id: row.get(0),
                full_name: row.get(1),
            })
            .fetch_one(pool)
            .await?;

    tracing::info!("db : Person updated : {:?}", &person);
    Ok(person)
}

#[allow(dead_code)]
pub async fn delete_person(id: i32, pool: &PgPool) -> sqlx::Result<String> {
    let pers = find_person_by_id(id, pool).await?;
    let name = pers.full_name;

    let _res = sqlx::query("DELETE FROM persons WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    tracing::info!("db : Person deleted : {}", &name);

    Ok(name)
}

//**********************************************************************************
// Fonctions de recherche d'enregistrements sur base de critères : nom, genre, titre, ...
//

///
/// find person by id
/// used as help function for others
///
#[allow(dead_code)]
pub async fn find_person_by_id(id: i32, pool: &PgPool) -> sqlx::Result<Person> {
    let person = sqlx::query("SELECT * FROM persons WHERE id = $1;")
        .bind(id)
        .map(|row: PgRow| Person {
            id: row.get(0),
            full_name: row.get(1),
        })
        .fetch_one(pool)
        .await?;

    tracing::info!("db : Personne trouvée : {}", &person.full_name);
    Ok(person)
}

///
/// find person by name strict
/// returns one author by name\
///
/// The full name should be introduced.
///
#[allow(dead_code)]
pub async fn find_persons_by_name_strict(full_name: String, pool: &PgPool) -> sqlx::Result<Person> {
    let select_query = sqlx::query(
        "SELECT * FROM persons \
                         WHERE full_name LIKE $1",
    );
    let person = select_query
        .bind(full_name)
        .map(|row: PgRow| Person {
            id: row.get("id"),
            full_name: row.get("full_name"),
        })
        .fetch_one(pool)
        .await?;

    Ok(person)
}

///
/// find person by name parts (1st letter e.g.)\
/// returns one author or a vector of authors by name\
///
/// e.g. two authors in the DB BREL, BRASSENS and BARTOK\
/// if you enter "B" the function returns all three\
/// if you enter "BR" the function returns BREL and BRASSENS\
/// if you enter "BRE" the function returns BREL (idem if you enter BREL)\
///
#[allow(dead_code)]
pub async fn find_persons_by_name_parts(
    full_name: String,
    pool: &PgPool,
) -> sqlx::Result<Vec<Person>> {
    let mut name = full_name.clone();
    name.push('%');

    let select_query = sqlx::query(
        "SELECT * FROM persons \
                         WHERE full_name LIKE $1",
    );
    let person = select_query
        .bind(name)
        .map(|row: PgRow| Person {
            id: row.get("id"),
            full_name: row.get("full_name"),
        })
        .fetch_all(pool)
        .await?;

    Ok(person)
}

///
/// Returns a list of musicians
/// under the form of a Vec<Person>
/// or a sqlx Error
///
#[allow(dead_code)]
pub async fn list_persons(pool: &PgPool) -> sqlx::Result<Vec<Person>> {
    //let mut persons: Vec<Person> = Vec::new();
    let recs = sqlx::query("SELECT id, full_name FROM persons ORDER BY full_name;")
        .map(|row: PgRow| Person {
            id: row.get("id"),
            full_name: row.get("full_name"),
        })
        .fetch_all(pool)
        .await?;

    Ok(recs)
}
