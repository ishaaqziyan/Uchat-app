#![allow(non_snake_case)]

use crate::{elements::post::PublicPostEntry, prelude::*};
use dioxus::prelude::*;

#[component]
pub fn Trending() -> Element {
    let api_client = ApiClient::global();
    let nav = use_navigator();
    let post_manager = use_post_manager();
    let toaster = use_toaster();

    use_future(move || async move {
        use uchat_endpoint::post::endpoint::{TrendingPosts, TrendingPostsOk};
        toaster
            .write()
            .info("Retrieving trending posts...", chrono::Duration::seconds(3));
        post_manager.write().clear();
        let response = fetch_json!(<TrendingPostsOk>, api_client, TrendingPosts);
        match response {
            Ok(res) => post_manager.write().populate(res.posts.into_iter()),
            Err(e) => toaster.write().error(
                format!("Failed to retrieve posts: {e}"),
                chrono::Duration::seconds(3),
            ),
        }
    });

    let post_ids: Vec<_> = post_manager
        .read()
        .posts
        .keys()
        .copied()
        .collect();

    rsx! {
        Appbar {
            title: "Trending Posts".to_string(),
            children: rsx! {
                AppbarImgButton {
                    click_handler: move |_| { let _ = nav.go_back(); },
                    img: "/static/icons/icon-back.svg".to_string(),
                    label: "Back".to_string(),
                    title: Some("Go to the previous page".to_string()),
                }
            }
        }
        for post_id in post_ids {
            PublicPostEntry { post_id }
        }
    }
}
