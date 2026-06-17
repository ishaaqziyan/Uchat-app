use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{schema, DieselError};
use uchat_domain::ids::{UserId, PostId};

#[derive(Debug, Queryable, Insertable)]
#[diesel(table_name = schema::notifications)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub actor_id: Uuid,
    pub kind: i16,
    pub post_id: Option<Uuid>,
    pub is_read: bool,
    pub created_at: DateTime<Utc>,
}

pub fn get_notifications(
    conn: &mut PgConnection,
    _user_id: UserId,
) -> Result<Vec<(Notification, crate::user::User)>, DieselError> {
    use crate::schema::notifications::dsl::*;
    use crate::schema::users;

    notifications
        .filter(crate::schema::notifications::user_id.eq(user_id))
        .inner_join(users::table.on(actor_id.eq(users::id)))
        .order_by(created_at.desc())
        .limit(30)
        .get_results::<(Notification, crate::user::User)>(conn)
}

pub fn create_notification(
    conn: &mut PgConnection,
    user_id: UserId,
    actor_id: UserId,
    kind: i16,
    post_id: Option<PostId>,
) -> Result<(), DieselError> {
    if user_id == actor_id {
        return Ok(());
    }

    let new_notification = Notification {
        id: Uuid::new_v4(),
        user_id: user_id.into_inner(),
        actor_id: actor_id.into_inner(),
        kind,
        post_id: post_id.map(|id| id.into_inner()),
        is_read: false,
        created_at: Utc::now(),
    };

    diesel::insert_into(crate::schema::notifications::table)
        .values(&new_notification)
        .execute(conn)?;

    Ok(())
}

pub fn get_unread_count(conn: &mut PgConnection, uid: UserId) -> Result<i64, DieselError> {
    let uid_uuid = uid.into_inner();
    use crate::schema::notifications::dsl::*;
    
    notifications
        .filter(user_id.eq(uid_uuid))
        .filter(is_read.eq(false))
        .count()
        .get_result(conn)
}

pub fn mark_all_as_read(conn: &mut PgConnection, uid: UserId) -> Result<(), DieselError> {
    let uid_uuid = uid.into_inner();
    use crate::schema::notifications::dsl::*;
    
    diesel::update(notifications)
        .filter(user_id.eq(uid_uuid))
        .filter(is_read.eq(false))
        .set(is_read.eq(true))
        .execute(conn)?;
        
    Ok(())
}
