#![allow(non_snake_case)] 

use crate::{elements::post::PublicPostEntry, prelude::*};
use dioxus::prelude::*;

/// Trending page component — fetches and displays trending posts from the API.
pub fn Trending(cx: Scope) -> Element {
    let api_client = ApiClient::global();       // Shared API client for making HTTP requests
    let router = use_router(cx);               // Router for navigating back to home
    let post_manager = use_post_manager(cx);   // Global post state manager
    let toaster = use_toaster(cx);             // Global toast notification system

    // Async task that runs once on mount to fetch trending posts from the backend
    let _fetch_trending_posts = {
        to_owned![api_client, toaster, post_manager]; // Clone into the async closure for ownership
        use_future(cx, (), |_| async move {
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
        })
    };

    // Build a list of rendered post elements from the current post manager state.
    // Each post is wrapped in a `div` and rendered as a `PublicPostEntry` component.
    let TrendingPosts = post_manager
        .read()
        .posts
        .iter()
        .map(|(&id, _)| {
            rsx! {
                div {
                    PublicPostEntry {
                        post_id: id, // Pass the post ID; PublicPostEntry looks up the rest from post_manager
                    }
                }
            }
        })
        .collect::<Vec<LazyNodes>>(); // Collect into a vec of lazy RSX nodes for rendering

    cx.render(rsx! {
        // Top app bar with a back button that navigates to the home page
        Appbar {
            title: "Trending Posts",
            AppbarImgButton {
                click_handler: move |_| router.navigate_to(page::HOME),
                img: "/static/icons/icon-back.svg",
                label: "Back",
                title: "Go to the previous page",
            }
        },
        // Render each trending post entry collected above
        TrendingPosts.into_iter(),
    })
}
