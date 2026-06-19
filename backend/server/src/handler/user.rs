use axum::{async_trait, Json};
use chrono::{Duration, Utc};
use hyper::StatusCode;
use tracing::info;
use uchat_domain::{ids::*, user::DisplayName};
use uchat_endpoint::{
    user::{
        endpoint::{
            CreateUser, CreateUserOk, FollowUser, FollowUserOk, GetMyProfile, GetMyProfileOk,
            Login, LoginOk, UpdateProfile, UpdateProfileOk, ViewProfile, ViewProfileOk,
            ForgotPassword, ForgotPasswordOk,
        },
        types::{FollowAction, PublicUserProfile},
    },
    RequestFailed, Update,
};
use uchat_query::{
    session::Session,
    user::{UpdateProfileParams, User},
};
use url::Url;

use crate::{
    error::{ApiError, ApiResult, ServerError},
    extractor::{DbConnection, UserSession},
    AppState,
};

use super::{save_image, AuthorizedApiRequest, PublicApiRequest};

pub fn profile_id_to_url(id: &str) -> Url {
    use uchat_endpoint::app_url::{self, user_content};
    app_url::domain_and(user_content::ROOT)
        .join(user_content::IMAGES)
        .unwrap()
        .join(id)
        .unwrap()
}

#[derive(Clone)]
pub struct SessionSignature(String);

pub fn to_public(
    conn: &mut uchat_query::AsyncConnection,
    session: Option<&UserSession>,
    user: User,
) -> ApiResult<PublicUserProfile> {
    let am_following = match session {
        Some(session) => uchat_query::user::is_following(conn, session.user_id, user.id)?,
        None => false,
    };
    Ok(PublicUserProfile {
        id: user.id,
        display_name: user
            .display_name
            .and_then(|name| DisplayName::new(name).ok()),
        handle: user.handle,
        profile_image: user.profile_image.as_ref().map(|id| profile_id_to_url(id)),
        created_at: user.created_at,
        am_following,
        last_seen: user.last_seen,
    })
}

fn new_session(
    state: &AppState,
    conn: &mut uchat_query::AsyncConnection,
    user_id: UserId,
) -> ApiResult<(Session, SessionSignature, Duration)> {
    let fingerprint = serde_json::json!({});
    let session_duration = Duration::weeks(3);
    let session = uchat_query::session::new(conn, user_id, session_duration, fingerprint.into())?;

    let mut rng = state.rng.clone();
    let signature = state
        .signing_keys
        .sign(&mut rng, session.id.as_uuid().as_bytes());

    let signature = uchat_crypto::encode_base64(signature);
    Ok((session, SessionSignature(signature), session_duration))
}

#[async_trait]
impl PublicApiRequest for CreateUser {
    type Response = (StatusCode, Json<CreateUserOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        state: AppState,
    ) -> ApiResult<Self::Response> {
        let password_hash = uchat_crypto::hash_password(&self.password)?;
        let user_id = uchat_query::user::new(&mut conn, password_hash, &self.username)
            .map_err(|_| ServerError::account_exists())?;

        info!(username = self.username.as_ref(), "new user created");

        let (session, signature, duration) = new_session(&state, &mut conn, user_id)?;

        Ok((
            StatusCode::CREATED,
            Json(CreateUserOk {
                user_id,
                username: self.username,
                session_signature: signature.0,
                session_id: session.id,
                session_expires: Utc::now() + duration,
            }),
        ))
    }
}

#[async_trait]
impl PublicApiRequest for Login {
    type Response = (StatusCode, Json<LoginOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        state: AppState,
    ) -> ApiResult<Self::Response> {
        let _span = tracing::span!(tracing::Level::INFO, "logging in",
                user = %self.username.as_ref())
        .entered();

        let hash = uchat_query::user::get_password_hash(&mut conn, &self.username)
            .map_err(|_| ServerError::wrong_password())?;

        let hash = uchat_crypto::password::deserialize_hash(&hash)
            .map_err(|_| ServerError::wrong_password())?;

        uchat_crypto::verify_password(self.password, &hash)
            .map_err(|_| ServerError::wrong_password())?;

        let user = uchat_query::user::find(&mut conn, &self.username)
            .map_err(|_| ServerError::missing_login())?;

        let (session, signature, duration) = new_session(&state, &mut conn, user.id)?;

        let profile_image_url = user.profile_image.as_ref().map(|id| profile_id_to_url(id));
        let unread_notifications = uchat_query::notification::get_unread_count(&mut conn, user.id).unwrap_or(0);

        Ok((
            StatusCode::OK,
            Json(LoginOk {
                session_id: session.id,
                session_expires: Utc::now() + duration,
                session_signature: signature.0,
                display_name: user.display_name,
                email: user.email,
                profile_image: profile_image_url,
                user_id: user.id,
                unread_notifications,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for GetMyProfile {
    type Response = (StatusCode, Json<GetMyProfileOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let _ = uchat_query::user::update_last_seen(&mut conn, session.user_id);
        
        let user = uchat_query::user::get(&mut conn, session.user_id)?;
        let unread_notifications = uchat_query::notification::get_unread_count(&mut conn, session.user_id).unwrap_or(0);

        let profile_image_url = user.profile_image.as_ref().map(|id| profile_id_to_url(id));

        Ok((
            StatusCode::OK,
            Json(GetMyProfileOk {
                display_name: user.display_name,
                email: user.email,
                profile_image: profile_image_url,
                user_id: user.id,
                unread_notifications,
                security_question: user.security_question,
                security_answer: user.security_answer,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for UpdateProfile {
    type Response = (StatusCode, Json<UpdateProfileOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let mut payload = self;
        let password = {
            if let Update::Change(ref password) = payload.password {
                Update::Change(uchat_crypto::hash_password(password)?)
            } else {
                Update::NoChange
            }
        };

        if let Update::Change(ref img) = payload.profile_image {
            let id = ImageId::new();
            save_image(id, img).await?;
            payload.profile_image = Update::Change(id.to_string());
        }

        let query_params = UpdateProfileParams {
            id: session.user_id,
            display_name: payload.display_name,
            email: payload.email,
            password_hash: password,
            profile_image: payload.profile_image.clone(),
            security_question: payload.security_question,
            security_answer: payload.security_answer,
        };

        uchat_query::user::update_profile(&mut conn, query_params)?;

        let profile_image_url = {
            let user = uchat_query::user::get(&mut conn, session.user_id)?;
            user.profile_image.as_ref().map(|id| profile_id_to_url(id))
        };

        Ok((
            StatusCode::OK,
            Json(UpdateProfileOk {
                profile_image: profile_image_url,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for ViewProfile {
    type Response = (StatusCode, Json<ViewProfileOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let profile = uchat_query::user::get(&mut conn, self.for_user)?;
        let profile = to_public(&mut conn, Some(&session), profile)?;

        let mut posts = vec![];

        for post in uchat_query::post::get_public_posts(&mut conn, self.for_user)? {
            let post_id = post.id;
            match super::post::to_public(&mut conn, post, Some(&session)) {
                Ok(post) => posts.push(post),
                Err(e) => {
                    tracing::error!(err = %e.err, post_id = ?post_id, "post contains invalid data");
                }
            }
        }

        Ok((StatusCode::OK, Json(ViewProfileOk { profile, posts })))
    }
}

#[async_trait]
impl AuthorizedApiRequest for FollowUser {
    type Response = (StatusCode, Json<FollowUserOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        if self.user_id == session.user_id {
            return Err(ApiError {
                code: Some(StatusCode::BAD_REQUEST),
                err: color_eyre::Report::new(RequestFailed {
                    msg: "cannot follow self".to_string(),
                }),
            });
        }
        match self.action {
            FollowAction::Follow => {
                uchat_query::user::follow(&mut conn, session.user_id, self.user_id)?;
            }
            FollowAction::Unfollow => {
                uchat_query::user::unfollow(&mut conn, session.user_id, self.user_id)?;
            }
        }

        Ok((
            StatusCode::OK,
            Json(FollowUserOk {
                status: self.action,
            }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for uchat_endpoint::user::endpoint::GetNotifications {
    type Response = (StatusCode, Json<uchat_endpoint::user::endpoint::GetNotificationsOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let notifications = uchat_query::notification::get_notifications(&mut conn, session.user_id)?
            .into_iter()
            .map(|(n, u)| {
                let kind = match n.kind {
                    1 => uchat_endpoint::user::types::NotificationKind::Follow,
                    2 => uchat_endpoint::user::types::NotificationKind::Unfollow,
                    3 => uchat_endpoint::user::types::NotificationKind::Comment,
                    4 => uchat_endpoint::user::types::NotificationKind::Reaction,
                    _ => uchat_endpoint::user::types::NotificationKind::DirectMessage,
                };
                uchat_endpoint::user::types::Notification {
                    id: n.id,
                    user_id: n.user_id.into(),
                    actor_id: n.actor_id.into(),
                    actor_handle: u.handle,
                    actor_name: u.display_name,
                    kind,
                    post_id: n.post_id.map(|id| id.into()),
                    is_read: n.is_read,
                    created_at: n.created_at,
                }
            })
            .collect();

        Ok((
            StatusCode::OK,
            Json(uchat_endpoint::user::endpoint::GetNotificationsOk { notifications }),
        ))
    }
}

#[async_trait]
impl AuthorizedApiRequest for uchat_endpoint::user::endpoint::MarkNotificationsAsRead {
    type Response = (StatusCode, Json<uchat_endpoint::user::endpoint::MarkNotificationsAsReadOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        uchat_query::notification::mark_all_as_read(&mut conn, session.user_id)?;

        Ok((
            StatusCode::OK,
            Json(uchat_endpoint::user::endpoint::MarkNotificationsAsReadOk),
        ))
    }
}

#[async_trait]
impl PublicApiRequest for ForgotPassword {
    type Response = (StatusCode, Json<ForgotPasswordOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let user = uchat_query::user::find(&mut conn, &self.username)
            .map_err(|_| ServerError::missing_login())?;

        if let Some(ref db_answer) = user.security_answer {
            if db_answer != &self.security_answer {
                return Err(ApiError {
                    code: Some(StatusCode::BAD_REQUEST),
                    err: color_eyre::Report::msg("Incorrect security answer"),
                });
            }
        } else {
            return Err(ApiError {
                code: Some(StatusCode::BAD_REQUEST),
                err: color_eyre::Report::msg("Security question not set"),
            });
        }

        let other_user = uchat_query::user::find(&mut conn, &self.chatted_with_username)
            .map_err(|_| ApiError {
                code: Some(StatusCode::BAD_REQUEST),
                err: color_eyre::Report::msg("Chatted user not found"),
            })?;

        let has_chatted = uchat_query::chat::has_chatted(&mut conn, user.id, other_user.id)?;
        if !has_chatted {
            return Err(ApiError {
                code: Some(StatusCode::BAD_REQUEST),
                err: color_eyre::Report::msg("No chat history with this user"),
            });
        }

        let new_hash = uchat_crypto::hash_password(&self.new_password)?;
        let query_params = UpdateProfileParams {
            id: user.id,
            display_name: Update::NoChange,
            email: Update::NoChange,
            password_hash: Update::Change(new_hash),
            profile_image: Update::NoChange,
            security_question: Update::NoChange,
            security_answer: Update::NoChange,
        };
        uchat_query::user::update_profile(&mut conn, query_params)?;

        Ok((
            StatusCode::OK,
            Json(ForgotPasswordOk),
        ))
    }
}
