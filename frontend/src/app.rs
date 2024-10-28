#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*; // Use prelude for correct imports
use fermi::{use_init_atom_root, AtomRef};
use crate::elements::{
    post::PostManager,
    toaster::{ToastRoot, Toaster},
    Navbar,
};
pub use crate::prelude::*;

pub static TOASTER: AtomRef<Toaster> = |_| Toaster::default();
pub static POSTMANAGER: AtomRef<PostManager> = |_| PostManager::default();
pub static LOCAL_PROFILE: AtomRef<LocalProfile> = |_| LocalProfile::default();
pub static SIDEBAR: AtomRef<SidebarManager> = |_| SidebarManager::default();

pub fn Init(cx: Scope) -> Element {
    let api_client = ApiClient::global();
    let navigator = use_navigator(cx).unwrap();
    let toaster = use_toaster(cx);
    let local_profile = use_local_profile(cx);

    let _fetch_local_profile = {
        to_owned![api_client, toaster, navigator, local_profile];
        use_future(cx, (), |_| async move {
            use uchat_endpoint::user::endpoint::{GetMyProfile, GetMyProfileOk};
            let response = fetch_json!(<GetMyProfileOk>, api_client, GetMyProfile);
            match response {
                Ok(res) => {
                    local_profile.write().image = res.profile_image;
                    local_profile.write().user_id = Some(res.user_id);
                }
                Err(_e) => {
                    toaster.write().error(
                        "Please log in or create an account to continue.",
                        chrono::Duration::seconds(5),
                    );
                    navigator.push("/account/login");
                }
            }
        })
    };

    None
}

pub fn App(cx: Scope) -> Element {
    use_init_atom_root(cx.scope_state());
    let _api_client = ApiClient::global();
    let toaster = use_toaster(cx);

    cx.render(rsx! {
        Router {
            Init {},
            Sidebar {},
            main {
                class: "max-w-[var(--content-max-width)]
                min-w-[var(--content-min-width)]
                mt-[var(--appbar-height)]
                mb-[var(--navbar-height)]
                mx-auto
                p-4",
                Route { to: "/account/register", component: page::Register {} },
                Route { to: "/account/login", component: page::Login {} },
                Route { to: "/", component: page::Home {} },
                Route { to: "/home/bookmarked", component: page::HomeBookmarked {} },
                Route { to: "/home/liked", component: page::HomeLiked {} },
                Route { to: "/post/new/chat", component: page::NewChat {} },
                Route { to: "/post/new/image", component: page::NewImage {} },
                Route { to: "/post/new/poll", component: page::NewPoll {} },
                Route { to: "/posts/trending", component: page::Trending {} },
                Route { to: "/profile/edit", component: page::EditProfile {} },
                Route { to: "/profile/view", component: page::ViewProfile {} },
            }
            ToastRoot { toaster: toaster },
            Navbar {}
        }
    })
}
