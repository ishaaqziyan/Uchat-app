#![allow(non_snake_case)]

use dioxus::prelude::*;
use uchat_domain::ids::UserId;

#[derive(Default, Clone)]
pub struct LocalProfile {
    pub image: Option<url::Url>,
    pub user_id: Option<UserId>,
}

pub fn use_local_profile() -> Signal<LocalProfile> {
    *crate::app::LOCAL_PROFILE
}
