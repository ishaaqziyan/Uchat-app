use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{schema, DieselError};
use uchat_domain::ids::{UserId, DirectMessageId};

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = schema::direct_messages)]
pub struct DirectMessageRow {
    pub id: Uuid,
    pub sender_id: Uuid,
    pub receiver_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

pub fn send_message(
    conn: &mut PgConnection,
    sender: UserId,
    receiver: UserId,
    content: String,
) -> Result<DirectMessageId, DieselError> {
    let msg_id = DirectMessageId::new();
    
    let new_msg = DirectMessageRow {
        id: msg_id.into_inner(),
        sender_id: sender.into_inner(),
        receiver_id: receiver.into_inner(),
        content,
        created_at: Utc::now(),
    };

    diesel::insert_into(schema::direct_messages::table)
        .values(&new_msg)
        .execute(conn)?;

    let _ = crate::notification::create_notification(conn, receiver, sender, 5, None);

    Ok(msg_id)
}

pub fn get_messages(
    conn: &mut PgConnection,
    user1: UserId,
    user2: UserId,
) -> Result<Vec<DirectMessageRow>, DieselError> {
    use crate::schema::direct_messages::dsl::*;
    
    let u1 = user1.into_inner();
    let u2 = user2.into_inner();
    
    direct_messages
        .filter(
            (sender_id.eq(u1).and(receiver_id.eq(u2)))
            .or(sender_id.eq(u2).and(receiver_id.eq(u1)))
        )
        .order_by(created_at.asc())
        .get_results(conn)
}

pub fn get_conversations(
    conn: &mut PgConnection,
    user: UserId,
) -> Result<Vec<(crate::user::User, DirectMessageRow)>, DieselError> {
    // We want to find the latest message per conversation
    // A simpler way: get all messages involving the user, order by desc, and unique by the other user.
    use crate::schema::direct_messages::dsl::*;
    
    let u = user.into_inner();
    
    let msgs = direct_messages
        .filter(sender_id.eq(u).or(receiver_id.eq(u)))
        .order_by(created_at.desc())
        .get_results::<DirectMessageRow>(conn)?;
        
    let mut conversations = Vec::new();
    let mut seen = std::collections::HashSet::new();
    
    for msg in msgs {
        let other = if msg.sender_id == u { msg.receiver_id } else { msg.sender_id };
        if !seen.contains(&other) {
            seen.insert(other);
            if let Ok(other_user) = crate::user::get(conn, UserId::from(other)) {
                conversations.push((other_user, msg));
            }
        }
    }
    
    Ok(conversations)
}

pub fn has_chatted(
    conn: &mut PgConnection,
    user1: UserId,
    user2: UserId,
) -> Result<bool, DieselError> {
    use crate::schema::direct_messages::dsl::*;
    use diesel::dsl::count;
    
    let u1 = user1.into_inner();
    let u2 = user2.into_inner();
    
    direct_messages
        .filter(
            (sender_id.eq(u1).and(receiver_id.eq(u2)))
            .or(sender_id.eq(u2).and(receiver_id.eq(u1)))
        )
        .select(count(id))
        .get_result(conn)
        .optional()
        .map(|n: Option<i64>| match n {
            Some(n) => n > 0,
            None => false,
        })
}
