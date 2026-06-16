#![allow(non_snake_case)]

use dioxus::prelude::*;


use crate::elements::{
    post::PostManager,
    toaster::{ToastRoot, Toaster},
    Navbar,
};
pub use crate::prelude::*;
use crate::page::{Register, Login, Home, HomeBookmarked, HomeLiked, NewChat};
use crate::page::{NewImage, NewPoll, Trending, EditProfile, ViewProfile};
use crate::elements::sidebar::{Sidebar, SidebarManager};
use crate::elements::local_profile::LocalProfile;
// Global state is now managed via Dioxus use_context_provider

/// Init component — runs on app startup to fetch the current user's profile.
/// If the profile fetch fails (e.g. not logged in), it redirects to the login page.
#[component]
pub
fn Init() -> Element {
    let api_client = ApiClient::global();       // Get the shared API client instance
    let router = use_navigator();               // Access the router for navigation
    let toaster = use_toaster();             // Access the global toast notification system
    let local_profile = use_local_profile(); // Access the global local profile state

    // Async task that runs once on component mount to fetch the user's profile
    let _fetch_local_profile = {
        use_future(move || {
            let api_client = api_client.clone();
            let mut toaster = toaster.clone();
            let router = router.clone();
            let mut local_profile = local_profile.clone();
            async move {
            use uchat_endpoint::user::endpoint::{GetMyProfile, GetMyProfileOk};

            // Make an API call to fetch the current user's profile
            let response = fetch_json!(<GetMyProfileOk>, api_client, GetMyProfile);

            match response {
                Ok(res) => {
                    // On success, update global profile state with fetched data
                    local_profile.write().image = res.profile_image;
                    local_profile.write().user_id = Some(res.user_id);
                }
                Err(_e) => {
                    // On failure, show an error toast and redirect to the login page
                    toaster.write().error(
                        "Please log in or create an account to continue.",
                        chrono::Duration::seconds(5),
                    );
                    { router.push(Route::Login {}); }
                }
            }
            }
        })
    };

    rsx! {}
}

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AuthLayout)]
        #[route("/account/register")]
        Register {},
        #[route("/account/login")]
        Login {},
        
    #[layout(AppLayout)]
        #[route("/home")]
        Home {},
        #[route("/home/bookmarked")]
        HomeBookmarked {},
        #[route("/home/liked")]
        HomeLiked {},
        #[route("/post/new_chat")]
        NewChat {},
        #[route("/post/new_image")]
        NewImage {},
        #[route("/post/new_poll")]
        NewPoll {},
        #[route("/posts/trending")]
        Trending {},
        #[route("/profile/edit")]
        EditProfile {},
        #[route("/profile/view/:user_id")]
        ViewProfile { user_id: uchat_domain::ids::UserId },
}

#[component]
pub fn AppLayout() -> Element {
    let local_profile = crate::elements::local_profile::use_local_profile();
    let is_logged_in = local_profile.read().user_id.is_some();

    rsx! {
        Init {}
        if is_logged_in {
            Sidebar {}
        }
        main {
            class: "max-w-[var(--content-max-width)]
            min-w-[var(--content-min-width)]
            mt-[var(--appbar-height)]
            mb-[var(--navbar-height)]
            mx-auto
            p-4",
            Outlet::<Route> {}
        }
        if is_logged_in {
            Navbar {}
        }
    }
}

#[component]
pub fn AuthLayout() -> Element {
    rsx! {
        main {
            class: "max-w-[var(--content-max-width)]
            min-w-[var(--content-min-width)]
            mx-auto
            p-4",
            Outlet::<Route> {}
        }
    }
}

pub fn App() -> Element {
    let toaster = Signal::new(Toaster::default());
    use_context_provider(|| toaster.clone());
    use_context_provider(|| Signal::new(PostManager::default()));
    use_context_provider(|| Signal::new(LocalProfile::default()));
    use_context_provider(|| Signal::new(SidebarManager::default()));

    let _api_client = ApiClient::global();

    rsx! {
        Router::<Route> {}
        ToastRoot { toaster: toaster }
    }
}
