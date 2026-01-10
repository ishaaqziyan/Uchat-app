#![allow(non_snake_case)]

use crate::elements::{
    post::PostManager,
    toaster::{ToastRoot, Toaster},
    Navbar,
};
pub use crate::prelude::*;

pub static TOASTER: GlobalSignal<Toaster> = Signal::global(|| Toaster::default());
pub static POSTMANAGER: GlobalSignal<PostManager> = Signal::global(|| PostManager::default());
pub static LOCAL_PROFILE: GlobalSignal<LocalProfile> = Signal::global(|| LocalProfile::default());
pub static SIDEBAR: GlobalSignal<SidebarManager> = Signal::global(|| SidebarManager::default());

#[component]
pub fn Init() -> Element {
    let api_client = ApiClient::global();
    let nav = use_navigator();
    let mut toaster = use_toaster();
    let mut local_profile = use_local_profile();

    use_future(move || async move {
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
                let _ = nav.push(page::ACCOUNT_LOGIN);
            }
        }
    });
    
    rsx! {}
}

// Define your router enum with component mappings
#[derive(Clone, Routable, Debug, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Layout)]
        #[route("/")]
        Home {},
        
        #[route("/home/bookmarked")]
        HomeBookmarked {},
        
        #[route("/home/liked")]
        HomeLiked {},
        
        #[route("/post/new/chat")]
        PostNewChat {},
        
        #[route("/post/new/image")]
        PostNewImage {},
        
        #[route("/post/new/poll")]
        PostNewPoll {},
        
        #[route("/posts/trending")]
        PostsTrending {},
        
        #[route("/profile/edit")]
        ProfileEdit {},
        
        #[route("/profile/:id")]
        ProfileView { id: String },
    #[end_layout]
    
    #[route("/account/register")]
    AccountRegister {},
    
    #[route("/account/login")]
    AccountLogin {},
}

// Implement component mapping for each route
#[component]
fn Home() -> Element {
    rsx! { crate::page::Home {} }
}

#[component]
fn HomeBookmarked() -> Element {
    rsx! { crate::page::HomeBookmarked {} }
}

#[component]
fn HomeLiked() -> Element {
    rsx! { crate::page::HomeLiked {} }
}

#[component]
fn PostNewChat() -> Element {
    rsx! { crate::page::NewChat {} }
}

#[component]
fn PostNewImage() -> Element {
    rsx! { crate::page::NewImage {} }
}

#[component]
fn PostNewPoll() -> Element {
    rsx! { crate::page::NewPoll {} }
}

#[component]
fn PostsTrending() -> Element {
    rsx! { crate::page::Trending {} }
}

#[component]
fn ProfileEdit() -> Element {
    rsx! { crate::page::EditProfile {} }
}

#[component]
fn ProfileView(id: String) -> Element {
    use std::str::FromStr;
    use uchat_domain::ids::UserId;
    
    let user_id = UserId::from_str(&id).unwrap_or_default();
    
    rsx! { 
        crate::page::ViewProfile { 
            user_id: user_id 
        } 
    }
}

#[component]
fn AccountRegister() -> Element {
    rsx! { crate::page::Register {} }
}

#[component]
fn AccountLogin() -> Element {
    rsx! { crate::page::Login {} }
}

#[component]
fn Layout() -> Element {
    rsx! {
        Init {}
        Sidebar {}
        main {
            class: "max-w-[var(--content-max-width)]
            min-w-[var(--content-min-width)]
            mt-[var(--appbar-height)]
            mb-[var(--navbar-height)]
            mx-auto
            p-4",
            Outlet::<Route> {}
        }
        ToastRoot {}
        Navbar {}
    }
}

#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}
