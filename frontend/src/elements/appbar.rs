#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

pub const BUTTON_SELECTED: &str = "border-slate-600";

#[component]
pub fn AppbarImgButton(
    append_class: Option<String>,
    click_handler: Option<EventHandler<MouseEvent>>,
    disabled: Option<bool>,
    img: String,
    label: String,
    title: Option<String>,
) -> Element {
    let append_class = append_class.as_deref().unwrap_or("");

    rsx! {
        button {
            class: "flex flex-col w-10 h-14
                justify-end items-center 
                border-b-4 {append_class}",
            disabled: disabled.unwrap_or_default(),
            onclick: move |ev| {
                if let Some(ref handler) = click_handler {
                    handler.call(ev);
                }
            },
            title: title,
            img {
                class: "w-6 h-6",
                src: "{img}",
            }
            span {
                class: "text-sm",
                "{label}",
            }
        }
    }
}

#[component]
pub fn Appbar(
    title: String,
    children: Element,
) -> Element {
    let local_profile = use_local_profile();
    let sidebar = use_sidebar();

    let local_profile = local_profile.read();
    let profile_img_src = local_profile
        .image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or("");

    rsx! {
        div {
            class: "max-w-[var(--content-max-width)] h-[var(--appbar-height)]
                    fixed top-0 right-0 left-0 mx-auto z-50
                    bg-slate-200",
            div {
                class: "flex flex-row gap-8 items-center w-full pr-5 h-full",
                div {
                    class: "cursor-pointer",
                    onclick: move |_| sidebar.write().open(),
                    img {
                        class: "profile-portrait",
                        src: "{profile_img_src}"
                    }
                }
                div {
                    class: "text-xl font-bold mr-auto",
                    "{title}",
                }
                {children}
            }
        }
    }
}
