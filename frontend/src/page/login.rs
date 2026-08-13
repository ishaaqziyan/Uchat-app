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
    magic_email: Signal<String>,
    form_errors: KeyedNotifications,
    server_messages: KeyedNotifications,
}

impl PageState {
    pub fn new() -> Self {
        Self {
            username: use_signal(String::new).clone(),
            password: use_signal(String::new).clone(),
            magic_email: use_signal(String::new).clone(),
            form_errors: KeyedNotifications::default(),
            server_messages: KeyedNotifications::default(),
        }
    }
    pub fn can_submit(&self) -> bool {
        !(self.form_errors.has_messages()
            || self.username.read().is_empty()
            || self.password.read().is_empty())
    }
}

#[component]
pub fn PasswordInput(state: Signal<String>, oninput: EventHandler<FormEvent>) -> Element {
    let mut show_password = use_signal(|| false);

    rsx! {
        div {
            class: "flex flex-col",
            label {
                r#for: "password",
                "Password",
            },
            div {
                class: "relative flex flex-row",
                input {
                    id: "password",
                    r#type: if *show_password.read() { "text" } else { "password" },
                    name: "password",
                    class: "input-field w-full",
                    placeholder: "Password",
                    value: "{state.read()}",
                    oninput: move |ev| oninput.call(ev),
                }
                button {
                    r#type: "button",
                    class: "absolute right-3 top-1/2 -translate-y-1/2 text-sm text-gray-400 hover:text-white",
                    onclick: move |_| {
                        let current = *show_password.read();
                        show_password.set(!current);
                    },
                    if *show_password.read() {
                        "Hide"
                    } else {
                        "Show"
                    }
                }
            }
        }
    }
}

#[component]
pub fn UsernameInput(state: Signal<String>, oninput: EventHandler<FormEvent>) -> Element {
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
pub fn RegisterLink() -> Element {
    rsx! {
        Link {
            class: "link text-center",
            to: page::ACCOUNT_REGISTER,
            "Create Account"
        }
    }
}
#[component]
pub fn Login() -> Element {
    let api_client = ApiClient::global();
    let page_state = PageState::new();
    let page_state = use_signal(|| page_state);
    let router = use_navigator();
    let local_profile = use_local_profile();

    let form_onsubmit = async_handler!(
        &cx,
        [api_client, page_state, router, local_profile],
        move |_| async move {
            use uchat_endpoint::user::endpoint::{Login, LoginOk};
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
                Login {
                    username: un,
                    password: pw,
                }
            };
            let response = fetch_json!(<LoginOk>, api_client, request_data);
            match response {
                Ok(res) => {
                    crate::util::cookie::set_session(
                        res.session_signature,
                        res.session_id,
                        res.session_expires,
                    );
                    local_profile.write().image = res.profile_image;
                    local_profile.write().user_id = Some(res.user_id);
                    local_profile.write().unread_notifications = res.unread_notifications;
                    {
                        router.push(page::HOME);
                    }
                }
                Err(e) => page_state
                    .with_mut(|state| state.server_messages.set("login-fail", e.to_string())),
            }
        }
    );

    let wallet_onclick = async_handler!(
        &cx,
        [api_client, page_state, router, local_profile],
        move |_| async move {
            use uchat_domain::EthAddress;
            use uchat_endpoint::user::endpoint::{
                LoginOk, WalletLogin, WalletNonceRequest, WalletNonceRequestOk,
            };

            let address = match crate::util::wallet::connect().await {
                Ok(address) => address,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("wallet", e.to_string()));
                    return;
                }
            };

            let eth_address = match EthAddress::new(address.clone()) {
                Ok(eth_address) => eth_address,
                Err(_) => {
                    page_state.with_mut(|state| {
                        state
                            .server_messages
                            .set("wallet", "Wallet returned an invalid address")
                    });
                    return;
                }
            };

            let nonce_request = WalletNonceRequest {
                address: eth_address.clone(),
            };
            let message = match fetch_json!(<WalletNonceRequestOk>, api_client, nonce_request) {
                Ok(res) => res.message,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("wallet", e.to_string()));
                    return;
                }
            };

            let signature = match crate::util::wallet::sign(&address, &message).await {
                Ok(signature) => signature,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("wallet", e.to_string()));
                    return;
                }
            };

            let wallet_login = WalletLogin {
                address: eth_address,
                message,
                signature,
            };
            let response = fetch_json!(<LoginOk>, api_client, wallet_login);
            match response {
                Ok(res) => {
                    crate::util::cookie::set_session(
                        res.session_signature,
                        res.session_id,
                        res.session_expires,
                    );
                    local_profile.write().image = res.profile_image;
                    local_profile.write().user_id = Some(res.user_id);
                    local_profile.write().unread_notifications = res.unread_notifications;
                    {
                        router.push(page::HOME);
                    }
                }
                Err(e) => page_state
                    .with_mut(|state| state.server_messages.set("wallet", e.to_string())),
            }
        }
    );

    let solana_wallet_onclick = async_handler!(
        &cx,
        [api_client, page_state, router, local_profile],
        move |_| async move {
            use uchat_domain::SolanaAddress;
            use uchat_endpoint::user::endpoint::{
                LoginOk, SolanaWalletLogin, SolanaWalletNonceRequest, SolanaWalletNonceRequestOk,
            };

            let address = match crate::util::solana::connect().await {
                Ok(address) => address,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("wallet", e.to_string()));
                    return;
                }
            };

            let solana_address = match SolanaAddress::new(address.clone()) {
                Ok(solana_address) => solana_address,
                Err(_) => {
                    page_state.with_mut(|state| {
                        state
                            .server_messages
                            .set("wallet", "Wallet returned an invalid address")
                    });
                    return;
                }
            };

            let nonce_request = SolanaWalletNonceRequest {
                address: solana_address.clone(),
            };
            let message = match fetch_json!(<SolanaWalletNonceRequestOk>, api_client, nonce_request)
            {
                Ok(res) => res.message,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("wallet", e.to_string()));
                    return;
                }
            };

            let signature = match crate::util::solana::sign(&message).await {
                Ok(signature) => signature,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("wallet", e.to_string()));
                    return;
                }
            };

            let wallet_login = SolanaWalletLogin {
                address: solana_address,
                message,
                signature,
            };
            let response = fetch_json!(<LoginOk>, api_client, wallet_login);
            match response {
                Ok(res) => {
                    crate::util::cookie::set_session(
                        res.session_signature,
                        res.session_id,
                        res.session_expires,
                    );
                    local_profile.write().image = res.profile_image;
                    local_profile.write().user_id = Some(res.user_id);
                    local_profile.write().unread_notifications = res.unread_notifications;
                    {
                        router.push(page::HOME);
                    }
                }
                Err(e) => page_state
                    .with_mut(|state| state.server_messages.set("wallet", e.to_string())),
            }
        }
    );

    let magic_onclick = async_handler!(
        &cx,
        [api_client, page_state, router, local_profile],
        move |_| async move {
            use uchat_domain::SolanaAddress;
            use uchat_endpoint::user::endpoint::{
                LoginOk, SolanaWalletLogin, SolanaWalletNonceRequest, SolanaWalletNonceRequestOk,
            };

            let email = page_state.read().magic_email.read().clone();

            let address = match crate::util::magic::login_with_email(&email).await {
                Ok(address) => address,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("magic", e.to_string()));
                    return;
                }
            };

            let solana_address = match SolanaAddress::new(address.clone()) {
                Ok(solana_address) => solana_address,
                Err(_) => {
                    page_state.with_mut(|state| {
                        state
                            .server_messages
                            .set("magic", "Magic returned an invalid address")
                    });
                    return;
                }
            };

            let nonce_request = SolanaWalletNonceRequest {
                address: solana_address.clone(),
            };
            let message = match fetch_json!(<SolanaWalletNonceRequestOk>, api_client, nonce_request)
            {
                Ok(res) => res.message,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("magic", e.to_string()));
                    return;
                }
            };

            let signature = match crate::util::magic::sign(&message).await {
                Ok(signature) => signature,
                Err(e) => {
                    page_state
                        .with_mut(|state| state.server_messages.set("magic", e.to_string()));
                    return;
                }
            };

            let wallet_login = SolanaWalletLogin {
                address: solana_address,
                message,
                signature,
            };
            let response = fetch_json!(<LoginOk>, api_client, wallet_login);
            match response {
                Ok(res) => {
                    crate::util::cookie::set_session(
                        res.session_signature,
                        res.session_id,
                        res.session_expires,
                    );
                    local_profile.write().image = res.profile_image;
                    local_profile.write().user_id = Some(res.user_id);
                    local_profile.write().unread_notifications = res.unread_notifications;
                    {
                        router.push(page::HOME);
                    }
                }
                Err(e) => page_state
                    .with_mut(|state| state.server_messages.set("magic", e.to_string())),
            }
        }
    );

    let magic_email_oninput = sync_handler!([page_state], move |ev: FormEvent| {
        page_state.with_mut(|state| state.magic_email.set(ev.value().clone()));
    });

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

    let submit_btn_style = maybe_class!("btn-disabled", !page_state.read().can_submit());

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
                legend: "Login Errors",
                notifications: page_state.read().server_messages.clone(),
            },

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

            button {
                class: "btn",
                r#type: "button",
                onclick: wallet_onclick,
                "Connect Wallet"
            }

            button {
                class: "btn",
                r#type: "button",
                onclick: solana_wallet_onclick,
                "Connect Solana Wallet"
            }

            input {
                r#type: "email",
                class: "input",
                placeholder: "Email (Magic sign-in)",
                value: "{page_state.read().magic_email}",
                oninput: magic_email_oninput,
            },

            button {
                class: "btn",
                r#type: "button",
                onclick: magic_onclick,
                "Sign in with Email"
            }

            RegisterLink {},

            Link {
                class: "link text-center text-sm",
                to: page::ACCOUNT_FORGOT_PASSWORD,
                "Forgot Password?"
            }

            KeyedNotificationBox {
                legend: "Form Errors",
                notifications: page_state.read().form_errors.clone(),
            }

            button {
                class: "btn {submit_btn_style}",
                r#type: "submit",
                disabled: !page_state.read().can_submit(),
                "Login"
            }
        }
    }
}
