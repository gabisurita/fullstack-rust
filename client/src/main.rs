extern crate fullstack_demo_client;
extern crate yew;

use fullstack_demo_client::{Context, Model};
use yew::prelude::*;

fn main() {
    yew::initialize();
    let app: App<_, Model> = App::new(Context::new());
    app.mount_to_body();
    yew::run_loop();
}
