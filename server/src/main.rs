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

use std::{env, io};
use std::ops::Deref;

use r2d2::{Pool, PooledConnection};
use r2d2_redis::RedisConnectionManager;
use redis::{Commands, RedisResult};
use rocket::{http, request, Outcome, Route, State, };
use rocket::response::NamedFile;


/// Redis connection pool.
pub type RedisPool = Pool<RedisConnectionManager>;

/// Creates a new Redis connection pool.
pub fn init_pool() -> RedisPool {
    let database_url =
        env::var("DATABASE_URL").unwrap_or("redis://localhost".to_string());
    let manager = RedisConnectionManager::new(database_url.as_str()).unwrap();

    Pool::builder().build(manager).unwrap()
}

/// Redis connection wrapper.
pub type RedisConnection = PooledConnection<RedisConnectionManager>;

pub struct RedisConnectionWrapper(RedisConnection);

/// Allow access to the inner connection type using Deref.
impl Deref for RedisConnectionWrapper {
    type Target = RedisConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Allow controller access to a RedisConnection.
impl<'a, 'r> request::FromRequest<'a, 'r> for RedisConnectionWrapper {
    type Error = ();

    fn from_request(request: &'a request::Request<'r>)
                    -> request::Outcome<RedisConnectionWrapper, ()> {
        let pool = request.guard::<State<RedisPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(RedisConnectionWrapper(conn)),
            Err(_) => Outcome::Failure((http::Status::ServiceUnavailable, ())),
        }
    }
}

use rocket_contrib::Json;
use schema::Todo;

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
    let deserialized =
        result.unwrap()
              .iter()
              .map(|row| serde_json::from_str(row.as_str()).unwrap())
              .collect();

    Json(deserialized)
}

/// Creates a new Todo.
#[post("/", data = "<new_todo>")]
pub fn create_todo(connection: RedisConnectionWrapper,
                   new_todo: Json<Todo>)
                   -> Json<Todo> {
    let index: RedisResult<i32> =
        connection.rpush("todos", serde_json::to_string(&new_todo.0).unwrap());

    // FIXME: Handle Error
    index.unwrap();
    Json(new_todo.0)
}

/// Updates the Todo on the given position.
#[put("/<index>", data = "<todo>")]
pub fn update_todo(connection: RedisConnectionWrapper,
                   index: isize,
                   todo: Json<Todo>)
                   -> Json<Todo> {
    let result: RedisResult<String> =
        connection.lset("todos",
                        index,
                        serde_json::to_string(&todo.0).unwrap());

    // FIXME: Handle Error
    result.unwrap();
    Json(todo.0)
}

/// Deletes the Todo on the given position.
#[delete("/<index>")]
pub fn delete_todo(connection: RedisConnectionWrapper,
                   index: isize)
                   -> Json<Todo> {
    let value: RedisResult<String> = connection.lindex("todos", index);

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
    let server = rocket::ignite().mount("/todos", todo_routes()).mount("/", routes![index, statics]).manage(db_pool);

    server.launch();
}
