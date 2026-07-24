#![allow(non_snake_case)]
#![allow(unused_mut)]

use crate::prelude::*;
use dioxus::prelude::*;
use uchat_domain::ids::UserId;
use uchat_endpoint::user::endpoint::{
    GetConversations, GetConversationsOk, GetDirectMessages, GetDirectMessagesOk,
    SendDirectMessage, SendDirectMessageOk,
};

#[component]
pub fn Conversations() -> Element {
    let api_client = ApiClient::global();
    let mut toaster = use_toaster();
    let mut convs = use_signal(Vec::new);

    use_future(move || {
        let mut convs = convs;
        let api_client = api_client.clone();
        let mut toaster = toaster.clone();
        async move {
            match fetch_json!(<GetConversationsOk>, api_client, GetConversations) {
                Ok(res) => {
                    convs.set(res.conversations);
                }
                Err(e) => {
                    toaster.write().error(
                        format!("Failed to load conversations: {}", e.to_string()),
                        chrono::Duration::seconds(3),
                    );
                }
            }
        }
    });

    rsx! {
        div {
            class: "flex flex-col gap-4",
            h1 { class: "text-2xl font-bold mb-4", "Direct Messages" }
            for conv in convs.read().iter() {
                ConversationItem { conv: conv.clone() }
            }
            if convs.read().is_empty() {
                div {
                    class: "text-center text-gray-500 mt-8",
                    "No conversations yet."
                }
            }
        }
    }
}

#[component]
fn ConversationItem(conv: uchat_endpoint::user::types::Conversation) -> Element {
    let router = use_navigator();
    let display_name = conv
        .other_user_name
        .clone()
        .unwrap_or_else(|| conv.other_user_handle.clone());
    let handle = conv.other_user_handle.clone();
    let user_id = conv.other_user_id;
    let latest_msg = conv.latest_message.clone();

    let time_ago = {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(conv.updated_at);
        if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else {
            format!("{}d ago", duration.num_days())
        }
    };

    let profile_img_src = conv
        .other_user_image
        .as_ref()
        .map(|url| url.as_str())
        .unwrap_or_else(|| "/static/icons/uchat.png");

    rsx! {
        div {
            class: "flex flex-row items-center gap-3 p-4 border rounded-md cursor-pointer hover:bg-slate-50 transition-colors",
            onclick: move |_| {
                let _ = router.push(crate::app::Route::Chat { user_id });
            },
            img {
                class: "w-12 h-12 rounded-full object-cover",
                src: "{profile_img_src}",
            }
            div {
                class: "flex-1",
                div {
                    class: "flex flex-row justify-between",
                    div {
                        span { class: "font-bold", "{display_name} " }
                        span { class: "text-gray-500", "@{handle} " }
                    }
                    div {
                        class: "text-sm text-gray-400",
                        "{time_ago}"
                    }
                }
                div {
                    class: "text-gray-700 truncate",
                    "{latest_msg}"
                }
            }
        }
    }
}

#[component]
pub fn Chat(user_id: UserId) -> Element {
    let api_client = ApiClient::global();
    let mut toaster = use_toaster();
    let mut messages = use_signal(Vec::new);
    let mut current_msg = use_signal(String::new);
    let mut other_user_profile = use_signal(|| None::<uchat_endpoint::user::types::PublicUserProfile>);
    let local_profile = use_local_profile();
    let router = use_navigator();

    let fetch_messages = {
        let api_client = api_client.clone();
        let mut toaster = toaster.clone();
        let mut messages = messages.clone();
        let mut other_user_profile = other_user_profile.clone();
        move || {
            let api_client = api_client.clone();
            let mut toaster = toaster.clone();
            let mut messages = messages.clone();
            let mut other_user_profile = other_user_profile.clone();
            async move {
                // Fetch profile if not loaded
                if other_user_profile.read().is_none() {
                    let req = uchat_endpoint::user::endpoint::ViewProfile { for_user: user_id };
                    if let Ok(res) = fetch_json!(<uchat_endpoint::user::endpoint::ViewProfileOk>, api_client, req)
                    {
                        other_user_profile.set(Some(res.profile));
                    }
                }

                let req = GetDirectMessages {
                    other_user_id: user_id,
                };
                match fetch_json!(<GetDirectMessagesOk>, api_client, req) {
                    Ok(res) => {
                        messages.set(res.messages);
                    }
                    Err(e) => {
                        toaster.write().error(
                            format!("Failed to load messages: {}", e.to_string()),
                            chrono::Duration::seconds(3),
                        );
                    }
                }
            }
        }
    };

    use_future({
        let fetch_messages = fetch_messages.clone();
        move || {
            let fetch_messages = fetch_messages.clone();
            async move {
                loop {
                    fetch_messages().await;
                    gloo_timers::future::sleep(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    let send_msg = async_handler!(
        &cx,
        [api_client, current_msg, toaster],
        move |_| async move {
            let msg = current_msg.read().clone();
            if msg.trim().is_empty() {
                return;
            }

            let req = SendDirectMessage {
                receiver_id: user_id,
                content: msg,
            };

            match fetch_json!(<SendDirectMessageOk>, api_client, req) {
                Ok(_) => {
                    current_msg.set(String::new());
                    // Messages will be updated by the polling loop shortly
                }
                Err(e) => {
                    toaster.write().error(
                        format!("Failed to send message: {}", e.to_string()),
                        chrono::Duration::seconds(3),
                    );
                }
            }
        }
    );

    let my_id = local_profile.read().user_id.unwrap_or_default();

    let ChatHeader = {
        if let Some(profile) = other_user_profile.read().clone() {
            let display_name = profile
                .display_name
                .map(|name| name.into_inner())
                .unwrap_or_else(|| profile.handle.clone());
            let profile_img_src = profile
                .profile_image
                .map(|url| url.to_string())
                .unwrap_or_else(|| "/static/icons/uchat.png".to_string());
            let last_seen_text = if let Some(last_seen) = profile.last_seen {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(last_seen);
                if duration.num_minutes() < 1 {
                    "Online".to_string()
                } else if duration.num_minutes() < 60 {
                    format!("Last seen {}m ago", duration.num_minutes())
                } else if duration.num_hours() < 24 {
                    format!("Last seen {}h ago", duration.num_hours())
                } else {
                    format!("Last seen {}d ago", duration.num_days())
                }
            } else {
                "Last seen recently".to_string()
            };

            rsx! {
                div {
                    class: "flex flex-row justify-between items-center p-3 border-b bg-white/50 backdrop-blur-sm dark:bg-slate-900/50 rounded-t-md shrink-0 w-full",
                    div {
                        class: "flex flex-row items-center gap-3 shrink-0",
                        img { class: "w-10 h-10 rounded-full object-cover shrink-0", src: "{profile_img_src}" }
                        div {
                            class: "flex flex-col overflow-hidden",
                            span { class: "font-bold truncate", "{display_name}" }
                            span { class: "text-xs text-gray-500 truncate", "{last_seen_text}" }
                        }
                    }
                    button {
                        class: "btn bg-gray-200 text-gray-800 px-4 py-2 text-sm rounded shadow-sm hover:bg-gray-300 transition-colors shrink-0 ml-2 whitespace-nowrap",
                        onclick: move |_| {
                            let _ = router.push(crate::app::Route::Home {});
                        },
                        "Home"
                    }
                }
            }
        } else {
            rsx! {
                div { class: "p-3 border-b bg-white/50 backdrop-blur-sm dark:bg-slate-900/50 rounded-t-md text-center text-gray-500", "Loading chat..." }
            }
        }
    };

    rsx! {
        div {
            class: "flex flex-col h-[calc(100vh-var(--navbar-height)-120px)]",
            {ChatHeader}
            div {
                class: "flex-1 overflow-y-auto flex flex-col gap-2 p-2 border-x bg-white/50 backdrop-blur-sm dark:bg-slate-900/50",
                for msg in messages.read().iter() {
                    if msg.sender_id == my_id {
                        div {
                            class: "self-end bg-blue-500 text-white p-3 rounded-lg max-w-[70%]",
                            "{msg.content}"
                        }
                    } else {
                        div {
                            class: "self-start bg-gray-200 text-gray-800 p-3 rounded-lg max-w-[70%]",
                            "{msg.content}"
                        }
                    }
                }
            }
            div {
                class: "flex flex-row p-2 border rounded-b-md bg-white/50 backdrop-blur-sm dark:bg-slate-900/50 gap-2",
                input {
                    class: "flex-1 input-field",
                    placeholder: "Type a message...",
                    value: "{current_msg.read()}",
                    oninput: move |ev| current_msg.set(ev.value().clone()),
                    onkeydown: move |ev| {
                        if ev.key() == dioxus_elements::input_data::keyboard_types::Key::Enter {
                            send_msg(());
                        }
                    }
                }
                button {
                    class: "btn bg-blue-500 text-white px-4",
                    onclick: move |_| send_msg(()),
                    "Send"
                }
            }
        }
    }
}
