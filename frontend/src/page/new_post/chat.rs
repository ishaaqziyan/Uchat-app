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
    let state = page_state.read();

    let wrong_len = maybe_class!(
        "err-text-color",
        state.message.len() > max_chars || state.message.is_empty()
    );

    rsx! {
        div {
            label {
                r#for: "message",
                div {
                    class: "flex flex-row justify-between",
                    span { "Message" }
                    span {
                        class: "text-right {wrong_len}",
                        "{state.message.len()}/{max_chars}",
                    }
                }
            }
            textarea {
                class: "input-field",
                id: "message",
                rows: 5,
                value: "{state.message}",
                oninput: move |ev| {
                    page_state.write().message = ev.value();
                }
            }
        }
    }
}

#[component]
pub fn HeadlineInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::post::Headline;

    let max_chars = Headline::MAX_CHARS;
    let state = page_state.read();

    let wrong_len = maybe_class!(
        "err-text-color",
        state.headline.len() > max_chars
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
pub fn NewChat() -> Element {
    let api_client = ApiClient::global();
    let nav = use_navigator();
    let mut toaster = use_toaster();
    let mut page_state = use_signal(PageState::default);

    let form_onsubmit = move |ev: Event<FormData>| {  // ✅ Add event parameter
        ev.prevent_default();  // ✅ Call prevent_default
        spawn(async move {
            use uchat_domain::post::{Headline, Message};
            use uchat_endpoint::post::endpoint::{NewPost, NewPostOk};
            use uchat_endpoint::post::types::{Chat, NewPostOptions};

            let state = page_state.read();
            let request = NewPost {
                content: Chat {
                    headline: {
                        let headline = &state.headline;
                        if headline.is_empty() {
                            None
                        } else {
                            Some(Headline::new(headline).unwrap())
                        }
                    },
                    message: Message::new(&state.message).unwrap(),
                }
                .into(),
                options: NewPostOptions::default(),
            };
            drop(state);

            let response = fetch_json!(<NewPostOk>, api_client, request);
            match response {
                Ok(_) => {
                    toaster.write().success("Posted!", Duration::seconds(3));
                    let _ = nav.replace(page::HOME);  // ✅ Add let _
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

    rsx! {
        Appbar {
            title: "New Chat".to_string(),
            children: rsx! {
                AppbarImgButton {
                    click_handler: move |_| (),
                    img: "/static/icons/icon-messages.svg".to_string(),
                    label: "Chat".to_string(),
                    title: Some("Post a new chat".to_string()),
                    disabled: Some(true),
                    append_class: Some(appbar::BUTTON_SELECTED.to_string()),
                }
                AppbarImgButton {
                    click_handler: move |_| { let _ = nav.replace(page::POST_NEW_IMAGE); },  // ✅ Add braces, let _, and semicolon
                    img: "/static/icons/icon-image.svg".to_string(),
                    label: "Image".to_string(),
                    title: Some("Post a new image".to_string()),
                }
                AppbarImgButton {
                    click_handler: move |_| { let _ = nav.replace(page::POST_NEW_POLL); },  // ✅ Add braces, let _, and semicolon
                    img: "/static/icons/icon-poll.svg".to_string(),
                    label: "Poll".to_string(),
                    title: Some("Post a new poll".to_string()),
                }
                AppbarImgButton {
                    click_handler: move |_| { let _ = nav.go_back(); },
                    img: "/static/icons/icon-back.svg".to_string(),
                    label: "Back".to_string(),
                    title: Some("Go to the previous page".to_string()),
                }
            }
        }
        form {
            class: "flex flex-col gap-4",
            onsubmit: form_onsubmit,
            // ✅ Remove: prevent_default: "onsubmit",
            MessageInput { page_state }
            HeadlineInput { page_state }
            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: !can_submit,
                "Post"
            }
        }
    }
}
