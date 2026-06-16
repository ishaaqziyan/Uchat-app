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
pub
fn ImageInput(page_state: Signal<PageState>) -> Element {
    let toaster = use_toaster();

    rsx! {
        div {
            label {
                r#for: "image-input",
                "Upload Image"
            },
            input {
                class: "w-full",
                id: "image-input",
                r#type: "file",
                accept: "image/*",
                oninput: move |_| {
                    let mut page_state = page_state.clone();
                    let mut toaster = toaster.clone();
                    spawn(async move {
                        use gloo_file::{File, futures::read_as_data_url};
                        use wasm_bindgen::JsCast;

                        let el = util::document()
                            .get_element_by_id("image-input")
                            .unwrap()
                            .unchecked_into::<HtmlInputElement>();
                        let file: File = el.files().unwrap().get(0).unwrap().into();
                        match read_as_data_url(&file).await {
                            Ok(data) => page_state.with_mut(|state| state.profile_image = Some(PreviewImageData::DataUrl(data))),
                            Err(e) => toaster.write().error(format!("Error loading file: {e}"), chrono::Duration::seconds(5)),
                        }
                    });
                }
            }
        }
    }
}

#[component]
pub
fn ImagePreview(page_state: Signal<PageState>) -> Element {
    let image_data = page_state.read().profile_image.clone();

    let img_el = |img_src: &str| {
        rsx! {
            img {
                class: "profile-portrait-lg",
                src:"{img_src}",
            }
        }
    };

    let image_data = match image_data {
        Some(PreviewImageData::DataUrl(ref data)) => img_el(data),
        Some(PreviewImageData::Remote(ref url)) => img_el(url),
        None => rsx! { div { "No image uploaded"}},
    };

    rsx! {
        div {
            class: "flex flex-row justify-center",
            {image_data}
        }
    }
}

#[component]
pub
fn DisplayNameInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::user::DisplayName;

    let max_chars = DisplayName::MAX_CHARS;

    let wrong_len = maybe_class!(
        "err-text-color",
        page_state.read().display_name.len() > max_chars
    );


    rsx! {
        div {
            label {
                r#for: "display-name",
                div {
                    class: "flex flex-row justify-between",
                    span { "Display Name" },
                    span {
                        class: "text-right {wrong_len}",
                        "{page_state.read().display_name.len()}/{max_chars}",
                    }
 
                }
            },
            input {
                id: "display-name",
                class: "input-field",
                placeholder: "Display Name",
                value: "{page_state.read().display_name}",
                oninput: move |ev| {
                    match DisplayName::new(&ev.value()) {
                        Ok(_) => {
                            page_state.with_mut(|state| state.form_errors.remove("bad-displayname"));
                        }
                        Err(e) => {
                            page_state.with_mut(|state| state.form_errors.set("bad-displayname", e.formatted_error()));
                        }
                    }
                    page_state.with_mut(|state| state.display_name = ev.value().clone());
                }
            }
        }
    }
}
#[component]
pub
fn PasswordInput(state: Signal<PageState>) -> Element {
    use uchat_domain::user::Password;

    let mut check_password_mismatch = move || {
        let password_matches = state.read().password == state.read().password_confirmation;
        match password_matches {
            true => state.with_mut(|state| state.form_errors.remove("password-mismatch")),
            false => state.with_mut(|state| state.form_errors.set("password-mismatch", "Passwords must match")),
        }
    };

    rsx!{
        fieldset {
            class: "fieldset",
            legend { "Set new password" },
            div {
                class: "flex flex-row w-full gap-2",
                div {
                    label {
                        r#for: "password",
                        "Password"
                    },
                    input {
                        id: "password",
                        class: "input-field",
                        r#type: "password",
                        placeholder: "Password",
                        value: "{state.read().password}",
                        oninput: move |ev| {
                            match Password::new(&ev.value()) {
                                Ok(_) => state.with_mut(|state| state.form_errors.remove("bad-password")),
                                Err(e) => state.with_mut(|state| state.form_errors.set("bad-password", e.formatted_error())),
                            }
                            state.with_mut(|state| state.password = ev.value().clone());
                            state.with_mut(|state| state.password_confirmation = "".to_string());

                            if state.read().password.is_empty() {
                                state.with_mut(|state| state.form_errors.remove("bad-password"));
                                state.with_mut(|state| state.form_errors.remove("password-mismatch"));
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
                    },
                    input {
                        id: "password-confirm",
                        class: "input-field",
                        r#type: "password",
                        placeholder: "Confirm",
                        value: "{state.read().password_confirmation}",
                        oninput: move |ev| {
                            state.with_mut(|state| state.password_confirmation = ev.value().clone());
                            check_password_mismatch();
                        }
 
                }
            }
        }
    }
    }
}
 


#[component]
pub
fn EmailInput(page_state: Signal<PageState>) -> Element {
    use uchat_domain::user::Email;

    rsx! {
        div {
            label {
                r#for: "email",
                div {
                    class: "flex flex-row justify-between",
                    span { "Email Address" },
                }
            },
            input {
                class: "input-field",
                id: "email",
                placeholder: "Email Address",
                value: "{page_state.read().email}",
                oninput: move |ev| {
                    if !&ev.value().is_empty() {
                        match Email::new(&ev.value()) {
                            Ok(_) => {
                                page_state.with_mut(|state| state.form_errors.remove("bad-email"));
                            }
                            Err(e) => {
                                page_state.with_mut(|state| state.form_errors.set("bad-email", e.formatted_error()));
                            }
                        }
                    } else {
                        page_state.with_mut(|state| state.form_errors.remove("bad-email"));
                    }
                    page_state.with_mut(|state| state.email = ev.value().clone());
                }
            }
        }
    }
}

#[component]
pub
fn EditProfile() -> Element {
    let api_client = ApiClient::global();
    let page_state = use_signal( PageState::default);
    let router = use_navigator();
    let toaster = use_toaster();

    let _fetch_profile = {
        use_future(move || {
            let api_client = api_client.clone();
            let mut toaster = toaster.clone();
            let mut page_state = page_state.clone();
            async move {
            use uchat_endpoint::user::endpoint::{GetMyProfile, GetMyProfileOk};
            toaster
                .write()
                .info("Retrieving profile ...", chrono::Duration::seconds(3));
            let response = fetch_json!(<GetMyProfileOk>, api_client, GetMyProfile);
            match response {
                Ok(res) => {
                    page_state.with_mut(|state| {
                        state.display_name = res.display_name.unwrap_or_default();
                        state.email = res.email.unwrap_or_default();
                        state.profile_image = res.profile_image.map(|img| PreviewImageData::Remote(img.to_string()));
                    });
                }
                Err(e) => toaster.write().error(
                    format!("Failed to retrieve profile: {e}"),
                    chrono::Duration::seconds(3),
                ),
            }
            }
        })
    };



    let local_profile = crate::elements::local_profile::use_local_profile();
    
    let form_onsubmit =
        async_handler!(&cx, [api_client, page_state, router, toaster, local_profile], move |_| async move {
            use uchat_endpoint::user::endpoint::{UpdateProfile, UpdateProfileOk};
            use uchat_endpoint::Update;

            let request_data = {
                use uchat_domain::{Password};
                UpdateProfile {
                    display_name: {
                        let name = page_state.read().display_name.clone();
                        if name.is_empty() {
                            Update::SetNull
                        } else {
                            Update::Change(name)
                        }
                    },
                    email: {
                        let email = page_state.read().email.clone();
                        if email.is_empty() {
                            Update::SetNull
                        } else {
                            Update::Change(email)
                        }
                    },
                    password: {
                        let password = page_state.read().password.clone();
                        if password.is_empty() {
                            Update::NoChange
                        } else {
                            Update::Change(Password::new(password).unwrap())
                        }
                    },
                    profile_image: {
                        let profile_image = page_state.read().profile_image.clone();
                        match profile_image {
                            Some(PreviewImageData::DataUrl(data)) => Update::Change(data),
                            Some(PreviewImageData::Remote(_)) => Update::NoChange,
                            None => Update::SetNull,
                        }
                    },
                }
            };

            let response = fetch_json!(<UpdateProfileOk>, api_client, request_data);
            match response {
                Ok(res) => {
                    local_profile.write().image = res.profile_image;
                    toaster.write().success("Profile updated", chrono::Duration::seconds(3));
                    { let _ = router.push(crate::app::Route::Home {}); }
                }
                Err(e) => {
                    toaster.write().error(format!("Failed to update profile: {}", e), chrono::Duration::seconds(3));
                }
            }
        });



    let disable_submit = page_state.read().form_errors.has_messages();
    let submit_btn_style = maybe_class!("btn-disabled", disable_submit);

    rsx! {
        Appbar {
            title: "Edit Profile",
            AppbarImgButton {
                click_handler: move |_| { router.go_back(); },
                img: "/static/icons/icon-back.svg",
                label: "Back",
                title: "Go to the previous page",
            }
        },
        form {
            class: "flex flex-col w-full gap-3",
            onsubmit: move |ev| {
                ev.prevent_default();
                form_onsubmit(ev);
            },

            ImagePreview { page_state: page_state.clone() },
            ImageInput { page_state: page_state.clone() },
            DisplayNameInput { page_state: page_state.clone() },
            EmailInput { page_state: page_state.clone() },
            PasswordInput { state: page_state.clone() },

            KeyedNotificationBox { notifications: page_state.clone().read().form_errors.clone() },

            div {
                class: "flex flex-row justify-end gap-3",
                button {
                    class: "btn",
                    onclick: move |ev| {
                        ev.prevent_default(); router.go_back(); },
                    "Cancel"
                },
                button {
                    class: "btn {submit_btn_style}",
                    r#type: "submit",
                    disabled: disable_submit,
                    "Submit"
                },
            }
        }
    }
}
