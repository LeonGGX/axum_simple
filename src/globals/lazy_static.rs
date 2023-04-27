//! src/globals/lazy_static.rs

use crate::models::{genre::Genre, musician::Person, partition::ShowPartition};
use std::sync::Mutex;

//*************************************************
// using crate lazy_static
//
use lazy_static::lazy_static;

lazy_static! {
    static ref VEC_PERSONS: Mutex<Vec<Person>> = Mutex::new(Vec::new());
    static ref VEC_GENRES: Mutex<Vec<Genre>> = Mutex::new(Vec::new());
    static ref VEC_SHOWPARTITIONS: Mutex<Vec<ShowPartition>> = Mutex::new(Vec::new());
}
///
/// # get_static_vec_persons
///
/// Returns a static vector of Person      
/// Useful to get data independent from the view    
/// used to show the result of a research on musicians   
///
#[allow(dead_code)]
pub fn get_static_vec_persons() -> Vec<Person> {
    VEC_PERSONS.lock().unwrap().clone()
}
///
/// # set_static_vec_persons
///
/// Populates a static vector of Person    
/// Argument : Vector of Person    
#[allow(dead_code)]
///
pub fn set_static_vec_persons(musicians: Vec<Person>) {
    *VEC_PERSONS.lock().unwrap() = musicians;
}
///
/// # get_static_vec_genres
///
/// Returns a static vector of Genre      
/// Useful to get data independent from the view    
/// used to show the result of a research on genres    
///
#[allow(dead_code)]
pub fn get_static_vec_genres() -> Vec<Genre> {
    VEC_GENRES.lock().unwrap().clone()
}
///
/// # set_static_vec_genres
///
/// Populates a static vector of Genre    
/// Argument : Vector of Genre    
///
#[allow(dead_code)]
pub fn set_static_vec_genres(genres: Vec<Genre>) {
    *VEC_GENRES.lock().unwrap() = genres;
}
///
/// # get_static_vec_partitions
///
/// Returns a static vector of ShowPartition      
/// Useful to get data independent from the view    
/// used to show the result of a research on partitions    
///
#[allow(dead_code)]
pub fn get_static_vec_partitions() -> Vec<ShowPartition> {
    VEC_SHOWPARTITIONS.lock().unwrap().clone()
}
///
/// # set_static_vec_partitions
///
/// Populates a static vector of ShowPartition    
/// Argument : Vector of ShowPartition
///
#[allow(dead_code)]
pub fn set_static_vec_partitions(partitions: Vec<ShowPartition>) {
    *VEC_SHOWPARTITIONS.lock().unwrap() = partitions;
}
