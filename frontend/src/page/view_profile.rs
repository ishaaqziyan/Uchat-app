#![allow(non_snake_case)]

use crate::{prelude::*, elements::post::PublicPostEntry};
use dioxus::prelude::*;
use uchat_domain::ids::UserId;

#[derive(Clone, Copy, PartialEq, Props)]
pub struct ViewProfileProps {
    pub user_id: UserId,
}

#[component]
pub fn ViewProfile(props: ViewProfileProps) -> Element {
    let api_client = ApiClient::global();
    let mut toaster = use_toaster();
    let nav = use_navigator();
    let mut post_manager = use_post_manager();
    let local_profile = use_local_profile();

    let mut profile = use_signal(|| None::<uchat_endpoint::user::endpoint::ViewProfileOk>);
    let user_id = props.user_id;

    use_effect(move || {
        spawn(async move {
            use uchat_endpoint::user::endpoint::{ViewProfile, ViewProfileOk};
            let request = ViewProfile { for_user: user_id };
            post_manager.write().clear();
            let response = fetch_json!(<ViewProfileOk>, api_client, request);
            match response {
                Ok(res) => {
                    let posts = res.posts.clone();
                    profile.set(Some(res));
                    post_manager.write().populate(posts.into_iter());
                }
                Err(e) => toaster.write().error(
                    format!("Failed to retrieve posts: {e}"),
                    chrono::Duration::seconds(3),
                ),
            }
        });
    });

    let follow_onclick = move |_| {
        spawn(async move {
            use uchat_endpoint::user::endpoint::{FollowUser, FollowUserOk};
            use uchat_endpoint::user::types::FollowAction;
            
            let am_following = match profile.read().as_ref() {
                Some(prof) => prof.profile.am_following,
                None => false,
            };

            let request = FollowUser {
                user_id,
                action: match am_following {
                    true => FollowAction::Unfollow,
                    false => FollowAction::Follow,
                },
            };
            match fetch_json!(<FollowUserOk>, api_client, request) {
                Ok(res) => {
                    if let Some(view_profile_ok) = profile.write().as_mut() {
                        view_profile_ok.profile.am_following = res.status.into();
                    }
                }
                Err(e) => toaster.write().error(
                    format!("Failed to update follow status: {}", e),
                    chrono::Duration::seconds(3),
                ),
            }
        });
    };

    let profile_data = profile.read().as_ref().map(|vp| vp.profile.clone());
    let post_ids = post_manager.read().all_post_ids();

    rsx! {
        Appbar {
            title: "View Profile".to_string(),
            children: rsx! {
                AppbarImgButton {
                    click_handler: move |_| { let _ = nav.go_back(); },
                    img: "/static/icons/icon-back.svg".to_string(),
                    label: "Back".to_string(),
                    title: Some("Go to the previous page".to_string()),
                }
            }
        }
        
        if let Some(prof) = profile_data {
            {
                let display_name = prof
                    .display_name
                    .map(|name| name.into_inner())
                    .unwrap_or_else(|| "(None)".to_string());
                let profile_image = prof
                    .profile_image
                    .map(|url| url.to_string())
                    .unwrap_or_else(|| "".to_string());

                let follow_button_text = match prof.am_following {
                    true => "Unfollow",
                    false => "Follow",
                };

                let show_follow_button = local_profile.read().user_id
                    .map(|id| id != prof.id)
                    .unwrap_or(false);

                rsx! {
                    div {
                        class: "flex flex-col gap-3",
                        div {
                            class: "flex flex-row justify-center",
                            img {
                                class: "profile-portrait-lg",
                                src: "{profile_image}",
                            }
                        }
                        div { "Handle: {prof.handle}" }
                        div { "Name: {display_name} " }
                        if show_follow_button {
                            button {
                                class: "btn",
                                onclick: follow_onclick,
                                "{follow_button_text}"
                            }
                        }
                    }
                }
            }
        } else {
            "Loading profile..."
        }

        div {
            class: "font-bold text-center my-6",
            "Posts"
        }
        hr {
            class: "h-px my-6 bg-gray-200 border-0",
        }
        for post_id in post_ids {
            PublicPostEntry { post_id }
        }
    }
}
