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
    pub last_seen: Option<DateTime<Utc>>,
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
    DirectMessage,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct DirectMessage {
    pub id: uchat_domain::ids::DirectMessageId,
    pub sender_id: UserId,
    pub receiver_id: UserId,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Conversation {
    pub other_user_id: UserId,
    pub other_user_handle: String,
    pub other_user_name: Option<String>,
    pub other_user_image: Option<Url>,
    pub latest_message: String,
    pub updated_at: DateTime<Utc>,
}
