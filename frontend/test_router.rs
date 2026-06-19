use dioxus::prelude::*;

#[derive(Routable, Clone, PartialEq)]
enum Route {
    #[route("/home")]
    Home {},

    #[redirect("/", || Route::Home {})]
    #[route("/:..route")]
    PageNotFound { route: Vec<String> },
}

#[component]
fn Home() -> Element { rsx! { "Home" } }

#[component]
fn PageNotFound(route: Vec<String>) -> Element { rsx! { "404" } }

fn main() {}
