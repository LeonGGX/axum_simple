//! src/db/musicians.rs

use crate::errors::MyAppError;
use axum::http::StatusCode;
use sqlx::PgPool;

use crate::models::musician::Person;

//*******************************************************************************************
// CRUD Operations on persons - musicians
//
///
/// **Adds a person (musician) to the DB based on the person name**
/// * pool : &PgPool
/// * full_name: String
///
/// returns a Person or MyAppError
///
#[allow(dead_code)]
pub async fn add_person(pool: &PgPool, full_name: String) -> Result<Person, MyAppError> {
    let person = sqlx::query_as!(
        Person,
        "
    INSERT INTO persons ( full_name )
    VALUES ( $1 )
    RETURNING id, full_name
            ",
        full_name
    )
    .fetch_one(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;

    tracing::info!("db : person added : {:?}", &person);
    Ok(person)
}

///
/// **Updates a person (musician) in the DB based on the person id and name**
/// * pool : &PgPool
/// * id: i32
/// * full_name: String
///
/// returns the modified  Person or MyAppError
///
#[allow(dead_code)]
pub async fn update_person(
    id: i32,
    person_name: String,
    pool: &PgPool,
) -> Result<Person, MyAppError> {
    let person = sqlx::query_as!(
        Person,
        "UPDATE persons SET full_name = $1 WHERE id = $2 RETURNING id, full_name;",
        person_name,
        id,
    )
    .fetch_one(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;
    Ok(person)
}
///
/// **deletes a person (musician) from the DB on basis of the ID**<br>
/// first check if the person exists in the DB, then delete that person.<br>
/// if the person doesn't exist in the DB return MyAppError<br>
/// if the person cannot be deleted return MyAppError
///
/// Notice that the person normally exists since we start with the list of persons
/// in the DB ...
///
#[allow(dead_code)]
pub async fn delete_person(id: i32, pool: &PgPool) -> Result<Person, MyAppError> {
    let person = find_person_by_id(id, pool).await?;
    match person {
        Some(p) => {
            sqlx::query_as!(Person, "DELETE FROM persons WHERE id= $1", id,)
                .execute(pool)
                .await?;
            Ok(p)
        }
        None => Err(MyAppError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error: Person doesn't exist",
        )),
    }
}

//**********************************************************************************
// Fonctions de recherche d'enregistrements sur base de critères : nom, genre, titre, ...
//

///
/// **Finds one person by id**<br>
/// Returns an Option with the found Person<br>
/// Or None is there is no Person with the id in the DB<br>
/// In case of error (DB Error from sqlx) returns MyAppError
///
/// used as help function for others
///
#[allow(dead_code)]
pub async fn find_person_by_id(id: i32, pool: &PgPool) -> Result<Option<Person>, MyAppError> {
    let person = sqlx::query_as!(Person, "SELECT * FROM persons WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    //.map_err(|err| MyAppError::from(err))?;
    Ok(person)
}

///
/// **Finds a person by name strict**<br>
/// returns one and only one person (musician) by name
///
/// The full name should be introduced.
///
#[allow(dead_code)]
pub async fn find_persons_by_name_strict(
    full_name: String,
    pool: &PgPool,
) -> Result<Person, MyAppError> {
    let person = sqlx::query_as!(
        Person,
        "SELECT * FROM persons WHERE full_name = $1",
        full_name
    )
    .fetch_one(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;

    Ok(person)
}

///
/// **find person by name parts**<br>
/// (1st letter e.g.)<br>
/// returns one author or a vector of persons by name<br>
///
/// e.g. two authors in the DB BREL, BRASSENS and BARTOK<br>
/// if you enter "B" the function returns all three<br>
/// if you enter "BR" the function returns BREL and BRASSENS<br>
/// if you enter "BRE" the function returns BREL (idem if you enter BREL)<br>
///
#[allow(dead_code)]
pub async fn find_persons_by_name_parts(
    full_name: String,
    pool: &PgPool,
) -> Result<Vec<Person>, MyAppError> {
    let mut name = full_name.clone();
    name.push('%');

    let persons = sqlx::query_as!(
        Person,
        "SELECT * FROM persons WHERE full_name LIKE $1",
        name
    )
    .fetch_all(pool)
    .await?;
    Ok(persons)
}

///
/// **Returns a list of all musicians**<br>
/// under the form of a Vec(Person)
/// or a MyAppError
///
#[allow(dead_code)]
pub async fn list_persons(pool: &PgPool) -> Result<Vec<Person>, MyAppError> {
    let persons = sqlx::query_as!(
        Person,
        "SELECT id, full_name FROM persons ORDER BY full_name"
    )
    .fetch_all(pool)
    .await?;
    Ok(persons)
}
