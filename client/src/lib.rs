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
use schema::Todo;
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

    pub fn update<'a>(&mut self, id: usize, todo: &'a Todo) {
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

    pub fn delete<'a>(&mut self, id: usize) {
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
    entries:    Vec<Entry>,
    filter:     Filter,
    value:      String,
    edit_value: String,
    fetch:      Option<FetchTask>,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    data:    Todo,
    editing: bool,
}

pub enum Msg {
    Load(Vec<Todo>),
    Add,
    Edit(usize),
    Update(String),
    UpdateEdit(String),
    Remove(usize),
    SetFilter(Filter),
    ToggleEdit(usize),
    Toggle(usize),
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
            entries:    vec![],
            filter:     Filter::All,
            value:      "".into(),
            edit_value: "".into(),
            fetch:      Some(env.todos.retrieve(callback)),
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
                        description: self.value.clone(),
                        completed:   false,
                    },
                    editing: false,
                };
                env.todos.create(&entry.data);
                self.entries.push(entry);
                self.value = "".to_string();
            }
            Msg::Edit(idx) => {
                let edit_value = self.edit_value.clone();
                self.complete_edit(idx, edit_value);
                env.todos.update(idx, &self.entries[idx].data);
                self.edit_value = "".to_string();
            }
            Msg::Update(val) => {
                println!("Input: {}", val);
                self.value = val;
            }
            Msg::UpdateEdit(val) => {
                println!("Input: {}", val);
                self.edit_value = val;
            }
            Msg::Remove(idx) => {
                self.remove(idx);
                env.todos.delete(idx);
            }
            Msg::SetFilter(filter) => {
                self.filter = filter;
            }
            Msg::ToggleEdit(idx) => {
                self.edit_value = self.entries[idx].data.description.clone();
                self.toggle_edit(idx);
            }
            Msg::Toggle(idx) => {
                self.toggle(idx);
                env.todos.update(idx, &self.entries[idx].data);
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
                   value=&self.value,
                   oninput=|e| Msg::Update(e.value),
                   onkeypress=|e| {
                       if e.key() == "Enter" { Msg::Add } else { Msg::Nope }
                   }, />
        }
    }
}

fn view_entry((idx, entry): (usize, &Entry)) -> Html<Context, Model> {
    html! {
        <li class=if entry.editing == true { "editing" } else { "" },>
            <div class="view",>
                <input class="toggle",
                       type="checkbox",
                       checked=entry.data.completed,
                       onclick=|_| Msg::Toggle(idx), />
                <label ondoubleclick=|_| Msg::ToggleEdit(idx),>
                    { &entry.data.description }
                </label>
                <button class="destroy", onclick=|_| Msg::Remove(idx), />
            </div>
            { view_entry_edit_input((idx, &entry)) }
        </li>
    }
}

fn view_entry_edit_input(
    (idx, entry): (usize, &Entry),
) -> Html<Context, Model> {
    if entry.editing == true {
        html! {
            <input class="edit",
                   type="text",
                   value=&entry.data.description,
                   oninput=|e| Msg::UpdateEdit(e.value),
                   onblur=|_| Msg::Edit(idx),
                   onkeypress=|e| {
                      if e.key() == "Enter" { Msg::Edit(idx) } else { Msg::Nope }
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
    fn total(&self) -> usize {
        self.entries.len()
    }

    fn toggle(&mut self, idx: usize) {
        let filter = self.filter.clone();
        let mut entries = self
            .entries
            .iter_mut()
            .filter(|e| filter.fit(e))
            .collect::<Vec<_>>();
        let entry = entries.get_mut(idx).unwrap();
        entry.data.completed = !entry.data.completed;
    }

    fn toggle_edit(&mut self, idx: usize) {
        let filter = self.filter.clone();
        let mut entries = self
            .entries
            .iter_mut()
            .filter(|e| filter.fit(e))
            .collect::<Vec<_>>();
        let entry = entries.get_mut(idx).unwrap();
        entry.editing = !entry.editing;
    }

    fn complete_edit(&mut self, idx: usize, val: String) {
        let filter = self.filter.clone();
        let mut entries = self
            .entries
            .iter_mut()
            .filter(|e| filter.fit(e))
            .collect::<Vec<_>>();
        let entry = entries.get_mut(idx).unwrap();
        entry.data.description = val;
        entry.editing = !entry.editing;
    }

    fn remove(&mut self, idx: usize) {
        let idx = {
            let filter = self.filter.clone();
            let entries = self
                .entries
                .iter()
                .enumerate()
                .filter(|&(_, e)| filter.fit(e))
                .collect::<Vec<_>>();
            let &(idx, _) = entries.get(idx).unwrap();
            idx
        };
        self.entries.remove(idx);
    }
}
