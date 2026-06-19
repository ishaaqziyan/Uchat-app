#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{
    elements::{keyed_notification_box::KeyedNotifications, KeyedNotificationBox},
    fetch_json,
    prelude::*,
    util::ApiClient,
};

#[derive(Clone, Default)]
pub struct PageState {
    username: String,
    chatted_with_username: String,
    security_answer: String,
    new_password: String,
    new_password_confirm: String,
    form_errors: KeyedNotifications,
    server_messages: KeyedNotifications,
}

impl PageState {
    pub fn can_submit(&self) -> bool {
        !(self.form_errors.has_messages()
            || self.username.is_empty()
            || self.chatted_with_username.is_empty()
            || self.security_answer.is_empty()
            || self.new_password.is_empty()
            || self.new_password_confirm.is_empty())
    }
}

#[component]
pub fn ForgotPassword() -> Element {
    let api_client = ApiClient::global();
    let mut page_state = use_signal(PageState::default);
    let router = use_navigator();
    let toaster = use_toaster();

    let form_onsubmit = async_handler!(
        &cx,
        [api_client, page_state, router, toaster],
        move |_| async move {
            use uchat_endpoint::user::endpoint::{ForgotPassword, ForgotPasswordOk};
            
            let request_data = {
                use uchat_domain::{Password, Username};
                let state = page_state.read();
                ForgotPassword {
                    username: Username::new(state.username.clone()).unwrap(),
                    security_answer: state.security_answer.clone(),
                    chatted_with_username: Username::new(state.chatted_with_username.clone()).unwrap(),
                    new_password: Password::new(state.new_password.clone()).unwrap(),
                }
            };
            let response = fetch_json!(<ForgotPasswordOk>, api_client, request_data);
            match response {
                Ok(_) => {
                    toaster.write().success("Password successfully reset!", chrono::Duration::seconds(5));
                    router.push(crate::app::Route::Login {});
                }
                Err(e) => page_state
                    .with_mut(|state| state.server_messages.set("reset-fail", e.to_string())),
            }
        }
    );

    let disable_submit = !page_state.read().can_submit();
    let submit_btn_style = maybe_class!("btn-disabled", disable_submit);

    rsx! {
        form {
            class: "flex flex-col gap-5",
            onsubmit: move |ev| {
                ev.prevent_default();
                if page_state.read().can_submit() {
                    form_onsubmit(ev);
                }
            },

            KeyedNotificationBox {
                legend: "Reset Errors",
                notifications: page_state.read().server_messages.clone(),
            },

            h2 { class: "text-2xl font-bold mx-auto mb-4", "Reset Password" },

            div {
                class: "flex flex-col",
                label { r#for: "username", "Your Username" },
                input {
                    class: "input-field",
                    id: "username",
                    value: "{page_state.read().username}",
                    oninput: move |ev| page_state.with_mut(|state| state.username = ev.value().clone()),
                }
            }

            div {
                class: "flex flex-col",
                label { r#for: "chatted", "Username of a person you have chatted with" },
                input {
                    class: "input-field",
                    id: "chatted",
                    value: "{page_state.read().chatted_with_username}",
                    oninput: move |ev| page_state.with_mut(|state| state.chatted_with_username = ev.value().clone()),
                }
            }

            div {
                class: "flex flex-col",
                label { r#for: "sec-answer", "Answer to your Security Question" },
                input {
                    class: "input-field",
                    id: "sec-answer",
                    value: "{page_state.read().security_answer}",
                    oninput: move |ev| page_state.with_mut(|state| state.security_answer = ev.value().clone()),
                }
            }

            div {
                class: "flex flex-col",
                label { r#for: "new-pw", "New Password" },
                input {
                    class: "input-field",
                    type: "password",
                    id: "new-pw",
                    value: "{page_state.read().new_password}",
                    oninput: move |ev| page_state.with_mut(|state| state.new_password = ev.value().clone()),
                }
            }

            div {
                class: "flex flex-col",
                label { r#for: "confirm-pw", "Confirm New Password" },
                input {
                    class: "input-field",
                    type: "password",
                    id: "confirm-pw",
                    value: "{page_state.read().new_password_confirm}",
                    oninput: move |ev| {
                        let val = ev.value().clone();
                        page_state.with_mut(|state| {
                            state.new_password_confirm = val.clone();
                            if val != state.new_password {
                                state.form_errors.set("pw-match", "Passwords do not match".to_string());
                            } else {
                                state.form_errors.remove("pw-match");
                            }
                        });
                    },
                }
            }

            Link {
                class: "link text-center",
                to: crate::app::Route::Login {},
                "Back to Login"
            }

            KeyedNotificationBox {
                legend: "Form Errors",
                notifications: page_state.read().form_errors.clone(),
            }

            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: disable_submit,
                "Reset Password"
            }
        }
    }
}
