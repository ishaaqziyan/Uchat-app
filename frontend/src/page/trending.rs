#![allow(non_snake_case)] 

use crate::{elements::post::PublicPostEntry, prelude::*};
use dioxus::prelude::*;

/// Trending page component — fetches and displays trending posts from the API.
#[component]
pub
fn Trending() -> Element {
    let api_client = ApiClient::global();       // Shared API client for making HTTP requests
    let router = use_navigator();               // Router for navigating back to home
    let post_manager = use_post_manager();   // Global post state manager
    let toaster = use_toaster();             // Global toast notification system

    // Async task that runs once on mount to fetch trending posts from the backend
    let _fetch_trending_posts = {
        let api_client = api_client.clone();
        let mut toaster = toaster.clone();
        let mut post_manager = post_manager.clone();
        use_future(move || {
            let api_client = api_client.clone();
            let mut toaster = toaster.clone();
            let mut post_manager = post_manager.clone();
            async move {
            use uchat_endpoint::post::endpoint::{TrendingPosts, TrendingPostsOk};

            // Notify the user that the fetch has started
            toaster
                .write()
                .info("Retrieving trending posts...", chrono::Duration::seconds(3));

            post_manager.write().clear(); // Clear any previously loaded posts before fetching new ones

            // Make the API call to fetch trending posts
            let response = fetch_json!(<TrendingPostsOk>, api_client, TrendingPosts);

            match response {
                // On success, populate the post manager with the returned posts
                Ok(res) => post_manager.write().populate(res.posts.into_iter()),
                // On failure, show an error toast with the error message
                Err(e) => toaster.write().error(
                    format!("Failed to retrieve posts: {e}"),
                    chrono::Duration::seconds(3),
                ),
            }
            }
        })
    };

    // Build a list of rendered post elements from the current post manager state.
    // Each post is wrapped in a `div` and rendered as a `PublicPostEntry` component.
    let post_ids: Vec<_> = post_manager.read().posts.keys().copied().collect();
    let TrendingPosts = post_ids.into_iter().map(|id| {
            rsx! {
                div {
                    PublicPostEntry {
                        post_id: id, // Pass the post ID; PublicPostEntry looks up the rest from post_manager
                    }
                }
            }
        });

    rsx! {
        // Top app bar with a back button that navigates to the home page
        Appbar {
            title: "Trending Posts",
            AppbarImgButton {
                click_handler: move |_| { let _ = router.push(page::HOME); },
                img: "/static/icons/icon-back.svg",
                label: "Back",
                title: "Go to the previous page",
            }
        },
        // Render each trending post entry collected above
        {TrendingPosts}
    }
}
