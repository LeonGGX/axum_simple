//! usr/src/user.rs

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub user_name: String,
    pub password: String,
    pub role: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginUser {
    pub user_name: String,
    pub password: String,
}
