//! src/db/users.rs
//!
//! CRUD operations on users (using struct User) with sqlx and PostgresQL DB
//! Search operations on users in the DB
//!

use crate::models::user::{FilteredUser, NewUser, User};
use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgTimeTz;
use sqlx::PgPool;
use uuid::Uuid;

//******************************************************************************************
// Authentication functions
//
#[allow(dead_code)]
pub async fn find_user_by_name(name: String, pool: &PgPool) -> sqlx::Result<User> {
    let row = sqlx::query!("SELECT * FROM users WHERE name = $1", name)
        .fetch_one(pool)
        .await?;

    let user = User {
        id: row.id,
        name: row.name,
        email: row.email,
        photo: row.photo,
        verified: row.verified,
        password: row.password,
        role: row.role,
        created_at: None,
        updated_at: None,
    };
    Ok(user)
}

///
/// # find_user_by_credentials
///
/// Retrieves a user based on the user name and password fields in the DB    
/// the password must be hashed before using the function.    
///
/// returns a Result with User or sqlxError
///
#[allow(dead_code)]
pub async fn find_user_by_credentials(
    user_name: String,
    password_hash: String,
    pool: &PgPool,
) -> sqlx::Result<User> {
    let row = sqlx::query!(
        "SELECT * FROM users WHERE name = $1 AND password = $2",
        user_name,
        password_hash,
    )
    .fetch_one(pool)
    .await?;

    let user = User {
        id: row.id,
        name: row.name,
        email: row.email,
        photo: row.photo,
        verified: row.verified,
        password: row.password,
        role: row.role,
        created_at: None,
        updated_at: None,
    };
    Ok(user)
}
///
/// # find_user_by_email
/// Retrieves a user by the email field which is unique in the DB    
/// Returns a Result with the corresponding user (User) or a sqlx::Error
///
#[allow(dead_code)]
pub async fn find_user_by_email(email: String, pool: &PgPool) -> sqlx::Result<User> {
    let row = sqlx::query!("SELECT * FROM users WHERE email = $1", email)
        .fetch_one(pool)
        .await?;

    let user = User {
        id: row.id,
        name: row.name,
        email: row.email,
        photo: row.photo,
        verified: row.verified,
        password: row.password,
        role: row.role,
        created_at: None,
        updated_at: None,
    };
    Ok(user)
}
///
/// # find_user_by_id
/// Retrieves a user by the id field which is unique in the DB    
/// Returns a Result with the corresponding user (User) or a sqlx::Error
///
#[allow(dead_code)]
pub async fn find_user_by_id(id: Uuid, pool: &PgPool) -> sqlx::Result<User> {
    let row = sqlx::query!("SELECT * FROM users WHERE id = $1", id)
        .fetch_one(pool)
        .await?;

    let user = User {
        id: row.id,
        name: row.name,
        email: row.email,
        photo: row.photo,
        verified: row.verified,
        password: row.password,
        role: row.role,
        created_at: None,
        updated_at: None,
    };
    Ok(user)
}

/*********************************************************************************
CRUD FUNCTIONS
 */
///
/// Adds a user to the users table
/// a hashed password must be set before entering the function
/// //RETURNING id, name, email, photo, verified, password, role",
#[allow(dead_code)]
pub async fn add_user(new_user: &NewUser, pool: &PgPool) -> sqlx::Result<User> {
    tracing::info!("fonction add_user: NewUser = {:#?}: ", new_user);
    let row = sqlx::query!(
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

    let user = User {
        id: row.id,
        name: row.name,
        email: row.email,
        photo: row.photo,
        verified: row.verified,
        password: row.password,
        role: row.role,
        created_at: row.created_at,
        updated_at: row.updated_at,
    };
    tracing::info!("{:#?}", user);
    Ok(user)
}

/*****************************************************************************
DISPLAY FUNCTIONS
 */
///
/// # list_users
/// Returns a Result with a list of users or sqlxError   
/// The list is populated with FilteredUser to hide sensitive data
///
pub async fn list_users(pool: &PgPool) -> sqlx::Result<Vec<FilteredUser>> {
    let mut list_safe_users: Vec<FilteredUser> = Vec::new();
    let rows = sqlx::query!(
        //"SELECT id,name,email,role,photo,verified,created_at,updated_at FROM users ORDER BY name"
        "SELECT * FROM users ORDER BY name"
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        list_safe_users.push(FilteredUser {
            id: row.id.to_string(),
            name: row.name,
            email: row.email,
            role: row.role,
            photo: row.photo,
            verified: row.verified,
            created_at: row.created_at.unwrap(),
            updated_at: row.updated_at.unwrap(),
        });
    }
    Ok(list_safe_users)
}
