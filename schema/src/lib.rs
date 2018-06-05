#[macro_use]
extern crate serde_derive;

pub type TodoID = usize;

#[derive(Serialize, Deserialize, Clone)]
pub struct Todo {
    pub description: String,
    pub completed:   bool,
}
