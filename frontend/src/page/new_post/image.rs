#![allow(non_snake_case)]

use crate::{fetch_json, prelude::*, util};
use chrono::Duration;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uchat_endpoint::post::types::{Image, ImageKind, NewPostOptions};
use web_sys::HtmlInputElement;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PageState {
    pub caption: String,
    pub image: Option<String>,
}

impl PageState {
    pub fn can_submit(&self) -> bool {
        use uchat_domain::post::Caption;

        if !self.caption.is_empty() && Caption::new(&self.caption).is_err() {
            return false;
        }

        if self.image.is_none() {
            return false;
        }

        true
    }
}

#[component]
pub fn ImageInput(page_state: Signal<PageState>) -> Element {
    let toaster = use_toaster();

    rsx! {
        div {
            label {
                r#for: "image-input",
                "Upload Image"
            }
            input {
                class: "w-full",
                id: "image-input",
                r#type: "file",
                accept: "image/*",
                oninput: move |_| {
                    spawn(async move {
                        use gloo_file::{File, futures::read_as_data_url};
                        use wasm_bindgen::JsCast;

                        let el = util::document()
                            .get_element_by_id("image-input")
                            .unwrap()
                            .unchecked_into::<HtmlInputElement>();
                        let file: File = el.files().unwrap().get(0).unwrap().into();
                        match read_as_data_url(&file).await {
                            Ok(data) => page_state.write().image = Some(data),
                            Err(e) => toaster.write().error(format!("Error loading file: {e}"), chrono::Duration::seconds(5)),
                        }
                    });
                }
            }
        }
    }
}

#[component]
pub fn ImagePreview(page_state: Signal<PageState>) -> Element {
    let image_data = page_state.read().image.clone();

    rsx! {
        div {
            class: "flex flex-row justify-center",
            if let Some(image) = image_data {
                img {
                    class: "max-w-[calc(var(--content-max-width)/2)]
                            max-h-[40vh]",
                    src: "{image}"
                }
            } else {
                div { "no image uploaded" }
            }
        }
    }
}

#[component]
pub fn CaptionInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::post::Caption;

    let max_chars = Caption::MAX_CHARS;
    let state = page_state.read();

    let wrong_len = maybe_class!(
        "err-text-color",
        state.caption.len() > max_chars
    );

    rsx! {
        div {
            label {
                r#for: "caption",
                div {
                    class: "flex flex-row justify-between",
                    span { "Caption (optional)" }
                    span {
                        class: "text-right {wrong_len}",
                        "{state.caption.len()}/{max_chars}",
                    }
                }
            }
            input {
                class: "input-field",
                id: "caption",
                value: "{state.caption}",
                oninput: move |ev| {
                    page_state.write().caption = ev.value();
                }
            }
        }
    }
}

#[component]
pub fn NewImage() -> Element {
    let api_client = ApiClient::global();
    let nav = use_navigator();
    let toaster = use_toaster();

    let mut page_state = use_signal(PageState::default);

    let form_onsubmit = move |_| {
        spawn(async move {
            use uchat_domain::post::Caption;
            use uchat_endpoint::post::endpoint::{NewPost, NewPostOk};

            let state = page_state.read();
            let request = NewPost {
                content: Image {
                    caption: {
                        let caption = &state.caption;
                        if caption.is_empty() {
                            None
                        } else {
                            Some(Caption::new(caption).unwrap())
                        }
                    },
                    kind: {
                        let image = &state.image;
                        ImageKind::DataUrl(image.clone().unwrap())
                    },
                }
                .into(),
                options: NewPostOptions::default(),
            };
            drop(state); // Release lock before async operation

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

    rsx! {
        Appbar {
            title: "New Image".to_string(),
            children: rsx! {
                AppbarImgButton {
                    click_handler: move |_| nav.replace(page::POST_NEW_CHAT),
                    img: "/static/icons/icon-messages.svg".to_string(),
                    label: "Chat".to_string(),
                    title: Some("Post a new chat".to_string()),
                }
                AppbarImgButton {
                    click_handler: move |_| (),
                    img: "/static/icons/icon-image.svg".to_string(),
                    label: "Image".to_string(),
                    disabled: Some(true),
                    title: Some("Post a new image".to_string()),
                    append_class: Some(appbar::BUTTON_SELECTED.to_string()),
                }
                AppbarImgButton {
                    click_handler: move |_| nav.replace(page::POST_NEW_POLL),
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
            prevent_default: "onsubmit",
            ImageInput { page_state }
            ImagePreview { page_state }
            CaptionInput { page_state }
            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: !can_submit,
                "Post"
            }
        }
    }
}
