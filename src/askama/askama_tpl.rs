//! src/askama.rs

use crate::models::genre::Genre;
//use crate::models::partition::ShowPartition;
use crate::models::partition::ShowPartition;
use crate::Person;
use askama::Template;
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html")]
pub struct HelloTemplate {
    pub title: String,
    pub name: String,
}

///
/// When Option<T> is used in a template struct, the html template must
/// be written so :\
///  {% if let Some(some_flash) = flash %}
///         {{ some_flash }}
///    {% endif %}
///
#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    pub title: String,
    pub flash: Option<String>,
}

//***********************************************************************
// Templates to show a printable list of Musicians, genres and Partitions
//
#[derive(Template)] // this will generate the code...
#[template(path = "list_persons.html")]
pub struct ListPersonsTemplate {
    pub list_persons: Vec<Person>,
}

#[derive(Template)] // this will generate the code...
#[template(path = "list_genres.html")]
pub struct ListGenresTemplate {
    pub list_genres: Vec<Genre>,
}
/// # ListPartitionsTemplate
/// Askama Template to list partitions in text form
/// arguments :
/// * title : String
/// * list_partitions : Vector of ShowPartition
///
#[derive(Template)] // this will generate the code...
#[template(path = "list_partitions.html")]
pub struct ListPartitionsTemplate {
    pub title: String,
    pub list_partitions: Vec<ShowPartition>,
}

//*****************************************************************************
// Templates to manage persons/genres/partitions
// main page for each category
//
///
/// When Option<T> is used in a template struct, the html template must
/// be written so :\
///  {% if let Some(some_flash) = flash %}
///         {{ some_flash }}
///    {% endif %}
///
#[derive(Template)] // this will generate the code...
#[template(path = "persons.html")]
pub struct HandlePersonsTemplate {
    pub title: String,
    pub flash: Option<String>,
    pub persons: Vec<Person>,
}

#[derive(Template)] // this will generate the code...
#[template(path = "genres.html")]
pub struct HandleGenresTemplate {
    pub title: String,
    pub flash: Option<String>,
    pub genres: Vec<Genre>,
}

#[derive(Template)] // this will generate the code...
#[template(path = "partitions.html")]
pub struct HandlePartitionsTemplate {
    pub title: String,
    pub flash: Option<String>,
    pub partitions: Vec<ShowPartition>,
    pub persons: Vec<Person>,
    pub genres: Vec<Genre>,
}

//*************************************************************************
// Template for the main page, starting the application
//
#[derive(Template)] // this will generate the code...
#[template(path = "start.html")]
pub struct StartTemplate {
    pub title: String,
}

//**************************************************************************
// Useful templates
//
#[derive(Template)] // this will generate the code...
#[template(path = "404.html")]
pub struct NotFoundTemplate {
    pub title: String,
    pub uri: Uri,
}

#[derive(Template)] // this will generate the code...
#[template(path = "about.html")]
pub struct AboutTemplate {
    pub title: String,
}

#[derive(Template)] // this will generate the code...
#[template(path = "debug.html")]
pub struct DebugTemplate {
    pub title: String,
    //pub flash: Option<String>,
    pub cookies: Vec<String>,
    pub str_auth: String,
    pub session_user: String,
    pub session_role: String,
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {:?}", err),
            )
                .into_response(),
        }
    }
}
