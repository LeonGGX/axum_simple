//! src/askama.rs

use crate::models::genre::Genre;
use crate::models::musician::Person;
use crate::models::partition::ShowPartition;
use crate::models::user::FilteredUser;
use askama::Template;
use axum::http::Uri;

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html")]
pub struct HelloTemplate {
    pub title: String,
    pub name: String,
}

#[derive(Template)] // this will generate the code...
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub title: String,
    pub error_message: String,
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

#[derive(Template)]
#[template(path = "logout.html")]
pub struct LogoutTemplate {
    pub title: String,
}

#[derive(Template)]
#[template(path = "sign_up.html")]
pub struct SignupTemplate {
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

#[derive(Template)] // this will generate the code...
#[template(path = "list_users.html")]
pub struct ListUsersTemplate {
    pub title: String,
    pub users: Vec<FilteredUser>,
    pub flash: Option<String>,
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
// Template for the start page, starting the application
//
#[derive(Template)] // this will generate the code...
#[template(path = "start.html")]
pub struct StartTemplate {
    pub title: String,
}

#[derive(Template)] // this will generate the code...
#[template(path = "welcome.html")]
pub struct WelcomeTemplate {
    pub title: String,
    pub flash: Option<String>,
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
    pub cookies: Vec<String>,
    pub str_auth: String,
    //pub session_user: Option<String>,
    //pub session_role: Option<String>,
}

#[derive(Template)] // this will generate the code...
#[template(path = "debug_two.html")]
pub struct DebugTemplateTwo {
    pub title: String,
    pub auth_token: Option<String>,
    pub refresh_token: Option<String>,
    pub logged_in: Option<String>,
    //pub str_auth: String,
    //pub session_user: Option<String>,
    //pub session_role: Option<String>,
}
