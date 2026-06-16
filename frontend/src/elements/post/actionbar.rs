#![allow(non_snake_case)]

use crate::{elements::post::quick_respond::QuickRespond, prelude::*};
use dioxus::prelude::*;
use uchat_domain::ids::PostId;
use uchat_endpoint::post::types::LikeStatus;

#[component]
pub
fn LikeDislike(
    post_id: PostId,
    like_status: LikeStatus,
    likes: i64,
    dislikes: i64,
) -> Element {
    let post_manager = use_post_manager();
    let toaster = use_toaster();
    let api_client = ApiClient::global();

    let like_icon = match like_status {
        LikeStatus::Like => "/static/icons/icon-like-selected.svg",
        _ => "/static/icons/icon-like.svg",
    };

    let dislike_icon = match like_status {
        LikeStatus::Dislike => "/static/icons/icon-dislike-selected.svg",
        _ => "/static/icons/icon-dislike.svg",
    };

    let like_onclick = async_handler!(
        &cx,
        [api_client, post_manager, toaster, post_id],
        move |like_status| async move {
            use uchat_endpoint::post::endpoint::{React, ReactOk};

            let like_status = {
                if post_manager.read().get(&post_id).unwrap().like_status == like_status {
                    LikeStatus::NoReaction
                } else {
                    like_status
                }
            };

            let request = React {
                like_status,
                post_id,
            };
            match fetch_json!(<ReactOk>, api_client, request) {
                Ok(res) => {
                    post_manager.write().update(post_id, |post| {
                        post.like_status = res.like_status;
                        post.likes = res.likes;
                        post.dislikes = res.dislikes;
                    });
                }
                Err(e) => toaster.write().error(
                    format!("Failed to react to post post: {}", e),
                    chrono::Duration::seconds(3),
                ),
            }
        }
    );

    rsx! {
        div {
            class: "cursor-pointer",
            onclick: move |_| like_onclick(LikeStatus::Like),
            img {
                class: "actionbar-icon",
                src: "{like_icon}",
            },
            div {
                class: "text-center",
                "{likes}"
            }
        },
        div {
            class: "cursor-pointer",
            onclick: move |_| like_onclick(LikeStatus::Dislike),
            img {
                class: "actionbar-icon",
                src: "{dislike_icon}",
            },
            div {
                class: "text-center",
                "{dislikes}"
            }
        }
    }
}

#[component]
pub
fn Boost(post_id: PostId, boosted: bool, boosts: i64) -> Element {
    let post_manager = use_post_manager();
    let toaster = use_toaster();
    let api_client = ApiClient::global();

    let icon = match boosted {
        true => "/static/icons/icon-boosted.svg",
        false => "/static/icons/icon-boost.svg",
    };

    let boost_onclick = async_handler!(
        &cx,
        [api_client, post_manager, toaster, post_id],
        move |_| async move {
            use uchat_endpoint::post::endpoint::{Boost, BoostOk};
            use uchat_endpoint::post::types::BoostAction;
            let action = match post_manager.read().get(&post_id).unwrap().boosted {
                true => BoostAction::Remove,
                false => BoostAction::Add,
            };

            let request = Boost { action, post_id };
            match fetch_json!(<BoostOk>, api_client, request) {
                Ok(res) => {
                    post_manager.write().update(post_id, |post| {
                        post.boosted = res.status.into();
                        if post.boosted {
                            post.boosts += 1;
                        } else {
                            post.boosts -= 1;
                        }
                    });
                }
                Err(e) => toaster.write().error(
                    format!("Failed to boost post: {}", e),
                    chrono::Duration::seconds(3),
                ),
            }
        }
    );

    rsx! {
        div {
            class: "cursor-pointer",
            onclick: boost_onclick,
            img {
                class: "actionbar-icon",
                src: "{icon}",
            }
            div {
                class: "text-center",
                "{boosts}",
            }
        }
    }
}

#[component]
pub
fn Bookmark(post_id: PostId, bookmarked: bool) -> Element {
    let post_manager = use_post_manager();
    let toaster = use_toaster();
    let api_client = ApiClient::global();

    let icon = match bookmarked {
        true => "/static/icons/icon-bookmark-saved.svg",
        false => "/static/icons/icon-bookmark.svg",
    };

    let bookmark_onclick = async_handler!(
        &cx,
        [api_client, post_manager, toaster, post_id],
        move |_| async move {
            use uchat_endpoint::post::endpoint::{Bookmark, BookmarkOk};
            use uchat_endpoint::post::types::BookmarkAction;
            let action = match post_manager.read().get(&post_id).unwrap().bookmarked {
                true => BookmarkAction::Remove,
                false => BookmarkAction::Add,
            };

            let request = Bookmark { action, post_id };
            match fetch_json!(<BookmarkOk>, api_client, request) {
                Ok(res) => {
                    post_manager.write().update(post_id, |post| {
                        post.bookmarked = res.status.into();
                    });
                }
                Err(e) => toaster.write().error(
                    format!("Failed to bookmark post: {}", e),
                    chrono::Duration::seconds(3),
                ),
            }
        }
    );

    rsx! {
        div {
            class: "cursor-pointer",
            onclick: bookmark_onclick,
            img {
                class: "actionbar-icon",
                src: "{icon}",
            }
        }
    }
}

#[component]
pub
fn Comment(opened: Signal<bool>) -> Element {
    let comment_onclick = sync_handler!([opened], move |_| {
        let current = opened();
        opened.set(!current);
    });

    rsx! {
        div {
            class: "cursor-pointer",
            onclick: comment_onclick,
            img {
                class: "actionbar-icon",
                src: "/static/icons/icon-messages.svg",
            }
        }
    }
}

#[component]
pub
fn QuickRespondBox(opened: Signal<bool>) -> Element {
    let element = match opened() {
        true => {
            to_owned![opened];
            Some(rsx! { QuickRespond { opened: opened } })
        }
        false => None,
    };

    element.unwrap_or_else(|| rsx! {})
}

#[component]
pub
fn Actionbar(post_id: PostId) -> Element {
    let post_manager = use_post_manager();
    let quick_respond_opened = use_signal(|| false).clone();

    let this_post = post_manager.read().get(&post_id).cloned();

    match this_post {
        Some(post) => {
            rsx! {
                div {
                    class: "flex flex-row justify-between w-full opacity-70 mt-4",
                    Boost {
                        post_id: post.id,
                        boosts: post.boosts,
                        boosted: post.boosted,
                    }

                    Bookmark {
                        bookmarked: post.bookmarked,
                        post_id: post.id,
                    },
                    LikeDislike {
                        post_id: post.id,
                        likes: post.likes,
                        dislikes: post.dislikes,
                        like_status: post.like_status,
                    }
                    Comment {
                        opened: quick_respond_opened.clone(),
                    }
                }
                QuickRespondBox {
                    opened: quick_respond_opened,
                }
            }
        },
        None => {
            // Handle the case where the post does not exist
            rsx! {
                div {
                    "Post not found"
                }
            }
        }
    }
}