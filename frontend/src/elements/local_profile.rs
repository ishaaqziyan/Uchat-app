#![allow(non_snake_case)]


use dioxus::prelude::*;

use uchat_domain::ids::UserId;

#[derive(Default)]
pub struct LocalProfile {
    pub image: Option<url::Url>,
    pub user_id: Option<UserId>,
    pub unread_notifications: i64,
}

pub fn use_local_profile() -> Signal<LocalProfile> {
    use_context::<Signal<LocalProfile>>()
}
