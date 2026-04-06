#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_router::{Route, Router};
use fermi::{use_init_atom_root, AtomRef}; // Fermi is Dioxus's global state management library

use crate::elements::{
    post::PostManager,
    toaster::{ToastRoot, Toaster},
    Navbar,
};
pub use crate::prelude::*;

pub static TOASTER: AtomRef<Toaster> = |_| Toaster::default();           // Manages toast notifications
pub static POSTMANAGER: AtomRef<PostManager> = |_| PostManager::default(); // Tracks post-related state
pub static LOCAL_PROFILE: AtomRef<LocalProfile> = |_| LocalProfile::default(); // Stores the logged-in user's profile
pub static SIDEBAR: AtomRef<SidebarManager> = |_| SidebarManager::default(); // Controls sidebar visibility/state

/// Init component — runs on app startup to fetch the current user's profile.
/// If the profile fetch fails (e.g. not logged in), it redirects to the login page.
pub fn Init(cx: Scope) -> Element {
    let api_client = ApiClient::global();       // Get the shared API client instance
    let router = use_router(cx);               // Access the router for navigation
    let toaster = use_toaster(cx);             // Access the global toast notification system
    let local_profile = use_local_profile(cx); // Access the global local profile state

    // Async task that runs once on component mount to fetch the user's profile
    let _fetch_local_profile = {
        to_owned![api_client, toaster, router, local_profile]; // Clone these into the async closure
        use_future(cx, (), |_| async move {
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
                    router.navigate_to(page::ACCOUNT_LOGIN)
                }
            }
        })
    };

    None 
}

/// App component — the root of the application.
/// Sets up global state, routing, layout, and top-level UI elements.
pub fn App(cx: Scope) -> Element {
    use_init_atom_root(cx); // Initialize Fermi's global atom store (must be called at the root)

    let _api_client = ApiClient::global(); // Initialize the global API client

    let toaster = use_toaster(cx); // Get the global toaster for passing to ToastRoot

    cx.render(rsx! {
        Router { // Top-level router that handles all page navigation
            Init {}, // Run initialization logic (profile fetch + auth check)
            Sidebar {}, // Render the sidebar (always visible across pages)

            // Main content area with responsive layout constraints via CSS variables
            main {
                class: "max-w-[var(--content-max-width)]
                min-w-[var(--content-min-width)]
                mt-[var(--appbar-height)]
                mb-[var(--navbar-height)]
                mx-auto
                p-4",

                // Each Route maps a URL path to its corresponding page component

                Route { to: page::ACCOUNT_REGISTER, page::Register {} },  // /register
                Route { to: page::ACCOUNT_LOGIN, page::Login {} },         // /login
                Route { to: page::HOME, page::Home {} },                   // /home
                Route { to: page::HOME_BOOKMARKED, page::HomeBookmarked {} }, // /home/bookmarked
                Route { to: page::HOME_LIKED, page::HomeLiked {} },        // /home/liked
                Route { to: page::POST_NEW_CHAT, page::NewChat {} },       // New chat post
                Route { to: page::POST_NEW_IMAGE, page::NewImage {} },     // New image post
                Route { to: page::POST_NEW_POLL, page::NewPoll {} },       // New poll post
                Route { to: page::POSTS_TRENDING, page::Trending {} },     // Trending posts feed
                Route { to: page::PROFILE_EDIT, page::EditProfile {} },    // Edit user profile
                Route { to: page::PROFILE_VIEW, page::ViewProfile {} },    // View a user's profile
            }

            ToastRoot { toaster: toaster }, // Renders active toast notifications on screen
            Navbar {}                       // Bottom navigation bar
        }
    })
}
