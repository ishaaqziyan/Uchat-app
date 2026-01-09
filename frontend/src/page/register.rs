#![allow(non_snake_case)]

use dioxus::prelude::*;
use uchat_domain::UserFacingError;

use crate::{
    elements::{keyed_notification_box::KeyedNotifications, KeyedNotificationBox},
    fetch_json,
    prelude::*,
    util::ApiClient,
};

#[derive(Clone)]
pub struct PageState {
    username: Signal<String>,
    password: Signal<String>,
    form_errors: Signal<KeyedNotifications>,
}

impl PageState {
    pub fn new() -> Self {
        Self {
            username: Signal::new(String::new()),
            password: Signal::new(String::new()),
            form_errors: Signal::new(KeyedNotifications::default()),
        }
    }
    pub fn can_submit(&self) -> bool {
        !(self.form_errors.read().has_messages()
            || self.username.read().is_empty()
            || self.password.read().is_empty())
    }
}

#[component]
pub fn PasswordInput(
    state: Signal<String>,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let state_val = state.read().clone();

    rsx! {
        div {
            class: "flex flex-col",
            label {
                r#for: "password",
                "Password",
            }
            input {
                id: "password",
                r#type: "password",
                name: "password",
                class: "input-field",
                placeholder: "Password",
                value: "{state_val}",
                oninput: move |ev| oninput.call(ev),
            }
        }
    }
}

#[component]
pub fn UsernameInput(
    state: Signal<String>,
    oninput: EventHandler<FormEvent>,
) -> Element {
    let state_val = state.read().clone();

    rsx! {
        div {
            class: "flex flex-col",
            label {
                r#for: "username",
                "Username",
            }
            input {
                id: "username",
                name: "username",
                class: "input-field",
                placeholder: "User Name",
                value: "{state_val}",
                oninput: move |ev| oninput.call(ev),
            }
        }
    }
}

#[component]
pub fn LoginLink() -> Element {
    let nav = use_navigator();

    rsx! {
        a {
            class: "link text-center cursor-pointer",
            onclick: move |_| nav.push(page::ACCOUNT_LOGIN),
            "Existing User Login"
        }
    }
}

#[component]
pub fn Register() -> Element {
    let api_client = ApiClient::global();
    let page_state = use_signal(|| PageState::new());
    let nav = use_navigator();
    let local_profile = use_local_profile();

    let form_onsubmit = move |_| {
        spawn(async move {
            use uchat_endpoint::user::endpoint::{CreateUser, CreateUserOk};
            
            let state = page_state.read();
            let request_data = {
                use uchat_domain::{Password, Username};
                CreateUser {
                    username: Username::new(&state.username.read()).unwrap(),
                    password: Password::new(&state.password.read()).unwrap(),
                }
            };
            drop(state);

            let response = fetch_json!(<CreateUserOk>, api_client, request_data);
            match response {
                Ok(res) => {
                    crate::util::cookie::set_session(
                        res.session_signature,
                        res.session_id,
                        res.session_expires,
                    );
                    local_profile.write().user_id = Some(res.user_id);
                    nav.push(page::HOME)
                }
                Err(_e) => (),
            }
        });
    };

    let username_oninput = move |ev: FormEvent| {
        let value = ev.value();
        if let Err(e) = uchat_domain::Username::new(&value) {
            page_state.read().form_errors.write().set("bad-username", e.formatted_error());
        } else {
            page_state.read().form_errors.write().remove("bad-username");
        }
        page_state.read().username.set(value);
    };

    let password_oninput = move |ev: FormEvent| {
        let value = ev.value();
        if let Err(e) = uchat_domain::Password::new(&value) {
            page_state.read().form_errors.write().set("bad-password", e.formatted_error());
        } else {
            page_state.read().form_errors.write().remove("bad-password");
        }
        page_state.read().password.set(value);
    };

    let state = page_state.read();
    let can_submit = state.can_submit();
    let submit_btn_style = maybe_class!("btn-disabled", !can_submit);
    let form_errors = state.form_errors.read().clone();
    let username_signal = state.username;
    let password_signal = state.password;
    drop(state);

    rsx! {
        form {
            class: "flex flex-col gap-5",
            prevent_default: "onsubmit",
            onsubmit: form_onsubmit,

            img {
                src: "/static/icons/uchat.jpg",
                alt: "Logo",
                class: "mx-auto mb-4",
            }

            UsernameInput {
                state: username_signal,
                oninput: username_oninput,
            }

            PasswordInput {
                state: password_signal,
                oninput: password_oninput,
            }
            
            LoginLink {}
            
            KeyedNotificationBox {
                legend: Some("Form Errors".to_string()),
                notifications: form_errors,
            }

            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: !can_submit,
                "Signup"
            }
        }
    }
}
