#[macro_use]
extern crate serde_derive;


#[derive(Serialize, Deserialize)]
pub struct NewTodo {
    pub description: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Todo {
    pub description: String,
    pub completed: bool,
}
