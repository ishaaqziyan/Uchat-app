use chrono::DateTime;
use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;
use password_hash::PasswordHashString;
use uchat_domain::ids::UserId;
use uchat_domain::Username;
use uchat_endpoint::Update;

use crate::post::DeleteStatus;
use crate::{DieselError, QueryError};

pub fn new<T: AsRef<str>>(
    conn: &mut PgConnection,
    hash: PasswordHashString,
    handle: T,
) -> Result<UserId, QueryError> {
    use crate::schema::users::{self, columns};

    let user_id = UserId::new();

    diesel::insert_into(users::table)
        .values((
            columns::id.eq(user_id),
            columns::password_hash.eq(hash.as_str()),
            columns::handle.eq(handle.as_ref()),
        ))
        .execute(conn)?;

    Ok(user_id)
}

pub fn get_password_hash(
    conn: &mut PgConnection,
    username: &Username,
) -> Result<String, QueryError> {
    use crate::schema::users::dsl::*;
    Ok(users
        .filter(handle.eq(username.as_ref()))
        .select(password_hash)
        .get_result(conn)?)
}

#[derive(Debug, Queryable)]
pub struct User {
    pub id: UserId,
    pub email: Option<String>,
    pub email_confirmed: Option<DateTime<Utc>>,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub handle: String,
    pub created_at: DateTime<Utc>,
    pub profile_image: Option<String>,
    pub last_seen: Option<DateTime<Utc>>,
    pub security_question: Option<String>,
    pub security_answer: Option<String>,
}

pub fn update_last_seen(conn: &mut PgConnection, uid: UserId) -> Result<(), DieselError> {
    use crate::schema::users::dsl::*;
    
    diesel::update(users)
        .filter(id.eq(uid.into_inner()))
        .set(last_seen.eq(chrono::Utc::now()))
        .execute(conn)?;
        
    Ok(())
}

pub fn get(conn: &mut PgConnection, user_id: UserId) -> Result<User, DieselError> {
    use crate::schema::users::dsl::*;
    users.filter(id.eq(user_id)).get_result(conn)
}

pub fn find(conn: &mut PgConnection, username: &Username) -> Result<User, DieselError> {
    use crate::schema::users::dsl::*;
    users.filter(handle.eq(username.as_ref())).get_result(conn)
}

#[derive(Debug)]
pub struct UpdateProfileParams {
    pub id: UserId,
    pub display_name: Update<String>,
    pub email: Update<String>,
    pub password_hash: Update<PasswordHashString>,
    pub profile_image: Update<String>,
    pub security_question: Update<String>,
    pub security_answer: Update<String>,
}

#[derive(AsChangeset, Debug)]
#[diesel(table_name = crate::schema::users)]
struct UpdateProfileParamsInternal {
    pub display_name: Option<Option<String>>,
    pub email: Option<Option<String>>,
    pub password_hash: Option<String>,
    pub profile_image: Option<Option<String>>,
    pub security_question: Option<Option<String>>,
    pub security_answer: Option<Option<String>>,
}

pub fn update_profile(
    conn: &mut PgConnection,
    query_params: UpdateProfileParams,
) -> Result<(), DieselError> {
    use crate::schema::users;

    let update = UpdateProfileParamsInternal {
        display_name: query_params.display_name.into_nullable(),
        email: query_params.email.into_nullable(),
        password_hash: query_params
            .password_hash
            .into_option()
            .map(|s| s.to_string()),
        profile_image: query_params.profile_image.into_nullable(),
        security_question: query_params.security_question.into_nullable(),
        security_answer: query_params.security_answer.into_nullable(),
    };

    diesel::update(users::table)
        .filter(users::id.eq(&query_params.id))
        .set(&update)
        .execute(conn)
        .map(|_| ())
}

pub fn follow(conn: &mut PgConnection, user_id: UserId, follow: UserId) -> Result<(), DieselError> {
    let uid = user_id;
    let fid = follow;
    {
        use crate::schema::followers::dsl::*;
        diesel::insert_into(followers)
            .values((user_id.eq(uid), follows.eq(fid)))
            .on_conflict((user_id, follows))
            .do_nothing()
            .execute(conn)?;
    }

    let _ = crate::notification::create_notification(conn, fid, uid, 1, None);
    Ok(())
}

pub fn unfollow(
    conn: &mut PgConnection,
    user_id: UserId,
    stop_following: UserId,
) -> Result<DeleteStatus, DieselError> {
    let uid = user_id;
    let fid = stop_following;
    {
        use crate::schema::followers::dsl::*;
        let res = diesel::delete(followers)
            .filter(user_id.eq(uid))
            .filter(follows.eq(fid))
            .execute(conn)
            .map(|rowcount| {
                if rowcount > 0 {
                    DeleteStatus::Deleted
                } else {
                    DeleteStatus::NotFound
                }
            });
        
        if let Ok(DeleteStatus::Deleted) = res {
            let _ = crate::notification::create_notification(conn, fid, uid, 2, None);
        }
        res
    }
}

pub fn is_following(
    conn: &mut PgConnection,
    user_id: UserId,
    is_following: UserId,
) -> Result<bool, DieselError> {
    let uid = user_id;
    let fid = is_following;
    {
        use crate::schema::followers::dsl::*;
        use diesel::dsl::count;

        followers
            .filter(user_id.eq(uid))
            .filter(follows.eq(fid))
            .select(count(user_id))
            .get_result(conn)
            .optional()
            .map(|n: Option<i64>| match n {
                Some(n) => n == 1,
                None => false,
            })
    }
}

#[cfg(test)]
pub mod tests {
    

    pub mod util {
        use diesel::PgConnection;

        use crate::user::User;

        pub fn new_user(conn: &mut PgConnection, handle: &str) -> User {
            use crate::user as user_query;

            let hash = uchat_crypto::hash_password("password").unwrap();
            let id = user_query::new(conn, hash, handle).unwrap();
            user_query::get(conn, id).unwrap()
        }
    }

    #[test]
    fn update_security_question() -> crate::test_db::Result<()> {
        let mut conn = crate::test_db::new_connection();
        let user = util::new_user(&mut conn, "user1");

        let update = crate::user::UpdateProfileParams {
            id: user.id,
            display_name: uchat_endpoint::Update::NoChange,
            email: uchat_endpoint::Update::NoChange,
            password_hash: uchat_endpoint::Update::NoChange,
            profile_image: uchat_endpoint::Update::NoChange,
            security_question: uchat_endpoint::Update::Change("What is your pet's name?".to_string()),
            security_answer: uchat_endpoint::Update::Change("Fluffy".to_string()),
        };

        crate::user::update_profile(&mut conn, update)?;

        let updated_user = crate::user::get(&mut conn, user.id)?;
        assert_eq!(updated_user.security_question, Some("What is your pet's name?".to_string()));
        assert_eq!(updated_user.security_answer, Some("Fluffy".to_string()));

        Ok(())
    }
}
