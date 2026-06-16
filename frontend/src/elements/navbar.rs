#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

#[component]
pub fn NewPostPopup(hide: Signal<bool>) -> Element {
    let router = use_navigator();

    let hide_class = maybe_class!("hidden", *hide.read());

    const BUTTON_CLASS: &str = "grid grid-cols-[20px_1fr] gap-4 pl-4
                                justify-center items-center
                                w-full h-12
                                border-y navbar-border-color";
    rsx! {
        div {
            class: "flex flex-col
                    absolute right-0 bottom-[var(--navbar-height)]
                    w-28 items-center {hide_class}
                    navbar-bg-color text-white text-sm",
            div {
                class: BUTTON_CLASS,
                onclick: move |_| {
                    let _ = router.push(crate::app::Route::NewPoll {});
                    hide.set(true);
                },
                img {
                    class: "invert",
                    src: "/static/icons/icon-poll.svg",
                },
                "Poll"
            }
            div {
                class: BUTTON_CLASS,
                onclick: move |_| {
                    let _ = router.push(crate::app::Route::NewImage {});
                    hide.set(true);
                },

                img {
                    class: "invert",
                    src: "/static/icons/icon-image.svg",
                },
                "Image"
            }
            div {
                class: BUTTON_CLASS,
                onclick: move |_| {
                    let _ = router.push(crate::app::Route::NewChat {});
                    hide.set(true);
                },
                img {
                    class: "invert",
                    src: "/static/icons/icon-messages.svg",
                },
                "Chat"
            }
        }
    }
}

#[component]
pub fn NavButton(
    img: String,
    label: String,
    onclick: EventHandler<MouseEvent>,
    highlight: Option<bool>,
    children: Element,
) -> Element {
    let selected_bgcolor = maybe_class!("bg-slate-500", matches!(highlight, Some(true)));

    rsx! {
        button {
            class: "cursor-pointer flex flex-col items-center justify-center h-full {selected_bgcolor}",
            onclick: move |ev| onclick.call(ev),
            img {
                class: "invert",
                src: "{img}",
                width: "25px",
                height: "25px",
            },
            div {
                class: "text-sm text-white",
                "{label}"
            },
            {children}
        }
    }
}

#[component]
pub fn Navbar() -> Element {
    let mut hide_new_post_popup = use_signal(|| true);
    let router = use_navigator();
    let route = use_route::<crate::app::Route>();

    let hide_navbar = use_signal(|| false);

    let current_route = route.to_string();

    use_effect(move || {
        let mut hide_navbar = hide_navbar.clone();
        let current_route = current_route.clone();
        spawn(async move {
            let should_hide =
                current_route == page::ACCOUNT_LOGIN || current_route == page::ACCOUNT_REGISTER;
            hide_navbar.set(should_hide);
        });
    });

    if *hide_navbar.read() {
        return rsx! {};
    }

    rsx! {
        nav {
            class: "max-w-[var(--content-max-width)] h-[var(--navbar-height)]
                fixed bottom-0 left-0 right-0 mx-auto
                border-t navbar-bg-color navbar-border-color",
            div {
                class: "grid grid-cols-3 justify-around w-full h-full items-center shadow-inner",
                NavButton {
                    img: "/static/icons/icon-home.svg",
                    label: "Home",
                    // onclick: |_| (),
                    onclick: move |_| { let _ = router.push(crate::app::Route::Home {}); },
                },
                NavButton {
                    img: "/static/icons/icon-trending.svg",
                    label: "Trending",
                    onclick: move |_| { let _ = router.push(crate::app::Route::Trending {}); },
                }
                NavButton {
                    img: "/static/icons/icon-new-post.svg",
                    label: "Post",
                    onclick: move |_| {
                        let is_hidden = *hide_new_post_popup.read();
                        hide_new_post_popup.set(!is_hidden);
                    },
                    NewPostPopup { hide: hide_new_post_popup.clone() }
                }
            }
        }
    }
}
