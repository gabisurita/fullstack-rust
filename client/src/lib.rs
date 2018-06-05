#![recursion_limit = "128"]

extern crate failure;
extern crate strum;
#[macro_use]
extern crate strum_macros;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate yew;
extern crate schema;

use failure::Error;
use schema::{Todo, TodoID};
use strum::IntoEnumIterator;
use yew::callback::Callback;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

/// Todo Resource client.
#[derive(Default)]
pub struct TodoAPI {
    fetch: FetchService,
}

// TODO: Extract CRUD features into a Trait.
impl TodoAPI {
    const URL: &'static str = "http://localhost:8000/todos";

    pub fn new() -> Self {
        Self {
            fetch: FetchService::new(),
        }
    }

    pub fn retrieve(
        &mut self,
        callback: Callback<Result<Vec<Todo>, Error>>,
    ) -> FetchTask {
        let request = Request::get(TodoAPI::URL).body(Nothing).unwrap();

        let handler =
            move |response: Response<Json<Result<Vec<Todo>, Error>>>| {
                let (meta, Json(body)) = response.into_parts();
                if meta.status.is_success() {
                    callback.emit(Ok(body.unwrap()));
                } else {
                    panic!("Failed to send request");
                }
            };

        self.fetch.fetch(request, handler.into())
    }

    pub fn create<'a>(&mut self, todo: &'a Todo) {
        let request = Request::post(TodoAPI::URL)
            .header("Content-Type", "application/json")
            .body(Ok(serde_json::to_string(todo).unwrap()))
            .unwrap();

        let handler = move |_response: Response<Json<Result<Todo, Error>>>| {
            panic!("Failed to send request");
        };

        self.fetch.fetch(request, handler.into());
    }

    pub fn update<'a>(&mut self, id: TodoID, todo: &'a Todo) {
        let url = format!("{}/{}", TodoAPI::URL, id);
        let request = Request::put(url.as_str())
            .header("Content-Type", "application/json")
            .body(Ok(serde_json::to_string(todo).unwrap()))
            .unwrap();

        let handler = move |_response: Response<Json<Result<Todo, Error>>>| {
            panic!("Failed to send request");
        };

        self.fetch.fetch(request, handler.into());
    }

    pub fn delete<'a>(&mut self, id: TodoID) {
        let url = format!("{}/{}", TodoAPI::URL, id);
        let request = Request::delete(url.as_str()).body(Nothing).unwrap();

        let handler = move |_response: Response<Json<Result<Todo, Error>>>| {
            panic!("Failed to send request");
        };

        self.fetch.fetch(request, handler.into());
    }
}

pub struct Context {
    pub todos: TodoAPI,
}

impl Context {
    pub fn new() -> Self {
        Self {
            todos: TodoAPI::new(),
        }
    }
}

pub struct Model {
    entries:          Vec<Entry>,
    filter:           Filter,
    new_description:  String,
    edit_description: String,
    fetch:            Option<FetchTask>,
}

struct Entry {
    data:    Todo,
    editing: bool,
}

pub enum Msg {
    Load(Vec<Todo>),
    Add,
    Edit(TodoID),
    Update(String),
    UpdateEdit(String),
    Remove(TodoID),
    SetFilter(Filter),
    ToggleEdit(TodoID),
    Toggle(TodoID),
    Nope,
}

impl Component<Context> for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, env: &mut Env<Context, Self>) -> Self {
        let callback = env.send_back(|result: Result<Vec<Todo>, Error>| {
            Msg::Load(result.unwrap())
        });

        Model {
            entries:          vec![],
            filter:           Filter::All,
            new_description:  "".into(),
            edit_description: "".into(),
            fetch:            Some(env.todos.retrieve(callback)),
        }
    }

    fn update(
        &mut self,
        msg: Self::Message,
        env: &mut Env<Context, Self>,
    ) -> ShouldRender {
        match msg {
            Msg::Load(todos) => {
                self.entries = todos
                    .into_iter()
                    .map(|todo| Entry {
                        data:    todo,
                        editing: false,
                    })
                    .collect();
            }
            Msg::Add => {
                let entry = Entry {
                    data:    Todo {
                        description: self.new_description.clone(),
                        completed:   false,
                    },
                    editing: false,
                };
                env.todos.create(&entry.data);
                self.entries.push(entry);
                self.new_description = "".to_string();
            }
            Msg::Edit(id) => {
                let edit_description = self.edit_description.clone();
                self.complete_edit(id, edit_description);
                env.todos.update(id, &self.entries[id].data);
                self.edit_description = "".to_string();
            }
            Msg::Update(val) => {
                println!("Input: {}", val);
                self.new_description = val;
            }
            Msg::UpdateEdit(val) => {
                println!("Input: {}", val);
                self.edit_description = val;
            }
            Msg::Remove(id) => {
                self.remove(id);
                env.todos.delete(id);
            }
            Msg::SetFilter(filter) => {
                self.filter = filter;
            }
            Msg::ToggleEdit(id) => {
                self.edit_description =
                    self.entries[id].data.description.clone();
                self.toggle_edit(id);
            }
            Msg::Toggle(id) => {
                self.toggle(id);
                env.todos.update(id, &self.entries[id].data);
            }
            Msg::Nope => {}
        }
        true
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> Html<Context, Self> {
        html! {
            <div class="todomvc-wrapper",>
                <section class="todoapp",>
                    <header class="header",>
                        <h1>{ "remote todos" }</h1>
                        { self.view_input() }
                    </header>
                    <section class="main",>
                        <ul class="todo-list",>
                            { for self.entries.iter()
                                .filter(|e| self.filter.fit(e))
                                .enumerate()
                                .map(view_entry)
                            }
                        </ul>
                    </section>
                    <footer class="footer",>
                        <span class="todo-count",>
                            <strong>{ self.total() }</strong>
                            { " item(s) left" }
                        </span>
                        <ul class="filters",>
                            { for Filter::iter()
                                .map(|flt| self.view_filter(flt))
                            }
                        </ul>
                    </footer>
                </section>
                <footer class="info",>
                    <p>{ "Double-click to edit a todo" }</p>
                    <p>{ "Based on " }
                        <a href="https://github.com/DenisKolodin/yew", target="_blank",>
                            { "Yew todos MVC" }
                        </a>
                    </p>
                </footer>
            </div>
        }
    }
}

impl Model {
    fn view_filter(&self, filter: Filter) -> Html<Context, Model> {
        let flt = filter.clone();
        html! {
            <li>
                <a class=if self.filter == flt { "selected" } else { "not-selected" },
                   href=&flt,
                   onclick=|_| Msg::SetFilter(flt.clone()),>
                    { filter }
                </a>
            </li>
        }
    }

    fn view_input(&self) -> Html<Context, Model> {
        html! {
            <input class="new-todo",
                   placeholder="What needs to be done?",
                   value=&self.new_description,
                   oninput=|e| Msg::Update(e.value),
                   onkeypress=|e| {
                       if e.key() == "Enter" { Msg::Add } else { Msg::Nope }
                   }, />
        }
    }
}

fn view_entry((id, entry): (TodoID, &Entry)) -> Html<Context, Model> {
    html! {
        <li class=if entry.editing == true { "editing" } else { "" },>
            <div class="view",>
                <input class="toggle",
                       type="checkbox",
                       checked=entry.data.completed,
                       onclick=|_| Msg::Toggle(id), />
                <label ondoubleclick=|_| Msg::ToggleEdit(id),>
                    { &entry.data.description }
                </label>
                <button class="destroy", onclick=|_| Msg::Remove(id), />
            </div>
            { view_entry_edit_input((id, &entry)) }
        </li>
    }
}

fn view_entry_edit_input(
    (id, entry): (TodoID, &Entry),
) -> Html<Context, Model> {
    if entry.editing == true {
        html! {
            <input class="edit",
                   type="text",
                   new_description=&entry.data.description,
                   oninput=|e| Msg::UpdateEdit(e.value),
                   onblur=|_| Msg::Edit(id),
                   onkeypress=|e| {
                      if e.key() == "Enter" { Msg::Edit(id) } else { Msg::Nope }
                   }, />
        }
    } else {
        html! { <input type="hidden", /> }
    }
}

#[derive(EnumIter, ToString, Clone, PartialEq, Serialize, Deserialize)]
pub enum Filter {
    All,
    Active,
    Completed,
}

impl<'a> Into<Href> for &'a Filter {
    fn into(self) -> Href {
        match *self {
            Filter::All => "#/".into(),
            Filter::Active => "#/active".into(),
            Filter::Completed => "#/completed".into(),
        }
    }
}

impl Filter {
    fn fit(&self, entry: &Entry) -> bool {
        match *self {
            Filter::All => true,
            Filter::Active => !entry.data.completed,
            Filter::Completed => entry.data.completed,
        }
    }
}

impl Model {
    fn total(&self) -> TodoID {
        self.entries.len()
    }

    fn toggle(&mut self, id: TodoID) {
        let filter = self.filter.clone();
        let mut entries = self
            .entries
            .iter_mut()
            .filter(|e| filter.fit(e))
            .collect::<Vec<_>>();
        let entry = entries.get_mut(id).unwrap();
        entry.data.completed = !entry.data.completed;
    }

    fn toggle_edit(&mut self, id: TodoID) {
        let filter = self.filter.clone();
        let mut entries = self
            .entries
            .iter_mut()
            .filter(|e| filter.fit(e))
            .collect::<Vec<_>>();
        let entry = entries.get_mut(id).unwrap();
        entry.editing = !entry.editing;
    }

    fn complete_edit(&mut self, id: TodoID, val: String) {
        let filter = self.filter.clone();
        let mut entries = self
            .entries
            .iter_mut()
            .filter(|e| filter.fit(e))
            .collect::<Vec<_>>();
        let entry = entries.get_mut(id).unwrap();
        entry.data.description = val;
        entry.editing = !entry.editing;
    }

    fn remove(&mut self, id: TodoID) {
        let id = {
            let filter = self.filter.clone();
            let entries = self
                .entries
                .iter()
                .enumerate()
                .filter(|&(_, e)| filter.fit(e))
                .collect::<Vec<_>>();
            let &(id, _) = entries.get(id).unwrap();
            id
        };
        self.entries.remove(id);
    }
}
