//! src/globals/once_cell.rs

use crate::models::genre::Genre;
use crate::models::musician::Person;
use crate::models::partition::ShowPartition;
use once_cell::sync::Lazy;
use std::sync::RwLock;

static VEC_PERSONS: Lazy<RwLock<Vec<Person>>> = Lazy::new(|| RwLock::new(vec![]));
static VEC_GENRES: Lazy<RwLock<Vec<Genre>>> = Lazy::new(|| RwLock::new(Vec::new()));
static VEC_SHOWPARTITIONS: Lazy<RwLock<Vec<ShowPartition>>> = Lazy::new(|| RwLock::new(Vec::new()));

///
/// # set_static_vec_persons
/// uses once_cell crate   
/// Populates a static vector of Person    
/// Argument : Vector of Person    
///
pub fn set_static_vec_persons(musicians: Vec<Person>) {
    *VEC_PERSONS.write().unwrap() = musicians;
}

///
/// # get_static_vec_persons
/// uses once_cell crate   
/// Returns a static vector of Person      
/// Useful to get data independent from the view    
/// used to show the result of a research on musicians   
///
pub fn get_static_vec_persons() -> Vec<Person> {
    VEC_PERSONS.read().unwrap().clone()
}

///
/// # set_static_vec_genres
/// uses once_cell crate    
/// Populates a static vector of Genre    
/// Argument : Vector of Genre    
///
pub fn set_static_vec_genres(genres: Vec<Genre>) {
    *VEC_GENRES.write().unwrap() = genres;
}

///
/// # get_static_vec_genres
/// uses once_cell crate    
/// Returns a static vector of Genre      
/// Useful to get data independent from the view    
/// used to show the result of a research on genres    
///
pub fn get_static_vec_genres() -> Vec<Genre> {
    VEC_GENRES.read().unwrap().clone()
}

///
/// # set_static_vec_partitions
/// uses once_cell crate    
/// Populates a static vector of ShowPartition    
/// Argument : Vector of ShowPartition
///
pub fn set_static_vec_partitions(partitions: Vec<ShowPartition>) {
    *VEC_SHOWPARTITIONS.write().unwrap() = partitions;
}

///
/// # get_static_vec_partitions
/// uses once_cell crate    
/// Returns a static vector of ShowPartition      
/// Useful to get data independent from the view    
/// used to show the result of a research on partitions    
///
pub fn get_static_vec_partitions() -> Vec<ShowPartition> {
    VEC_SHOWPARTITIONS.read().unwrap().clone()
}
