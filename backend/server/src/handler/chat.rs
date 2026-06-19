use axum::{async_trait, Json};
use hyper::StatusCode;
use uchat_endpoint::user::endpoint::{
    GetConversations, GetConversationsOk, GetDirectMessages, GetDirectMessagesOk,
    SendDirectMessage, SendDirectMessageOk,
};

use crate::{
    error::ApiResult,
    extractor::{DbConnection, UserSession},
    AppState,
};

#[async_trait]
impl crate::handler::AuthorizedApiRequest for SendDirectMessage {
    type Response = (StatusCode, Json<SendDirectMessageOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let message_id = uchat_query::chat::send_message(
            &mut conn,
            session.user_id,
            self.receiver_id,
            self.content,
        )?;

        Ok((StatusCode::OK, Json(SendDirectMessageOk { message_id })))
    }
}

#[async_trait]
impl crate::handler::AuthorizedApiRequest for GetConversations {
    type Response = (StatusCode, Json<GetConversationsOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let convs = uchat_query::chat::get_conversations(&mut conn, session.user_id)?;

        let conversations = convs
            .into_iter()
            .map(|(user, msg)| uchat_endpoint::user::types::Conversation {
                other_user_id: user.id,
                other_user_handle: user.handle,
                other_user_name: user.display_name,
                other_user_image: user.profile_image.map(|id| crate::handler::user::profile_id_to_url(&id)),
                latest_message: msg.content,
                updated_at: msg.created_at,
            })
            .collect();

        Ok((StatusCode::OK, Json(GetConversationsOk { conversations })))
    }
}

#[async_trait]
impl crate::handler::AuthorizedApiRequest for GetDirectMessages {
    type Response = (StatusCode, Json<GetDirectMessagesOk>);
    async fn process_request(
        self,
        DbConnection(mut conn): DbConnection,
        session: UserSession,
        _state: AppState,
    ) -> ApiResult<Self::Response> {
        let msgs = uchat_query::chat::get_messages(&mut conn, session.user_id, self.other_user_id)?;

        let messages = msgs
            .into_iter()
            .map(|m| uchat_endpoint::user::types::DirectMessage {
                id: m.id.into(),
                sender_id: m.sender_id.into(),
                receiver_id: m.receiver_id.into(),
                content: m.content,
                created_at: m.created_at,
            })
            .collect();

        Ok((StatusCode::OK, Json(GetDirectMessagesOk { messages })))
    }
}
