//! usr/src/models/user.rs
//!
//! All the models used for managing users    
//!
//! in the PostgresQL DB DateTime-Utc is translated in 'time with time zone'
//! and "now()" as default
//! with PgAdmin.
//!

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::fmt::Formatter;
use uuid::Uuid;

///
/// # struct User
///
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub photo: String,
    pub verified: bool,
    pub password: String,
    pub role: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

///
/// # Struct NewUser   
/// to add a User to the DB    
/// the other fields have defoult values in the DB
///
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

///
/// # Struct FilteredUser
/// Struct used to show users data    
/// hides sensitive data as password
///
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FilteredUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub photo: String,
    pub verified: bool,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

pub fn filter_user_record(user: &User) -> FilteredUser {
    FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        photo: user.photo.to_owned(),
        role: user.role.to_owned(),
        verified: user.verified,
        created_at: user.created_at.unwrap(),
        updated_at: user.updated_at.unwrap(),
    }
}

//**************************************************************************
// axum-login

/// Role
/// to be used with axum-login
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum Role {
    Administrateur,
    Utilisateur,
    Autre,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<String> for Role {
    fn from(value: String) -> Self {
        if value == "Administrateur" {
            Self::Administrateur
        } else if value == "Utilisateur" {
            Self::Utilisateur
        } else {
            Self::Autre
        }
    }
}
