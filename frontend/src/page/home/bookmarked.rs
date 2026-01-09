#![allow(non_snake_case)]

use crate::prelude::*;

use dioxus::prelude::*;

#[component]
pub fn HomeBookmarked() -> Element {
    let toaster = use_toaster();
    let api_client = ApiClient::global();
    let post_manager = use_post_manager();
    let nav = use_navigator();

    use_future(move || async move {
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
    });

    let post_ids = post_manager.read().all_post_ids();
    let has_posts = !post_ids.is_empty();

    rsx! {
        Appbar {
            title: "Bookmarked".to_string(),
            children: rsx! {
                AppbarImgButton {
                    click_handler: move |_| nav.push(page::HOME_LIKED),
                    img: "/static/icons/icon-like.svg".to_string(),
                    label: "Liked".to_string(),
                    title: Some("Show liked posts".to_string()),
                }
                AppbarImgButton {
                    click_handler: move |_| (),
                    img: "/static/icons/icon-bookmark.svg".to_string(),
                    label: "Bookmarked".to_string(),
                    title: Some("Show bookmarked posts".to_string()),
                    disabled: Some(true),
                    append_class: Some(appbar::BUTTON_SELECTED.to_string()),
                }
                AppbarImgButton {
                    click_handler: move |_| nav.push(page::HOME),
                    img: "/static/icons/icon-home.svg".to_string(),
                    label: "Home".to_string(),
                    title: Some("Go to the home page".to_string()),
                }
            }
        }

        if !has_posts {
            div {
                class: "flex flex-col text-center justify-center
                h-[calc(100vh_-_var(--navbar-height)_-_var(--appbar-height))]",
                span {
                    "You don't have any bookmarked posts yet. Check out what's "
                    a {
                        class: "link",
                        onclick: move |_| nav.push(page::POSTS_TRENDING),
                        "trending"
                    }
                    ", and follow some users to get started."
                }
            }
        } else {
            for post_id in post_ids {
                PublicPostEntry { post_id }
            }
        }
    }
}
