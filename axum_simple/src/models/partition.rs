//! src/models/partition.rs

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Partition {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub title: String,
    pub person_id: i32,
    pub genre_id: i32,
}
/*
///
/// une struct pour présenter les partitions avec les
/// éléments des différentes tables
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowPartition {
    #[serde(skip_deserializing)]
    pub id: Option<i32>,
    pub title: String,
    pub full_name: String,
    pub name: String,
}
*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShowPartition {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub title: String,
    pub full_name: String,
    pub name: String,
}
