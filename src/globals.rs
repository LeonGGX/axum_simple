//! src/globals.rs

//*******************************************************************************
// Des variables globales qui fonctionnent dans le système async
// Elles vont servir à stocker des valeurs de la DB sous forme de vecteurs
// Ils vont servir à afficher le résultat d'une recherche dans la page d'affichage et aussi dans
// la page d'impression, d'où la nécessité d'être des variables globales
// *******************************************************************************

use crate::models::{genre::Genre, musician::Person, partition::ShowPartition};
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref VEC_PERSONS: Mutex<Vec<Person>> = Mutex::new(Vec::new());
    static ref VEC_GENRES: Mutex<Vec<Genre>> = Mutex::new(Vec::new());
    static ref VEC_SHOWPARTITIONS: Mutex<Vec<ShowPartition>> = Mutex::new(Vec::new());
}

pub fn get_static_vec_persons() -> Vec<Person> {
    VEC_PERSONS.lock().unwrap().clone()
}

pub fn set_static_vec_persons(musicians: Vec<Person>) {
    *VEC_PERSONS.lock().unwrap() = musicians;
}

pub fn get_static_vec_genres() -> Vec<Genre> {
    VEC_GENRES.lock().unwrap().clone()
}

pub fn set_static_vec_genres(genres: Vec<Genre>) {
    *VEC_GENRES.lock().unwrap() = genres;
}

pub fn get_static_vec_partitions() -> Vec<ShowPartition> {
    VEC_SHOWPARTITIONS.lock().unwrap().clone()
}

pub fn set_static_vec_partitions(partitions: Vec<ShowPartition>) {
    *VEC_SHOWPARTITIONS.lock().unwrap() = partitions;
}
