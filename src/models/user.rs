//! usr/src/models/user.rs

//use axum_login::{secrecy::SecretVec, AuthLayer, AuthUser, RequireAuthorizationLayer};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::fmt::Formatter;
use uuid::Uuid;

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
    pub created_at: Option<time::Time>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<time::Time>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: String,
}
#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FilteredUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub photo: String,
    pub verified: bool,
    //pub createdAt: Option<DateTime<Utc>>,
    //pub createdAt: Option<time::Time>,
    //pub updatedAt: Option<DateTime<Utc>>,
    //pub updatedAt: Option<time::Time>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<time::Time>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<time::Time>,
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
impl Role {
    fn to_string(&self) -> String {
        if *self == Role::Administrateur {
            "Administrateur".to_string()
        } else if *self == Role::Utilisateur {
            "Utilisateur".to_string()
        } else {
            "Autre".to_string()
        }
    }
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
/*
impl ToString for Role {
    fn to_string(&self) -> String {
        if *self == Role::Administrateur {
            "Administrateur".to_string()
        } else if *self == Role::Utilisateur {
            "Utilisateur".to_string()
        } else {
            "Autre".to_string()
        }
    }
}
*/
/*
impl AuthUser<Role> for User {
    fn get_id(&self) -> String {
        format!("{}", self.id)
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.clone().into())
    }

    fn get_role(&self) -> Option<Role> {
        Some(self.role.clone())
    }
}
*/
