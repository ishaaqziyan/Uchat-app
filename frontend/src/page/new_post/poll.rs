#![allow(non_snake_case)]

use std::collections::BTreeMap;

use crate::{fetch_json, prelude::*};
use chrono::Duration;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uchat_domain::{
    ids::PollChoiceId,
    post::{PollChoiceDescription, PollHeadline},
};
use uchat_endpoint::post::types::{NewPostOptions, Poll, PollChoice};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PageState {
    pub headline: String,
    pub poll_choices: BTreeMap<usize, String>,
    pub next_id: usize,
}

impl Default for PageState {
    fn default() -> Self {
        Self {
            headline: "".to_string(),
            poll_choices: {
                let mut map = BTreeMap::new();
                map.insert(0, "".to_string());
                map.insert(1, "".to_string());
                map
            },
            next_id: 2,
        }
    }
}

impl PageState {
    pub fn can_submit(&self) -> bool {
        if PollHeadline::new(&self.headline).is_err() {
            return false;
        }

        if self.poll_choices.len() < 2 {
            return false;
        }

        if self
            .poll_choices
            .values()
            .map(PollChoiceDescription::new)
            .collect::<Result<Vec<PollChoiceDescription>, _>>()
            .is_err()
        {
            return false;
        }
        true
    }

    pub fn push_choice<T: Into<String>>(&mut self, choice: T) {
        self.poll_choices.insert(self.next_id, choice.into());
        self.next_id += 1;
    }

    pub fn replace_choice<T: Into<String>>(&mut self, key: usize, choice: T) {
        self.poll_choices.insert(key, choice.into());
    }
}

#[component]
pub fn HeadlineInput(page_state: Signal<PageState>) -> Element {
    let max_chars = PollHeadline::MAX_CHARS;
    let state = page_state.read();

    let wrong_len = maybe_class!(
        "err-text-color",
        state.headline.len() > max_chars || state.headline.is_empty()
    );

    rsx! {
        div {
            label {
                r#for: "headline",
                div {
                    class: "flex flex-row justify-between",
                    span { "Headline" }
                    span {
                        class: "text-right {wrong_len}",
                        "{state.headline.len()}/{max_chars}",
                    }
                }
            }
            input {
                class: "input-field",
                id: "headline",
                value: "{state.headline}",
                oninput: move |ev| {
                    page_state.write().headline = ev.value();
                }
            }
        }
    }
}

#[component]
pub fn PollChoices(page_state: Signal<PageState>) -> Element {
    let state = page_state.read();
    let choices: Vec<_> = state
        .poll_choices
        .iter()
        .map(|(&key, choice)| (key, choice.clone()))
        .collect();
    drop(state);

    rsx! {
        div {
            class: "flex flex-col gap-2",
            "Poll Choices"
            ol {
                class: "list-decimal ml-4 flex flex-col gap-2",
                for (key, choice) in choices {
                    {
                        let max_chars = PollChoiceDescription::MAX_CHARS;
                        let wrong_len = maybe_class!(
                            "err-text-color",
                            PollChoiceDescription::new(&choice).is_err()
                        );
                        
                        rsx! {
                            li {
                                key: "{key}",
                                div {
                                    class: "grid grid-cols-[1fr_3rem_3rem] w-full gap-2 items-center h-8",
                                    input {
                                        class: "input-field",
                                        placeholder: "Choice Description",
                                        oninput: move |ev| {
                                            page_state.write().replace_choice(key, ev.value())
                                        },
                                        value: "{choice}",
                                    }
                                    div {
                                        class: "text-right {wrong_len}",
                                        "{choice.len()}/{max_chars}"
                                    }
                                    button {
                                        class: "btn p-0 h-full bg-red-700",
                                        onclick: move |evt| {
                                            evt.prevent_default();
                                            page_state.write().poll_choices.remove(&key);
                                        },
                                        "X"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div {
                class: "flex flex-row justify-end",
                button {
                    class: "btn w-12",
                    onclick: move |evt| {
                        evt.prevent_default();
                        page_state.write().push_choice("")
                    },
                    "+"
                }
            }
        }
    }
}

#[component]
pub fn NewPoll() -> Element {
    let api_client = ApiClient::global();
    let nav = use_navigator();
    let mut toaster = use_toaster();

    let mut page_state = use_signal(PageState::default);

    let form_onsubmit = move |evt: Event<FormData>| {
        evt.prevent_default();
        spawn(async move {
            use uchat_endpoint::post::endpoint::{NewPost, NewPostOk};

            let state = page_state.read();
            let request = NewPost {
                content: Poll {
                    headline: {
                        let headline = &state.headline;
                        PollHeadline::new(headline).unwrap()
                    },
                    choices: {
                        state
                            .poll_choices
                            .values()
                            .map(|choice| {
                                let id = PollChoiceId::new();
                                PollChoice {
                                    id,
                                    num_votes: 0,
                                    description: PollChoiceDescription::new(choice).unwrap(),
                                }
                            })
                            .collect::<Vec<PollChoice>>()
                    },
                    voted: None,
                }
                .into(),
                options: NewPostOptions::default(),
            };
            drop(state);

            let response = fetch_json!(<NewPostOk>, api_client, request);
            match response {
                Ok(_) => {
                    toaster.write().success("Posted!", Duration::seconds(3));
                    nav.replace(page::HOME);
                }
                Err(e) => {
                    toaster
                        .write()
                        .error(format!("Post failed: {e}"), Duration::seconds(3));
                }
            }
        });
    };

    let can_submit = page_state.read().can_submit();
    let submit_btn_style = maybe_class!("btn-disabled", !can_submit);

    let on_chat_click = move |_ev: MouseEvent| {
        nav.replace(page::POST_NEW_CHAT);
    };

    let on_image_click = move |_ev: MouseEvent| {
        nav.replace(page::POST_NEW_IMAGE);
    };

    let on_back_click = move |_ev: MouseEvent| {
        let _ = nav.go_back();
    };

    rsx! {
        Appbar {
            title: "New Poll".to_string(),
            children: rsx! {
                AppbarImgButton {
                    click_handler: on_chat_click,
                    img: "/static/icons/icon-messages.svg".to_string(),
                    label: "Chat".to_string(),
                    title: Some("Post a new chat".to_string()),
                }
                AppbarImgButton {
                    click_handler: on_image_click,
                    img: "/static/icons/icon-image.svg".to_string(),
                    label: "Image".to_string(),
                    title: Some("Post a new image".to_string()),
                }
                AppbarImgButton {
                    img: "/static/icons/icon-poll.svg".to_string(),
                    label: "Poll".to_string(),
                    disabled: Some(true),
                    title: Some("Post a new poll".to_string()),
                    append_class: Some(appbar::BUTTON_SELECTED.to_string()),
                }
                AppbarImgButton {
                    click_handler: on_back_click,
                    img: "/static/icons/icon-back.svg".to_string(),
                    label: "Back".to_string(),
                    title: Some("Go to the previous page".to_string()),
                }
            }
        }

        form {
            class: "flex flex-col gap-4",
            onsubmit: form_onsubmit,
            HeadlineInput { page_state }
            PollChoices { page_state }
            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: !can_submit,
                "Post"
            }
        }
    }
}
