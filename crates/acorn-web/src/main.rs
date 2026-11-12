//! ACORN's Dioxus browser composition root
mod ui;
use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
}
fn app() -> Element {
    use_context_provider(|| Signal::new(String::new()));
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/app.css") }
        document::Script { src: asset!("/assets/effects.js") }
        Router::<ui::Route> {}
    }
}
