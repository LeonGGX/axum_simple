//! src/db/users.rs
//!
//! CRUD operations on users (using struct User) with sqlx and PostgresQL DB
//! Search operations on users in the DB
//!

use crate::errors::MyAppError;
use crate::models::user::{FilteredUser, NewUser, User};
use sqlx::PgPool;
use uuid::Uuid;

///
/// # Finds user by name
///
/// returns a Result with an Option(User)
/// with corresponding name from DB or a MyAppError
///
#[allow(dead_code)]
pub async fn find_user_by_name(name: String, pool: &PgPool) -> Result<Option<User>, MyAppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE name = $1", name)
        .fetch_optional(pool)
        .await?;
    //.map_err(|err| MyAppError::from(err))?;
    Ok(user)
}
///
/// # Finds user by credentials
///
/// **Retrieves a user based on the user name and password fields in the DB**    
/// the password must be hashed before using the function.    
///
/// returns a Result with an Option(User) or MyAppError
/// Option because the query can find no user with these credentials without this being a DB error
///
#[allow(dead_code)]
pub async fn find_user_by_credentials(
    user_name: String,
    password_hash: String,
    pool: &PgPool,
) -> Result<Option<User>, MyAppError> {
    let option_user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE name = $1 AND password = $2",
        user_name,
        password_hash
    )
    .fetch_optional(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;

    Ok(option_user)
}
///
/// # Find a user by email
///
/// **Retrieves a user by the email field which is unique in the DB**    
/// Returns a Result with Option(User) or MyAppError    
/// Option because the query can find no user with this email without this being a DB error
///
#[allow(dead_code)]
pub async fn find_user_by_email(email: String, pool: &PgPool) -> Result<Option<User>, MyAppError> {
    let option_user = sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
        .fetch_optional(pool)
        .await?;
    //.map_err(|err| MyAppError::from(err))?;
    Ok(option_user)
}
///
/// # find_user_by_id
///
/// Retrieves a user by the id field which is unique in the DB    
/// Returns a Result with an Option<User> or MyAppError     
/// Option is used since there can be no user with that id in the DB     
/// and this is no error
///
#[allow(dead_code)]
pub async fn find_user_by_id(id: Uuid, pool: &PgPool) -> Result<Option<User>, MyAppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(pool)
        .await?;
    //.map_err(|err| MyAppError::from(err))?;
    Ok(user)
}

//*********************************************************************************
// CRUD FUNCTIONS

///
/// **Adds a user to the users table**<br>
/// a hashed password must be set before entering the function<br>
/// RETURNING id, name, email, photo, verified, password, role,
///
#[allow(dead_code)]
pub async fn add_user(new_user: &NewUser, pool: &PgPool) -> Result<User, MyAppError> {
    tracing::info!("fonction add_user: NewUser = {:#?}: ", new_user);

    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (name, email, password, role)
            VALUES ($1, $2, $3, $4)            
            RETURNING *",
        new_user.name,
        new_user.email,
        new_user.password,
        new_user.role,
    )
    .fetch_one(pool)
    .await?;
    //.map_err(|err| MyAppError::from(err))?;

    Ok(user)
}

//*****************************************************************************
//DISPLAY FUNCTIONS

///
/// # list_users
/// **Returns a Result with a list of users or sqlxError**   
/// The list is populated with FilteredUser instead of User to hide sensitive data
///
pub async fn list_users(pool: &PgPool) -> Result<Vec<FilteredUser>, MyAppError> {
    let mut list_filtered_users: Vec<FilteredUser> = Vec::new();
    let users = sqlx::query_as!(User, "SELECT * FROM users ORDER BY name")
        .fetch_all(pool)
        .await?;
    //.map_err(|err| MyAppError::from(err))?;

    for user in users {
        list_filtered_users.push(FilteredUser {
            id: user.id.to_string(),
            name: user.name,
            email: user.email,
            role: user.role,
            photo: user.photo,
            verified: user.verified,
            created_at: user.created_at.unwrap(),
            updated_at: user.updated_at.unwrap(),
        });
    }
    Ok(list_filtered_users)
}
