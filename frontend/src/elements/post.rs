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
    *crate::app::POSTMANAGER
}

#[derive(Default, Clone)]
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

    pub fn all_post_ids(&self) -> Vec<PostId> {
        self.posts.keys().copied().collect()
    }
}

pub fn view_profile_onclick(
    nav: Navigator,
    user_id: UserId,
) -> impl Fn(MouseEvent) + 'static {
    move |_| {
        let route = crate::page::route::profile_view(user_id);
        nav.push(route);
    }
}

#[component]
pub fn ProfileImage(post: ReadOnlySignal<PublicPost>) -> Element {
    let nav = use_navigator();

    let post_data = post();
    let poster_info = &post_data.by_user;

    let profile_img_src = poster_info
        .profile_image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or("");

    rsx! {
        div {
            img {
                class: "profile-portrait cursor-pointer",
                onclick: view_profile_onclick(nav, post_data.by_user.id),
                src: "{profile_img_src}",
            }
        }
    }
}

#[component]
pub fn Header(post: ReadOnlySignal<PublicPost>) -> Element {
    let post_data = post();
    
    let (post_date, post_time) = {
        let date = post_data.time_posted.format("%Y-%m-%d");
        let time = post_data.time_posted.format("%H:%M:%S");
        (date, time)
    };

    let display_name = match &post_data.by_user.display_name {
        Some(name) => name.as_ref(),
        None => "",
    };

    let handle = &post_data.by_user.handle;

    rsx! {
        div {
            class: "flex flex-row justify-between",
            div {
                class: "cursor-pointer",
                onclick: move |_| (),
                div { "{display_name} " }
                div {
                    class: "font-light",
                    "{handle}"
                }
            }
            div {
                class: "text-right",
                div { "{post_date}" }
                div { "{post_time}" }
            }
        }
    }
}

#[component]
pub fn PublicPostEntry(post_id: PostId) -> Element {
    let post_manager = use_post_manager();

    let this_post = use_memo(move || {
        post_manager.read().get(&post_id).cloned()
    });

    let Some(post_data) = this_post() else {
        return None;
    };

    let post_signal = use_signal(|| post_data.clone());

    rsx! {
        div {
            key: "{post_id.to_string()}",
            class: "grid grid-cols-[50px_1fr] gap-2 mb-4",
            ProfileImage {
                post: post_signal.into(),
            }
            div {
                class: "flex flex-col gap-3",
                Header { post: post_signal.into() }
                // reply to
                Content { post: post_signal.into() }
                Actionbar { post_id: post_data.id }
                hr {}
            }
        }
    }
}
