#![allow(non_snake_case)]

use crate::prelude::*;
use dioxus::prelude::*;
use uchat_endpoint::user::endpoint::{
    GetNotifications, GetNotificationsOk, MarkNotificationsAsRead, MarkNotificationsAsReadOk,
};
use uchat_endpoint::user::types::{Notification, NotificationKind};

#[component]
pub fn Notifications() -> Element {
    let api_client = ApiClient::global();
    let toaster = use_toaster();
    let mut notifications = use_signal(Vec::new);
    let local_profile = use_local_profile();

    use_future(move || {
        let api_client = api_client.clone();
        let mut toaster = toaster.clone();
        let mut local_profile = local_profile.clone();
        async move {
            let _ = fetch_json!(<MarkNotificationsAsReadOk>, api_client, MarkNotificationsAsRead);
            local_profile.write().unread_notifications = 0;

            match fetch_json!(<GetNotificationsOk>, api_client, GetNotifications) {
                Ok(res) => {
                    notifications.set(res.notifications);
                }
                Err(e) => {
                    toaster.write().error(
                        format!("Failed to load notifications: {}", e.to_string()),
                        chrono::Duration::seconds(3),
                    );
                }
            }
        }
    });

    rsx! {
        div { class: "flex flex-col gap-4",
            h1 { class: "text-2xl font-bold mb-4", "Notifications" }
            for notification in notifications.read().iter() {
                NotificationItem { notification: notification.clone() }
            }
            if notifications.read().is_empty() {
                div { class: "text-center text-gray-500 mt-8", "No notifications yet." }
            }
        }
    }
}

#[component]
fn NotificationItem(notification: Notification) -> Element {
    let router = use_navigator();
    let action_text = match notification.kind {
        NotificationKind::Follow => "started following you",
        NotificationKind::Unfollow => "stopped following you",
        NotificationKind::Comment => "commented on your post",
        NotificationKind::Reaction => "reacted to your post",
        NotificationKind::DirectMessage => "sent you a direct message",
    };

    let time_ago = {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(notification.created_at);
        if duration.num_minutes() < 60 {
            format!("{}m ago", duration.num_minutes())
        } else if duration.num_hours() < 24 {
            format!("{}h ago", duration.num_hours())
        } else {
            format!("{}d ago", duration.num_days())
        }
    };

    let display_name = notification
        .actor_name
        .unwrap_or_else(|| notification.actor_handle.clone());
    let handle = notification.actor_handle;
    let actor_id = notification.actor_id;

    rsx! {
        div {
            class: "flex flex-row items-center gap-3 p-4 border rounded-md cursor-pointer hover:bg-slate-50 transition-colors",
            onclick: move |_| {
                if notification.kind == NotificationKind::DirectMessage {
                    let _ = router.push(crate::app::Route::Chat { user_id: actor_id });
                } else if let Some(_post_id) = notification.post_id {
                    // Navigate to post? We don't have a view post page right now.
                    // Instead, let's navigate to the user's profile.
                    let _ = router
                        .push(crate::app::Route::ViewProfile {
                            user_id: actor_id,
                        });
                } else {
                    let _ = router
                        .push(crate::app::Route::ViewProfile {
                            user_id: actor_id,
                        });
                }
            },
            div { class: "flex-1",
                span { class: "font-bold", "{display_name} " }
                span { class: "text-gray-500", "@{handle} " }
                span { "{action_text}" }
            }
            div { class: "text-sm text-gray-400", "{time_ago}" }
        }
    }
}
