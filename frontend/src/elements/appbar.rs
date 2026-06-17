#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

pub const BUTTON_SELECTED: &str = "border-slate-600";

#[component]
pub fn AppbarImgButton(
    #[props(into)] append_class: Option<String>,
    click_handler: Option<EventHandler<MouseEvent>>,
    disabled: Option<bool>,
    #[props(into)] img: String,
    #[props(into)] label: String,
    #[props(into)] title: Option<String>,
) -> Element {
    let append_class = append_class.unwrap_or_default();
    let disabled = disabled.unwrap_or_default();

    rsx! {
        button {
            class: "flex flex-col h-14 px-3
                justify-end items-center 
                border-b-4 {append_class}",
            disabled: disabled,
            onclick: move |ev| {
                if let Some(callback) = &click_handler {
                    callback.call(ev);
                }
            },
            title: title,
            img {
                class: "w-6 h-6",
                src: "{img}",
            },
            span {
                class: "text-sm",
                "{label}",
            }
        }
    }
}

#[component]
pub fn Appbar(title: String, children: Element) -> Element {
    let local_profile = use_local_profile();
    let mut sidebar = use_sidebar();
    let mut dark_mode = use_context::<Signal<crate::app::DarkMode>>();

    let profile_img_src = local_profile
        .read()
        .image
        .as_ref()
        .map(|url| url.to_string())
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| "/static/icons/uchat.png".to_string());

    rsx! {
        div {
            class: "max-w-[var(--content-max-width)] h-[var(--appbar-height)]
                    fixed top-0 right-0 left-0 mx-auto z-50
                    bg-slate-200 dark:bg-slate-800 transition-colors duration-300",
            div {
                class: "flex flex-row gap-2 sm:gap-4 items-center w-full pr-5 h-full",
                div {
                    class: "cursor-pointer shrink-0",
                    onclick: move |_| sidebar.write().open(),
                    img {
                        class: "profile-portrait shrink-0",
                        src: "{profile_img_src}"
                    },
                },
                div {
                    class: "text-xl font-bold mr-auto dark:text-white truncate hidden sm:block",
                    "{title}",
                }
                button {
                    class: "btn dark:bg-slate-600 dark:text-white hover:bg-slate-500 dark:hover:bg-slate-500 text-xs px-2 py-1 mr-2",
                    onclick: move |_| {
                        let is_dark = !dark_mode.read().0;
                        dark_mode.write().0 = is_dark;
                        if is_dark {
                            let _ = js_sys::eval("document.documentElement.classList.add('dark')");
                        } else {
                            let _ = js_sys::eval("document.documentElement.classList.remove('dark')");
                        }
                    },
                    if dark_mode.read().0 {
                        "☀️ Light"
                    } else {
                        "🌙 Dark"
                    }
                }
                {children}
            }
        }
    }
}
