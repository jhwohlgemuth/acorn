//! ACORN's Dioxus browser composition root
extern crate alloc;
mod geospatial_page;
mod results;
mod skill;
mod ui;
mod workspace;
use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
}
fn app() -> Element {
    use_context_provider(|| Signal::new(String::new()));
    use_context_provider(|| Signal::new(acorn_web::session::Workspace::default()));
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/app.css") }
        document::Script { src: asset!("/assets/effects.js") }
        Router::<ui::Route> {}
    }
}
