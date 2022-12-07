//! src/models/musician
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// this struct will be used to represent database record
#[derive(Clone, Serialize, Deserialize, FromRow, Debug, Eq, PartialEq)]
pub struct Person {
    pub id: i32,
    pub full_name: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow, Debug, Eq, PartialEq)]
pub struct NewPerson {
    pub full_name: String,
}
