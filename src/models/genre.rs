//! src/models/genre.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, FromRow)]
pub struct Genre {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize, FromRow, Debug, Eq, PartialEq)]
pub struct NewGenre {
    pub name: String,
}
