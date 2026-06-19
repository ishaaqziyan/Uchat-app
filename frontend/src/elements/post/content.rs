#![allow(non_snake_case)]

use std::collections::HashSet;

use crate::prelude::*;
use dioxus::prelude::*;
use itertools::Itertools;
use uchat_domain::ids::{PollChoiceId, PostId};

use uchat_endpoint::post::types::{
    Chat as EndpointChat, Image as EndpointImage, ImageKind, Poll as EndpointPoll, VoteCast,
};

#[component]
pub fn Chat(content: EndpointChat) -> Element {
    let Headline = content.headline.as_ref().map(|headline| {
        rsx! {
            div { class: "font-bold", "{headline.as_ref()}" }
        }
    });

    rsx! {
        div {
            {Headline}
            p { "{content.message.as_ref()}" }
        }
    }
}

#[component]
pub fn Image(content: EndpointImage) -> Element {
    let url = if let ImageKind::Url(url) = &content.kind {
        url
    } else {
        return rsx! { "image not found" };
    };

    let Caption = content.caption.as_ref().map(|caption| {
        rsx! {
            figcaption {
                em { "{caption.as_ref()}" }
            }
        }
    });

    rsx! {
        figure { class: "flex flex-col gap-2",
            {Caption}
            img { class: "w-full object-contain max-h-[80vh]", src: "{url}" }
        }
    }
}

#[component]
pub fn Poll(post_id: PostId, content: EndpointPoll) -> Element {
    let toaster = use_toaster();
    let api_client = ApiClient::global();

    let vote_onclick = async_handler!(
        &cx,
        [api_client, toaster],
        move |post_id, choice_id| async move {
            use uchat_endpoint::post::endpoint::{Vote, VoteOk};
            let request = Vote { post_id, choice_id };
            match fetch_json!(<VoteOk>, api_client, request) {
                Ok(res) => match res.cast {
                    VoteCast::Yes => toaster
                        .write()
                        .success("Vote cast!", chrono::Duration::seconds(3)),
                    VoteCast::AlreadyVoted => toaster
                        .write()
                        .info("Already voted", chrono::Duration::seconds(5)),
                },
                Err(e) => toaster.write().error(
                    format!("Failed to cast vote: {}", e),
                    chrono::Duration::seconds(3),
                ),
            }
        }
    );

    let total_votes = content
        .choices
        .iter()
        .map(|choice| choice.num_votes)
        .sum::<i64>();

    let leader_ids = {
        let leaders = content
            .choices
            .iter()
            .max_set_by(|x, y| x.num_votes.cmp(&y.num_votes));
        let ids: HashSet<PollChoiceId> = HashSet::from_iter(leaders.iter().map(|choice| choice.id));
        ids
    };

    let Choices: Vec<_> = content.choices.iter().map(|choice| {
        let choice_id = choice.id;
        let percent = if total_votes > 0 {
            let percent = (choice.num_votes as f64 / total_votes as f64) * 100.0;
            format!("{percent:.0}%")
        } else {
            "0%".to_string()
        };

        let background_color = if leader_ids.contains(&choice_id) {
            "bg-blue-300"
        } else {
            "bg-neutral-300"
        };

        let foreground_styles = maybe_class!("font-bold", leader_ids.contains(&choice_id));

        rsx! {
            li {
                key: "{choice_id.to_string()}",
                class: "relative p-2 m-2 cursor-pointer grid grid-cols-[3rem_1fr] border rounded border-slate-400",
                onclick: move |_| vote_onclick(post_id, choice_id),
                div {
                    class: "absolute left-0 {background_color} h-full rounded z-[-1]",
                    style: "width: {percent}",
                }
                div { class: "{foreground_styles}", "{percent}" }
                div { class: "{foreground_styles}", "{choice.description.as_ref()}" }
            }
        }
    }).collect();

    let Headline = rsx! {
        figcaption { "{content.headline.as_ref()}" }

    };

    rsx! {
        div {
            {Headline}
            ul { {Choices.into_iter()} }
        }
    }
}

#[component]
pub fn Content(post_id: PostId) -> Element {
    use uchat_endpoint::post::types::Content as EndpointContent;
    let post_manager = crate::elements::post::use_post_manager();
    let post_read = post_manager.read();
    let post = post_read.get(&post_id).unwrap();

    rsx! {
        div {
            match &post.content {
                EndpointContent::Chat(c) => rsx! {
                    Chat { content: c.clone() }
                },
                EndpointContent::Image(c) => rsx! {
                    Image { content: c.clone() }
                },
                EndpointContent::Poll(c) => rsx! {
                    Poll { post_id: post.id, content: c.clone() }
                },
            }
        }
    }
}
