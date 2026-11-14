//! ACORN's Dioxus browser composition root
extern crate alloc;
use dioxus::prelude::*;

mod details;
mod geospatial_page;
mod results;
mod skill;
mod syntax;
mod ui;
mod workspace;
fn app() -> Element {
    use_context_provider(|| Signal::new(String::new()));
    use_context_provider(|| Signal::new(acorn_web::session::Workspace::default()));
    use_context_provider(|| Signal::new(None::<details::Detail>));
    use_context_provider(|| Signal::new(geospatial_page::MapPins::default()));
    rsx! {
        document::Link { rel: "icon", r#type: "image/svg+xml", href: asset!("/assets/generated/acorn-app-icon.svg") }
        document::Link { rel: "license", title: "Lucide icon license", href: asset!("/assets/vendor/lucide.LICENSE") }
        document::Link { rel: "stylesheet", href: asset!("/assets/app.css") }
        document::Script { src: asset!("/assets/effects.js") }
        document::Script { src: asset!("/assets/details.js") }
        document::Script { src: asset!("/assets/remote.js") }
        document::Script { src: asset!("/assets/syntax.js"), "data-prism": asset!("/assets/vendor/prism.js") }
        Router::<ui::Route> {}
    }
}
fn main() {
    dioxus::launch(app);
}
