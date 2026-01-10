#![allow(non_snake_case)]

use crate::{
    elements::{keyed_notification_box::KeyedNotifications, KeyedNotificationBox},
    prelude::*,
    util,
};
use dioxus::prelude::*;
use uchat_domain::UserFacingError;
use web_sys::HtmlInputElement;

#[derive(Clone, Debug)]
enum PreviewImageData {
    DataUrl(String),
    Remote(String),
}

#[derive(Clone, Debug, Default)]
pub struct PageState {
    form_errors: KeyedNotifications,

    display_name: String,
    email: String,
    password: String,
    password_confirmation: String,
    profile_image: Option<PreviewImageData>,
}

#[component]
pub fn ImageInput(page_state: Signal<PageState>) -> Element {
    let mut toaster = use_toaster();

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
                            Ok(data) => page_state.write().profile_image = Some(PreviewImageData::DataUrl(data)),
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
    let image_data = page_state.read().profile_image.clone();

    rsx! {
        div {
            class: "flex flex-row justify-center",
            match image_data {
                Some(PreviewImageData::DataUrl(data)) => rsx! {
                    img {
                        class: "profile-portrait-lg",
                        src: "{data}",
                    }
                },
                Some(PreviewImageData::Remote(url)) => rsx! {
                    img {
                        class: "profile-portrait-lg",
                        src: "{url}",
                    }
                },
                None => rsx! { div { "No image uploaded" } },
            }
        }
    }
}

#[component]
pub fn DisplayNameInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::user::DisplayName;

    let max_chars = DisplayName::MAX_CHARS;
    let state = page_state.read();

    let wrong_len = maybe_class!(
        "err-text-color",
        state.display_name.len() > max_chars
    );

    rsx! {
        div {
            label {
                r#for: "display-name",
                div {
                    class: "flex flex-row justify-between",
                    span { "Display Name" }
                    span {
                        class: "text-right {wrong_len}",
                        "{state.display_name.len()}/{max_chars}",
                    }
                }
            }
            input {
                id: "display-name",
                class: "input-field",
                placeholder: "Display Name",
                value: "{state.display_name}",
                oninput: move |ev| {
                    let value = ev.value();
                    match DisplayName::new(&value) {
                        Ok(_) => {
                            page_state.write().form_errors.remove("bad-displayname");
                        }
                        Err(e) => {
                            page_state.write().form_errors.set("bad-displayname", e.formatted_error());
                        }
                    }
                    page_state.write().display_name = value;
                }
            }
        }
    }
}

#[component]
pub fn PasswordInput(state: Signal<PageState>) -> Element {
    use uchat_domain::user::Password;

    let mut check_password_mismatch = move || {
        let state_read = state.read();
        let password_matches = state_read.password == state_read.password_confirmation;
        drop(state_read);
        
        match password_matches {
            true => state.write().form_errors.remove("password-mismatch"),
            false => state.write().form_errors.set("password-mismatch", "Passwords must match"),
        }
    };

    let state_read = state.read();
    let password_val = state_read.password.clone();
    let password_confirmation_val = state_read.password_confirmation.clone();
    drop(state_read);

    rsx! {
        fieldset {
            class: "fieldset",
            legend { "Set new password" }
            div {
                class: "flex flex-row w-full gap-2",
                div {
                    label {
                        r#for: "password",
                        "Password"
                    }
                    input {
                        id: "password",
                        class: "input-field",
                        r#type: "password",
                        placeholder: "Password",
                        value: "{password_val}",
                        oninput: move |ev| {
                            let value = ev.value();
                            match Password::new(&value) {
                                Ok(_) => state.write().form_errors.remove("bad-password"),
                                Err(e) => state.write().form_errors.set("bad-password", e.formatted_error()),
                            }
                            let mut state_write = state.write();
                            state_write.password = value.clone();
                            state_write.password_confirmation = "".to_string();
                            drop(state_write);

                            if value.is_empty() {
                                let mut state_write = state.write();
                                state_write.form_errors.remove("bad-password");
                                state_write.form_errors.remove("password-mismatch");
                            } else {
                                check_password_mismatch();
                            }
                        }
                    }
                }
                div {
                    label {
                        r#for: "password-confirm",
                        "Confirm"
                    }
                    input {
                        id: "password-confirm",
                        class: "input-field",
                        r#type: "password",
                        placeholder: "Confirm",
                        value: "{password_confirmation_val}",
                        oninput: move |ev| {
                            state.write().password_confirmation = ev.value();
                            check_password_mismatch();
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn EmailInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::user::Email;

    let state = page_state.read();
    let email_val = state.email.clone();
    drop(state);

    rsx! {
        div {
            label {
                r#for: "email",
                div {
                    class: "flex flex-row justify-between",
                    span { "Email Address" }
                }
            }
            input {
                class: "input-field",
                id: "email",
                placeholder: "Email Address",
                value: "{email_val}",
                oninput: move |ev| {
                    let value = ev.value();
                    if !value.is_empty() {
                        match Email::new(&value) {
                            Ok(_) => {
                                page_state.write().form_errors.remove("bad-email");
                            }
                            Err(e) => {
                                page_state.write().form_errors.set("bad-email", e.formatted_error());
                            }
                        }
                    } else {
                        page_state.write().form_errors.remove("bad-email");
                    }
                    page_state.write().email = value;
                }
            }
        }
    }
}

#[component]
pub fn EditProfile() -> Element {
    let api_client = ApiClient::global();
    let mut page_state = use_signal(PageState::default);
    let nav = use_navigator();
    let mut toaster = use_toaster();

    use_future(move || async move {
        use uchat_endpoint::user::endpoint::{GetMyProfile, GetMyProfileOk};
        toaster
            .write()
            .info("Retrieving profile ...", chrono::Duration::seconds(3));
        let response = fetch_json!(<GetMyProfileOk>, api_client, GetMyProfile);
        match response {
            Ok(res) => {
                let mut state = page_state.write();
                state.display_name = res.display_name.unwrap_or_default();
                state.email = res.email.unwrap_or_default();
                state.profile_image = res.profile_image.map(|img| PreviewImageData::Remote(img.to_string()));
            }
            Err(e) => toaster.write().error(
                format!("Failed to retrieve profile: {e}"),
                chrono::Duration::seconds(3),
            ),
        }
    });

    let form_onsubmit = move |ev: Event<FormData>| {
        ev.prevent_default();
        spawn(async move {
            use uchat_endpoint::user::endpoint::{UpdateProfile, UpdateProfileOk};
            use uchat_endpoint::Update;

            let state = page_state.read();
            let request_data = {
                use uchat_domain::Password;
                UpdateProfile {
                    display_name: {
                        let name = state.display_name.clone();
                        if name.is_empty() {
                            Update::SetNull
                        } else {
                            Update::Change(name)
                        }
                    },
                    email: {
                        let email = state.email.clone();
                        if email.is_empty() {
                            Update::SetNull
                        } else {
                            Update::Change(email)
                        }
                    },
                    password: {
                        let password = state.password.clone();
                        if password.is_empty() {
                            Update::NoChange
                        } else {
                            Update::Change(Password::new(password).unwrap())
                        }
                    },
                    profile_image: {
                        let profile_image = state.profile_image.clone();
                        match profile_image {
                            Some(PreviewImageData::DataUrl(data)) => Update::Change(data),
                            Some(PreviewImageData::Remote(_)) => Update::NoChange,
                            None => Update::SetNull,
                        }
                    },
                }
            };
            drop(state);

            let response = fetch_json!(<UpdateProfileOk>, api_client, request_data);
            match response {
                Ok(_res) => {
                    toaster.write().success("Profile updated", chrono::Duration::seconds(3));
                    let _ = nav.push(crate::page::HOME);
                }
                Err(e) => {
                    toaster.write().error(format!("Failed to update profile: {}", e), chrono::Duration::seconds(3));
                }
            }
        });
    };

    let disable_submit = page_state.read().form_errors.has_messages();
    let submit_btn_style = maybe_class!("btn-disabled", disable_submit);
    let form_errors = page_state.read().form_errors.clone();

    rsx! {
        Appbar {
            title: "Edit Profile".to_string(),
            children: rsx! {
                AppbarImgButton {
                    click_handler: move |_| { let _ = nav.go_back(); },
                    img: "/static/icons/icon-back.svg".to_string(),
                    label: "Back".to_string(),
                    title: Some("Go to the previous page".to_string()),
                }
            }
        }
        form {
            class: "flex flex-col w-full gap-3",
            onsubmit: form_onsubmit,

            ImagePreview { page_state }
            ImageInput { page_state }
            DisplayNameInput { page_state }
            EmailInput { page_state }
            PasswordInput { state: page_state }

            KeyedNotificationBox { notifications: form_errors }

            div {
                class: "flex flex-row justify-end gap-3",
                button {
                    class: "btn",
                    onclick: move |ev| {
                        ev.prevent_default();
                        let _ = nav.go_back();
                    },
                    "Cancel"
                }
                button {
                    class: "btn {submit_btn_style}",
                    r#type: "submit",
                    disabled: disable_submit,
                    "Submit"
                }
            }
        }
    }
}
