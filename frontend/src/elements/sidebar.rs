#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

pub fn use_sidebar() -> Signal<SidebarManager> {
    *crate::app::SIDEBAR
}

#[derive(Default, Clone)]
pub struct SidebarManager {
    is_open: bool,
}

impl SidebarManager {
    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    pub fn is_open(&self) -> bool {
        self.is_open
    }
}

#[component]
pub fn Sidebar() -> Element {
    let sidebar = use_sidebar();
    let nav = use_navigator();
    let local_profile = use_local_profile();

    let sidebar_width = if sidebar.read().is_open() {
        "w-[var(--sidebar-width)]"
    } else {
        "w-0"
    };

    let overlay_class = if sidebar.read().is_open() {
        "w-full opacity-80"
    } else {
        "w-0 opacity-0"
    };

    let read_local_profile = local_profile.read();
    let profile_img_src = read_local_profile
        .image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or("");

    rsx! {
        div {
            class: "fixed top-0 left-0 h-full navbar-bg-color transition z-[99] {overlay_class}",
            onclick: move |_| sidebar.write().close(),
        }
        div {
            class: "{sidebar_width} z-[100] fixed top-0 left-0 h-full
            overflow-x-hidden
            flex flex-col
            navbar-bg-color transition-[width] duration-300",
            a {
                class: "flex flex-row justify-center py-5 cursor-pointer",
                onclick: move |_| {
                    sidebar.write().close();
                    if let Some(id) = local_profile.read().user_id {
                        let url = crate::page::profile_view(id);
                        nav.push(url);
                    }
                },
                img {
                    class: "profile-portrait-lg",
                    src: "{profile_img_src}",
                }
            }
            a {
                class: "sidebar-navlink border-t",
                onclick: move |_| {
                    sidebar.write().close();
                    nav.push(page::PROFILE_EDIT);
                },
                "Edit Profile"
            }
            a {
                class: "sidebar-navlink mb-auto",
                onclick: move |_| {
                    sidebar.write().close();
                    nav.push(page::HOME_BOOKMARKED);
                },
                "Bookmarks"
            }
            a {
                class: "sidebar-navlink",
                onclick: move |_| {
                    use chrono::Utc;
                    use uchat_domain::ids::SessionId;
                    crate::util::cookie::set_session("".to_string(), SessionId::new(), Utc::now());
                    local_profile.write().user_id = None;
                    local_profile.write().image = None;
                    sidebar.write().close();
                    nav.push(page::ACCOUNT_LOGIN);
                },
                "Logout"
            }
        }
    }
}
