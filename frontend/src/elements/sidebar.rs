#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;

pub fn use_sidebar() -> Signal<SidebarManager> {
    use_context::<Signal<SidebarManager>>()
}

#[derive(Default)]
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
    let mut sidebar = use_sidebar();
    let router = use_navigator();
    let mut local_profile = use_local_profile();
    let api_client = ApiClient::global();

    use_future(move || {
        let mut local_profile = local_profile.clone();
        let api_client = api_client.clone();
        async move {
            loop {
                gloo_timers::future::sleep(std::time::Duration::from_secs(5)).await;
                if local_profile.read().user_id.is_some() {
                    if let Ok(res) = fetch_json!(<uchat_endpoint::user::endpoint::GetMyProfileOk>, api_client, uchat_endpoint::user::endpoint::GetMyProfile)
                    {
                        if local_profile.read().unread_notifications != res.unread_notifications {
                            local_profile.write().unread_notifications = res.unread_notifications;
                        }
                    }
                }
            }
        }
    });

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

    let Overlay = rsx! {
        div {
            class: "fixed top-0 left-0 h-full navbar-bg-color transition z-[99] {overlay_class}",
            onclick: move |_| sidebar.write().close(),
        }
    };

    let read_local_profile = local_profile.read();
    let profile_img_src = read_local_profile
        .image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or_else(|| "/static/icons/uchat.png");

    rsx! {
        {Overlay}
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
                        let route = crate::app::Route::ViewProfile { user_id: id };
                        let _ = router.push(route);
                    }
                },
                img {
                    class: "profile-portrait-lg",
                    src: "{profile_img_src}",
                }
            },
            a {
                class: "sidebar-navlink border-t",
                onclick: move |_| {
                    sidebar.write().close();
                    let _ = router.push(crate::app::Route::EditProfile {});
                },
                "Edit Profile"
            }
            a {
                class: "sidebar-navlink",
                onclick: move |_| {
                    sidebar.write().close();
                    let _ = router.push(crate::app::Route::HomeBookmarked {});
                },
                "Bookmarks"
            },
            a {
                class: "sidebar-navlink flex flex-row items-center gap-2",
                onclick: move |_| {
                    sidebar.write().close();
                    let _ = router.push(crate::app::Route::Notifications {});
                },
                "Notifications"
                if local_profile.read().unread_notifications > 0 {
                    span {
                        class: "bg-red-500 text-white text-xs font-bold px-2 py-1 rounded-full",
                        "{local_profile.read().unread_notifications}"
                    }
                }
            },
            div {
                class: "sidebar-navlink mb-auto",
                div { class: "font-semibold mb-2", "Background Theme" }
                div {
                    class: "flex flex-col gap-2",
                    button {
                        class: "btn text-xs",
                        onclick: move |_| {
                            let _ = js_sys::eval("document.getElementById('app-bg').className = 'mesh-bg-1'");
                            let _ = js_sys::eval("localStorage.setItem('bg_theme', 'mesh-bg-1')");
                        },
                        "Vibrant"
                    }
                    button {
                        class: "btn text-xs",
                        onclick: move |_| {
                            let _ = js_sys::eval("document.getElementById('app-bg').className = 'mesh-bg-2'");
                            let _ = js_sys::eval("localStorage.setItem('bg_theme', 'mesh-bg-2')");
                        },
                        "Sunset"
                    }
                    button {
                        class: "btn text-xs",
                        onclick: move |_| {
                            let _ = js_sys::eval("document.getElementById('app-bg').className = 'mesh-bg-3'");
                            let _ = js_sys::eval("localStorage.setItem('bg_theme', 'mesh-bg-3')");
                        },
                        "Ocean"
                    }
                }
            }
            a {
                class: "sidebar-navlink",
                onclick: move |_| {
                    sidebar.write().close();
                    let _ = router.push(crate::app::Route::Conversations {});
                },
                "Messages"
            },
            a {
                class: "sidebar-navlink",
                onclick: move |_| {
                    use chrono::Utc;
                    use uchat_domain::ids::SessionId;
                    crate::util::cookie::set_session("".to_string(), SessionId::new(), Utc::now());
                    local_profile.write().user_id = None;
                    local_profile.write().image = None;
                    sidebar.write().close();
                    let _ = router.push(crate::app::Route::Login {});
                },
                "Logout"
            }
        }
    }
}
