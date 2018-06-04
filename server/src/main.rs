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

pub mod storage;

use std::io;

use redis::Commands;
use redis::RedisResult;
use rocket_contrib::Json;
use schema::{Todo, TodoID};

use rocket::response::NamedFile;
use rocket::Route;
use storage::{init_pool, RedisConnectionWrapper};

/// Serves the frontend App.
#[get("/")]
pub fn index() -> io::Result<NamedFile> {
    NamedFile::open("static/index.html")
}

/// Serves the frontend App.
#[get("/<filename>")]
pub fn statics(filename: String) -> io::Result<NamedFile> {
    NamedFile::open(format!("static/{}", filename).as_str())
}

/// List Todos.
#[get("/")]
pub fn list_todos(connection: RedisConnectionWrapper) -> Json<Vec<Todo>> {
    let result: RedisResult<Vec<String>> = connection.lrange("todos", 0, -1);
    let deserialized = result
        .unwrap()
        .iter()
        .map(|row| serde_json::from_str(row.as_str()).unwrap())
        .collect();

    Json(deserialized)
}

/// Creates a new Todo.
#[post("/", data = "<new_todo>")]
pub fn create_todo(
    connection: RedisConnectionWrapper,
    new_todo: Json<Todo>,
) -> Json<Todo> {
    let index: RedisResult<i32> =
        connection.rpush("todos", serde_json::to_string(&new_todo.0).unwrap());

    // FIXME: Handle Error
    index.unwrap();
    Json(new_todo.0)
}

/// Updates the Todo on the given position.
#[put("/<index>", data = "<todo>")]
pub fn update_todo(
    connection: RedisConnectionWrapper,
    index: TodoID,
    todo: Json<Todo>,
) -> Json<Todo> {
    let result: RedisResult<String> = connection.lset(
        "todos",
        index as isize,
        serde_json::to_string(&todo.0).unwrap(),
    );

    // FIXME: Handle Error
    result.unwrap();
    Json(todo.0)
}

/// Deletes the Todo on the given position.
#[delete("/<index>")]
pub fn delete_todo(
    connection: RedisConnectionWrapper,
    index: TodoID,
) -> Json<Todo> {
    let value: RedisResult<String> = connection.lindex("todos", index as isize);

    let data = value.unwrap();

    let result: RedisResult<i32> = connection.lrem("todos", 1, &data);

    // FIXME: Handle Error
    result.unwrap();
    Json(serde_json::from_str(&data).unwrap())
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
