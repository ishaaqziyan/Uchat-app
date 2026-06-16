#![allow(non_snake_case)]

use dioxus::prelude::*;

use uchat_domain::UserFacingError;

use crate::{
    elements::{keyed_notification_box::KeyedNotifications, KeyedNotificationBox},
    fetch_json,
    prelude::*,
    util::ApiClient,
};

pub struct PageState {
    username: Signal<String>,
    password: Signal<String>,
    form_errors: KeyedNotifications,
}

impl PageState {
    pub fn new() -> Self {
        Self {
            username: use_signal(String::new).clone(),
            password: use_signal(String::new).clone(),
            form_errors: KeyedNotifications::default(),
        }
    }
    pub fn can_submit(&self) -> bool {
        !(self.form_errors.has_messages()
            || self.username.read().is_empty()
            || self.password.read().is_empty())
    }
}

#[component]
pub
fn PasswordInput(state: Signal<String>,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            label {
                r#for: "password",
                "Password",
            },
            input {
                id: "password",
                r#type: "password",
                name: "password",
                class: "input-field",
                placeholder: "Password",
                value: "{state.read()}",
                oninput: move |ev| oninput.call(ev),
            }
        }
    }
}

#[component]
pub
fn UsernameInput(state: Signal<String>,
    oninput: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        div {
            class: "flex flex-col",
            label {
                r#for: "username",
                "Username",
            },
            input {
                id: "username",
                name: "username",
                class: "input-field",
                placeholder: "User Name",
                value: "{state.read()}",
                oninput: move |ev| oninput.call(ev),
            }
        }
    }
}

#[component]
pub
fn LoginLink() -> Element {
    rsx! {
        Link {
            class: "link text-center",
            to: page::ACCOUNT_LOGIN,
            "Existing User Login"
        }
    }
}

#[component]
pub
fn Register() -> Element {
    let api_client = ApiClient::global();
    let page_state = PageState::new();
    let page_state = use_signal( || page_state);
    let router = use_navigator();
    let local_profile = use_local_profile();

    let form_onsubmit = async_handler!(
        &cx,
        [api_client, page_state, router, local_profile],
        move |_| async move {
            use uchat_endpoint::user::endpoint::{CreateUser, CreateUserOk};
            let raw_username = page_state.with(|state| state.username.read().to_string());
            let raw_password = page_state.with(|state| state.password.read().to_string());
            
            let request_data = {
                use uchat_domain::{Password, Username};
                let un = match Username::new(raw_username) {
                    Ok(u) => u,
                    Err(_) => return,
                };
                let pw = match Password::new(raw_password) {
                    Ok(p) => p,
                    Err(_) => return,
                };
                CreateUser {
                    username: un,
                    password: pw,
                }
            };
            let response = fetch_json!(<CreateUserOk>, api_client, request_data);
            match response {
                Ok(res) => {
                    crate::util::cookie::set_session(
                        res.session_signature,
                        res.session_id,
                        res.session_expires,
                    );
                    local_profile.write().user_id = Some(res.user_id);
                    { router.push(page::HOME); }
                }
                Err(_e) => (),
            }
        }
    );

    let username_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        if let Err(e) = uchat_domain::Username::new(&ev.value()) {
            page_state.with_mut(|state| state.form_errors.set("bad-username", e.formatted_error()));
        } else {
            page_state.with_mut(|state| state.form_errors.remove("bad-username"));
        }
        page_state.with_mut(|state| state.username.set(ev.value().clone()));
    });

    let password_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        if let Err(e) = uchat_domain::Password::new(&ev.value()) {
            page_state.with_mut(|state| state.form_errors.set("bad-password", e.formatted_error()));
        } else {
            page_state.with_mut(|state| state.form_errors.remove("bad-password"));
        }
        page_state.with_mut(|state| state.password.set(ev.value().clone()));
    });

    let submit_btn_style =
        maybe_class!("btn-disabled", !page_state.read().can_submit());

    rsx! {
        form {
            class: "flex flex-col gap-5",

            img {
                src: "/static/icons/uchat.png", 
                alt: "Logo",
                class: "mx-auto mb-4", 
            },

            UsernameInput {
                state: page_state.read().username.clone(),
                oninput: username_oninput,
            },

            PasswordInput {
                state: page_state.read().password.clone(),
                oninput: password_oninput,
            },
            LoginLink {},
            KeyedNotificationBox {
                legend: "Form Errors",
                notifications: page_state.read().form_errors.clone(),
            }

            button {
                class: "btn {submit_btn_style}",
                r#type: "button",
                disabled: !page_state.read().can_submit(),
                onclick: move |ev| {
                    ev.prevent_default();
                    form_onsubmit(ev);
                },
                "Signup"
            }
        }
    }
}
