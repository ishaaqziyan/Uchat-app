#![allow(non_snake_case)]

use crate::{
    elements::post::{actionbar::Actionbar, content::Content},
    prelude::*,
};
use dioxus::prelude::*;


use indexmap::IndexMap;
use uchat_domain::ids::{PostId, UserId};
use uchat_endpoint::post::types::PublicPost;

pub mod actionbar;
pub mod content;
pub mod quick_respond;

pub fn use_post_manager() -> Signal<PostManager> {
    use_context::<Signal<PostManager>>()
}

#[derive(Default)]
pub struct PostManager {
    pub posts: IndexMap<PostId, PublicPost>,
}

impl PostManager {
    pub fn update<F>(&mut self, id: PostId, mut update_fn: F) -> bool
    where
        F: FnMut(&mut PublicPost),
    {
        if let Some(post) = self.posts.get_mut(&id) {
            update_fn(post);
            true
        } else {
            false
        }
    }

    pub fn populate<T>(&mut self, posts: T)
    where
        T: Iterator<Item = PublicPost>,
    {
        self.posts.clear();
        for post in posts {
            self.posts.insert(post.id, post);
        }
    }

    pub fn clear(&mut self) {
        self.posts.clear();
    }

    pub fn get(&self, post_id: &PostId) -> Option<&PublicPost> {
        self.posts.get(post_id)
    }

    pub fn remove(&mut self, post_id: &PostId) {
        self.posts.remove(post_id);
    }

    pub fn all_to_public(&self) -> Vec<Element> {
        self.posts
            .iter()
            .map(|(&id, _)| {
                rsx! {
                    div {
                        key: "{id.to_string()}",
                        PublicPostEntry {
                            post_id: id
                        }
                    }
                }
            })
            .collect()
    }

    pub fn bookmarked_to_public(&self) -> Vec<Element> {
        self.posts
            .iter()
            .filter(|(_, post)| post.bookmarked)
            .map(|(&id, _)| {
                rsx! {
                    div {
                        key: "{id.to_string()}",
                        PublicPostEntry {
                            post_id: id
                        }
                    }
                }
            })
            .collect()
    }

    pub fn liked_to_public(&self) -> Vec<Element> {
        self.posts
            .iter()
            .filter(|(_, post)| post.like_status == uchat_endpoint::post::types::LikeStatus::Like)
            .map(|(&id, _)| {
                rsx! {
                    div {
                        key: "{id.to_string()}",
                        PublicPostEntry {
                            post_id: id
                        }
                    }
                }
            })
            .collect()
    }
}

pub fn view_profile_onclick(
    router: dioxus::router::Navigator,
    user_id: UserId,
) -> impl FnMut(MouseEvent) + 'static {
    sync_handler!([router], move |_| {
        let route = crate::app::Route::ViewProfile { user_id };
        { router.push(route); }
    })
}

#[component]
pub
fn ProfileImage(post_id: PostId) -> Element {
    let post_manager = use_post_manager();
    let router = use_navigator();
    
    let post_read = post_manager.read();
    let post = match post_read.get(&post_id) {
        Some(p) => p,
        None => return rsx! {},
    };

    let poster_info = &post.by_user;

    let profile_img_src = poster_info
        .profile_image
        .as_ref()
        .map(|url| url.as_str())
        .filter(|url| !url.is_empty())
        .unwrap_or_else(|| "/static/icons/uchat.png");

    rsx! {
        div {
            img {
                class: "profile-portrait cursor-pointer",
                onclick: view_profile_onclick(router.clone(), post.by_user.id),
                src: "{profile_img_src}",
            }
        }
    }
}

#[component]
pub
fn Header(post_id: PostId) -> Element {
    let post_manager = use_post_manager();
    let router = use_navigator();
    
    let post_read = post_manager.read();
    let post = match post_read.get(&post_id) {
        Some(p) => p,
        None => return rsx! {},
    };
    
    let (post_date, post_time) = {
        let date = post.time_posted.format("%Y-%m-%d");
        let time = post.time_posted.format("%H:%M:%S");
        (date, time)
    };

    let display_name = match &post.by_user.display_name {
        Some(name) => name.as_ref(),
        None => "",
    };

    let handle = &post.by_user.handle;

    rsx! {
        div {
            class: "flex flex-row justify-between",
            div {
                class: "cursor-pointer",
                onclick: view_profile_onclick(router.clone(), post.by_user.id),
                div { "{display_name} "},
                div {
                    class: "font-light",
                    "{handle}"
                }
            },
            div {
                class: "text-right",
                div { "{post_date}" },
                div { "{post_time}" },
            }
        }
    }
}

#[component]
pub
fn PublicPostEntry(post_id: PostId) -> Element {
    let post_manager = use_post_manager();
    let _router = use_navigator();

    let post_exists = post_manager.read().get(&post_id).is_some();

    if !post_exists {
        return rsx! {};
    }

    rsx! {
        div {
            class: "grid grid-cols-[50px_1fr] gap-2 mb-4",
            ProfileImage {
                post_id: post_id,
            }
            div {
                class: "flex flex-col gap-3",
                Header { post_id: post_id },
                // reply to
                Content { post_id: post_id },
                Actionbar { post_id: post_id },
                hr {},
            }
        }
    }
}


