#![allow(non_snake_case)]

use std::collections::HashSet;

use crate::prelude::*;
use dioxus::prelude::*;
use itertools::Itertools;
use uchat_domain::ids::{PollChoiceId, PostId};

use uchat_endpoint::post::types::{
    Chat as EndpointChat, Image as EndpointImage, ImageKind, Poll as EndpointPoll, PublicPost, VoteCast,
};

#[component]
pub fn Chat(content: ReadOnlySignal<EndpointChat>) -> Element {
    rsx! {
        div {
            if let Some(headline) = &content().headline {
                div {
                    class: "font-bold",
                    "{headline.as_ref()}"
                }
            }
            p { "{content().message.as_ref()}" }
        }
    }
}

#[component]
pub fn Image(content: ReadOnlySignal<EndpointImage>) -> Element {
    let url = if let ImageKind::Url(url) = &content().kind {
        url.clone()
    } else {
        return rsx! { "image not found" };
    };

    rsx! {
        figure {
            class: "flex flex-col gap-2",
            if let Some(caption) = &content().caption {
                figcaption { 
                    em { "{caption.as_ref()}" } 
                }
            }
            img {
                class: "w-full object-contain max-h-[80vh]",
                src: "{url}"
            }
        }
    }
}

#[component]
pub fn Poll(post_id: PostId, content: ReadOnlySignal<EndpointPoll>) -> Element {
    let mut toaster = use_toaster();  // ✅ Add mut
    let api_client = ApiClient::global();

    let vote_onclick = move |choice_id: PollChoiceId| {
        spawn(async move {
            use uchat_endpoint::post::endpoint::{Vote, VoteOk};
            let request = Vote { post_id, choice_id };
            match fetch_json!(<VoteOk>, api_client, request) {
                Ok(res) => {
                    match res.cast {
                        VoteCast::Yes => toaster.write().success("Vote cast!", chrono::Duration::seconds(3)),
                        VoteCast::AlreadyVoted => toaster.write().info("Already voted", chrono::Duration::seconds(5)),
                    }
                }
                Err(e) => toaster.write().error(
                    format!("Failed to cast vote: {}", e),
                    chrono::Duration::seconds(3),
                ),
            }
        });
    };

    let poll_data = content();
    let total_votes = poll_data
        .choices
        .iter()
        .map(|choice| choice.num_votes)
        .sum::<i64>();

    let leader_ids = {
        let leaders = poll_data
            .choices
            .iter()
            .max_set_by(|x, y| x.num_votes.cmp(&y.num_votes));
        let ids: HashSet<PollChoiceId> = HashSet::from_iter(leaders.iter().map(|choice| choice.id));
        ids
    };

    rsx! {
        div {
            figcaption { "{poll_data.headline.as_ref()}" }
            ul {
                for choice in poll_data.choices.iter() {
                    {
                        let percent = if total_votes > 0 {
                            let percent = (choice.num_votes as f64 / total_votes as f64) * 100.0;
                            format!("{percent:.0}%")
                        } else {
                            "0%".to_string()
                        };

                        let background_color = if leader_ids.contains(&choice.id) {
                            "bg-blue-300"
                        } else {
                            "bg-neutral-300"
                        };

                        let foreground_styles = maybe_class!("font-bold", leader_ids.contains(&choice.id));
                        let choice_id = choice.id;

                        rsx! {
                            li {
                                key: "{choice.id.to_string()}",
                                class: "relative p-2 m-2 cursor-pointer grid grid-cols-[3rem_1fr] border rounded border-slate-400",
                                onclick: move |_| vote_onclick(choice_id),
                                div {
                                    class: "absolute left-0 {background_color} h-full rounded z-[-1]",
                                    style: "width: {percent}",
                                }
                                div {
                                    class: "{foreground_styles}",
                                    "{percent}",
                                }
                                div {
                                    class: "{foreground_styles}",
                                    "{choice.description.as_ref()}",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Content(post: ReadOnlySignal<PublicPost>) -> Element {
    use uchat_endpoint::post::types::Content as EndpointContent;
    
    let post_data = post();
    
    rsx! {
        div {
            match &post_data.content {
                EndpointContent::Chat(content) => {
                    let chat_signal = use_signal(|| content.clone());  // ✅ Fixed
                    rsx! { Chat { content: chat_signal } }
                }
                EndpointContent::Image(content) => {
                    let image_signal = use_signal(|| content.clone());  // ✅ Fixed
                    rsx! { Image { content: image_signal } }
                }
                EndpointContent::Poll(content) => {
                    let poll_signal = use_signal(|| content.clone());  // ✅ Fixed
                    rsx! { Poll { post_id: post_data.id, content: poll_signal } }
                }
            }
        }
    }
}
