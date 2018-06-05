#![feature(plugin, decl_macro, associated_type_defaults)]
#![plugin(rocket_codegen)]

pub extern crate r2d2;
pub extern crate r2d2_redis;
pub extern crate redis;
pub extern crate rocket;
pub extern crate rocket_contrib;
pub extern crate schema;
pub extern crate serde;
pub extern crate serde_json;

pub mod repository;
pub mod storage;

use std::io;

use redis::Connection;
use rocket_contrib::Json;
use schema::{Todo, TodoID};

use repository::{QueueRepository, RepositoryError};
use rocket::response::NamedFile;
use rocket::Route;
use storage::{init_pool, RedisConnectionWrapper};

impl QueueRepository<Todo> for Connection {
    fn key(&self) -> &str {
        "todos"
    }
}

/// Serves the frontend App index.html.
#[get("/")]
pub fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

/// Serves the frontend Wasm application and CSS styles.
#[get("/<filename>")]
pub fn statics(filename: String) -> io::Result<NamedFile> {
    NamedFile::open(format!("static/{}", filename).as_str())
}

/// List Todos.
#[get("/")]
pub fn list_todos(
    connection: RedisConnectionWrapper,
) -> Result<Json<Vec<Todo>>, RepositoryError> {
    Ok(Json(connection.all()?))
}

/// Creates a new Todo.
#[post("/", data = "<new_todo>")]
pub fn create_todo(
    connection: RedisConnectionWrapper,
    new_todo: Json<Todo>,
) -> Result<Json<Todo>, RepositoryError> {
    connection.push(&new_todo.0)?;
    Ok(Json(new_todo.0))
}

/// Updates the Todo on the given position.
#[put("/<index>", data = "<todo>")]
pub fn update_todo(
    connection: RedisConnectionWrapper,
    index: TodoID,
    todo: Json<Todo>,
) -> Result<Json<Todo>, RepositoryError> {
    connection.replace(index as isize, &todo.0)?;
    Ok(Json(todo.0))
}

/// Deletes the Todo on the given position.
#[delete("/<index>")]
pub fn delete_todo(
    connection: RedisConnectionWrapper,
    index: TodoID,
) -> Result<Json<Todo>, RepositoryError> {
    let data = connection.delete(index as isize)?;
    Ok(Json(data))
}

/// Generate routes for the Todo resource.
pub fn todo_routes() -> Vec<Route> {
    routes![list_todos, create_todo, update_todo, delete_todo,]
}

fn main() {
    let db_pool = init_pool();
    let server = rocket::ignite()
        .mount("/todos", todo_routes())
        .mount("/", routes![index, statics])
        .manage(db_pool);

    server.launch();
}
