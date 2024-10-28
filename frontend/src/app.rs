#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::prelude::*; // Use prelude for correct imports
use fermi::{use_init_atom_root, AtomRef};
use crate::elements::{
    post::PostManager,
    toaster::{ToastRoot, Toaster},
    Navbar,
};
use crate::page::{Register, Login, Home, HomeBookmarked, HomeLiked, NewChat, NewImage, NewPoll, Trending, EditProfile, ViewProfile}; // Ensure correct component imports
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

#[derive(Routable, Clone)]
pub enum Route {
    #[route("/account/register")]
    AccountRegister,
    #[route("/account/login")]
    AccountLogin,
    #[route("/")]
    Home,
    #[route("/home/bookmarked")]
    HomeBookmarked,
    #[route("/home/liked")]
    HomeLiked,
    #[route("/post/new/chat")]
    NewChat,
    #[route("/post/new/image")]
    NewImage,
    #[route("/post/new/poll")]
    NewPoll,
    #[route("/posts/trending")]
    Trending,
    #[route("/profile/edit")]
    EditProfile,
    #[route("/profile/view")]
    ViewProfile,
}

pub fn App(cx: Scope) -> Element {
    use_init_atom_root(cx.scope_state());
    let _api_client = ApiClient::global();
    let toaster = use_toaster(cx);

    cx.render(rsx! {
        Router {
            Route::routes(), // Ensure routes method is correctly used
            Init {},
            Sidebar {},
            main {
                class: "max-w-[var(--content-max-width)]
                min-w-[var(--content-min-width)]
                mt-[var(--appbar-height)]
                mb-[var(--navbar-height)]
                mx-auto
                p-4",
                Route::AccountRegister {},
                Route::AccountLogin {},
                Route::Home {},
                Route::HomeBookmarked {},
                Route::HomeLiked {},
                Route::NewChat {},
                Route::NewImage {},
                Route::NewPoll {},
                Route::Trending {},
                Route::EditProfile {},
                Route::ViewProfile {},
            }
            ToastRoot { toaster: toaster },
            Navbar {}
        }
    })
}
