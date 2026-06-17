pub mod edit_profile; // Page for editing the current user's profile
pub mod home; // Home feed and its sub-pages (bookmarked, liked)
pub mod login; // Login page
pub mod new_post; // New post creation pages (chat, image, poll)
pub mod register; // Account registration page
pub mod trending; // Trending posts feed page
pub mod view_profile; // Page for viewing another user's profile

pub use edit_profile::EditProfile;
pub use home::{bookmarked::HomeBookmarked, liked::HomeLiked, Home}; // Re-export Home and its sub-pages
pub use login::Login;
pub use new_post::*; // Re-export all new post page components (NewChat, NewImage, NewPoll)
pub use register::Register;
pub use trending::Trending;
pub use view_profile::ViewProfile;
pub mod notifications;
pub use notifications::Notifications;
pub mod chat;
pub use chat::{Chat, Conversations};

pub use route::*; // Re-export all route constants and helpers at the top level

/// Route constants and URL builder helpers.
/// Centralizing all routes here prevents hardcoded URL strings scattered across the codebase.
pub mod route {
    use uchat_domain::ids::UserId;

    // Static Routes
    // These are fixed URL paths used with the router for navigation

    pub const ACCOUNT_LOGIN: &str = "/account/login";
    pub const ACCOUNT_REGISTER: &str = "/account/register";
    pub const HOME: &str = "/home";
    pub const HOME_BOOKMARKED: &str = "/home/bookmarked";
    pub const HOME_LIKED: &str = "/home/liked";
    pub const POST_NEW_CHAT: &str = "/post/new_chat";
    pub const POST_NEW_IMAGE: &str = "/post/new_image";
    pub const POST_NEW_POLL: &str = "/post/new_poll";
    pub const POSTS_TRENDING: &str = "/posts/trending";
    pub const PROFILE_EDIT: &str = "/profile/edit";
    pub const PROFILE_VIEW: &str = "/profile/view/:user";

    /// Builds a concrete profile view URL by replacing the `:user` placeholder with an actual UserId.
    pub fn profile_view(user_id: UserId) -> String {
        PROFILE_VIEW.replace(":user", &user_id.to_string())
    }
}
