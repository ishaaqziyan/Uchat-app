#![allow(non_snake_case)]

use dioxus::prelude::*;

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
    let toaster = use_toaster();
    let local_profile = use_local_profile();

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
                nav.push(page::ACCOUNT_LOGIN)
            }
        }
    });
    
    None
}

#[component]
pub fn App() -> Element {
    let _api_client = ApiClient::global();

    rsx! {
        Router::<Route> {
            config: || RouterConfig::default().history(WebHistory::default())
        }
    }
}

// Define your router in a separate file or here:
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/account/register")]
    AccountRegister {},
    #[route("/account/login")]
    AccountLogin {},
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
}

// Or use the new layout pattern:
#[component]
pub fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Route(route: Route) -> Element {
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
            {match route {
                Route::Home {} => rsx! { page::Home {} },
                Route::AccountRegister {} => rsx! { page::Register {} },
                Route::AccountLogin {} => rsx! { page::Login {} },
                Route::HomeBookmarked {} => rsx! { page::HomeBookmarked {} },
                Route::HomeLiked {} => rsx! { page::HomeLiked {} },
                Route::PostNewChat {} => rsx! { page::NewChat {} },
                Route::PostNewImage {} => rsx! { page::NewImage {} },
                Route::PostNewPoll {} => rsx! { page::NewPoll {} },
                Route::PostsTrending {} => rsx! { page::Trending {} },
                Route::ProfileEdit {} => rsx! { page::EditProfile {} },
                Route::ProfileView { id } => rsx! { page::ViewProfile {} },
            }}
        }
        ToastRoot {}
        Navbar {}
    }
}
