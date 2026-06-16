#![allow(non_snake_case)]

use crate::prelude::*;

use dioxus::prelude::*;

#[component]
pub
fn HomeBookmarked() -> Element {
    let toaster = use_toaster();
    let api_client = ApiClient::global();
    let post_manager = use_post_manager();
    let router = use_navigator();

    let _fetch_posts = {
        use_future(move || {
            let api_client = api_client.clone();
            let mut toaster = toaster.clone();
            let mut post_manager = post_manager.clone();
            async move {
            use uchat_endpoint::post::endpoint::{BookmarkedPosts, BookmarkedPostsOk};
            toaster
                .write()
                .info("Retrieving posts...", chrono::Duration::seconds(3));
            post_manager.write().clear();
            let response = fetch_json!(<BookmarkedPostsOk>, api_client, BookmarkedPosts);
            match response {
                Ok(res) => post_manager.write().populate(res.posts.into_iter()),
                Err(e) => toaster.write().error(
                    format!("Failed to retrieve posts: {e}"),
                    chrono::Duration::seconds(3),
                ),
            }
            }
        })
    };

    let Posts = {
        let posts = post_manager.read().all_to_public();
        if posts.is_empty() {
            let TrendingLink = rsx! {
                a {
                    class: "link",
                    onclick: move |_| {
                        { router.push(crate::app::Route::Trending {}); };
                    },
                    "trending"
                },
            };
            rsx! {
                div {
                    class: "flex flex-col text-center justify-center
                    h-[calc(100vh_-_var(--navbar-height)_-_var(--appbar-height))]",
                    span {
                        "You don't have any bookmarked posts yet. Check out what's ", {TrendingLink}, ", and follow some users to get started."
                    }
                }
            }
        } else {
            rsx! { {posts.into_iter()} }
        }
    };

    rsx! {
        Appbar {
            title: "Bookmarked",
            AppbarImgButton {
                click_handler: move |_| { router.push(crate::app::Route::HomeLiked {}); },
                img: "/static/icons/icon-like.svg",
                label: "Liked",
                title: "Show liked posts",
            },
            AppbarImgButton {
                click_handler: move |_| (),
                img: "/static/icons/icon-bookmark.svg",
                label: "Bookmarked",
                title: "Show bookmarked posts",
                disabled: true,
                append_class: appbar::BUTTON_SELECTED,
            },
            AppbarImgButton {
                click_handler: move |_| { router.push(crate::app::Route::Home {}); },
                img: "/static/icons/icon-home.svg",
                label: "Home",
                title: "Go to the home page",
            },

        },

        {Posts}
    }
}
