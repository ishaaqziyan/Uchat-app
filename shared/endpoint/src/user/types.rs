use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uchat_domain::{ids::UserId, user::DisplayName};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PublicUserProfile {
    pub id: UserId,
    pub display_name: Option<DisplayName>,
    pub handle: String,
    pub profile_image: Option<Url>,
    pub created_at: DateTime<Utc>,
    pub am_following: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub enum FollowAction {
    Follow,
    Unfollow,
}

impl From<FollowAction> for bool {
    fn from(value: FollowAction) -> Self {
        match value {
            FollowAction::Follow => true,
            FollowAction::Unfollow => false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Notification {
    pub id: uuid::Uuid,
    pub user_id: UserId,
    pub actor_id: UserId,
    pub actor_handle: String,
    pub actor_name: Option<String>,
    pub kind: NotificationKind,
    pub post_id: Option<uchat_domain::ids::PostId>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum NotificationKind {
    Follow,
    Unfollow,
    Comment,
    Reaction,
}
