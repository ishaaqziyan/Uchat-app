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
            class: "flex flex-col w-10 h-14
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

    let profile_img_src = local_profile.read()
        .image
        .clone()
        .map(|url| url.to_string())
        .unwrap_or_default();

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
                    },
                },
                div {
                    class: "text-xl font-bold mr-auto",
                    "{title}",
                }
                {children}
            }
        }
    }
}
