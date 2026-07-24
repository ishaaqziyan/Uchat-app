#![allow(non_snake_case)]

use crate::{elements::appbar, fetch_json, prelude::*};
use chrono::Duration;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PageState {
    pub message: String,
    pub headline: String,
}

impl PageState {
    pub fn can_submit(&self) -> bool {
        use uchat_domain::post::{Headline, Message};

        if Message::new(&self.message).is_err() {
            return false;
        }

        if !self.headline.is_empty() && Headline::new(&self.headline).is_err() {
            return false;
        }

        true
    }
}

#[component]
pub fn MessageInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::post::Message;

    let max_chars = Message::MAX_CHARS;

    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().message.len() > max_chars || page_state.read().message.is_empty()
    );

    rsx! {
        div {
            label {
                r#for: "message",
                div {
                    class: "flex flex-row justify-between",
                    span { "Message" },
                    span {
                        class: "text-right {wrong_len}",
                        "{page_state.read().message.len()}/{max_chars}",
                    }
                }
            },
            textarea {
                class: "input-field",
                id: "message",
                rows: 5,
                value: "{page_state.read().message}",
                oninput: move |ev| {
                    page_state.with_mut(|state| state.message = ev.data.value().clone());
                }
            }
        }
    }
}

#[component]
pub fn HeadlineInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::post::Headline;

    let max_chars = Headline::MAX_CHARS;

    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().headline.len() > max_chars
    );

    rsx! {
        div {
            label {
                r#for: "headline",
                div {
                    class: "flex flex-row justify-between",
                    span { "Headline" },
                    span {
                        class: "text-right {wrong_len}",
                        "{page_state.read().headline.len()}/{max_chars}",
                    }
                }
            },
            input {
                class: "input-field",
                id: "headline",
                value: "{page_state.read().headline}",
                oninput: move |ev| {
                    page_state.with_mut(|state| state.headline = ev.data.value().clone());
                }
            }
        }
    }
}

#[component]
pub fn NewChat() -> Element {
    let api_client = ApiClient::global();
    let router = use_navigator();
    let toaster = use_toaster();
    let page_state = use_signal(PageState::default);

    let form_onsubmit = async_handler!(
        &cx,
        [api_client, page_state, toaster, router],
        move |_| async move {
            use uchat_domain::post::{Headline, Message};
            use uchat_endpoint::post::endpoint::{NewPost, NewPostOk};
            use uchat_endpoint::post::types::{Chat, NewPostOptions};

            let request = NewPost {
                content: Chat {
                    headline: {
                        let headline = &page_state.read().headline;
                        if headline.is_empty() {
                            None
                        } else {
                            Some(Headline::new(headline).unwrap())
                        }
                    },
                    message: Message::new(&page_state.read().message).unwrap(),
                }
                .into(),
                options: NewPostOptions::default(),
            };
            let response = fetch_json!(<NewPostOk>, api_client, request);
            match response {
                Ok(_) => {
                    toaster.write().success("Posted!", Duration::seconds(3));
                    let _ = router.replace(page::HOME);
                }
                Err(e) => {
                    toaster
                        .write()
                        .error(format!("Post failed: {e}"), Duration::seconds(3));
                }
            }
        }
    );

    let submit_btn_style = maybe_class!("btn-disabled", !page_state.read().can_submit());

    rsx! {
        Appbar {
            title: "New Chat",
            AppbarImgButton {
                click_handler: move |_| (),
                img: "/static/icons/icon-messages.svg",
                label: "Chat",
                title: "Post a new chat",
                disabled: true,
                append_class: appbar::BUTTON_SELECTED,
            },
            AppbarImgButton {
                click_handler: move |_| { let _ = router.replace(page::POST_NEW_IMAGE); },
                img: "/static/icons/icon-image.svg",
                label: "Image",
                title: "Post a new image",
            },
            AppbarImgButton {
                click_handler: move |_| { let _ = router.replace(page::POST_NEW_POLL); },
                img: "/static/icons/icon-poll.svg",
                label: "Poll",
                title: "Post a new poll",
            },
            AppbarImgButton {
                click_handler: move |_| { router.go_back(); },
                img: "/static/icons/icon-back.svg",
                label: "Back",
                title: "Go to the previous page",
            }
        },
        form {
            class: "flex flex-col gap-4",
            onsubmit: move |ev| {
                ev.prevent_default();
                form_onsubmit(ev);
            },
            MessageInput { page_state: page_state.clone() },
            HeadlineInput { page_state: page_state.clone() },
            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: !page_state.read().can_submit(),
                "Post"
            }
        }
    }
}
