use chrono::Utc;
use diesel::prelude::*;
use diesel::PgConnection;
use password_hash::PasswordHashString;
use uchat_domain::ids::UserId;
use uuid::Uuid;

use crate::user::User;
use crate::QueryError;

const NONCE_TTL_MINUTES: i64 = 10;

pub fn create_nonce(
    conn: &mut PgConnection,
    address: &str,
    nonce: &str,
    message: &str,
) -> Result<(), QueryError> {
    use crate::schema::wallet_nonces::{self, columns};

    diesel::insert_into(wallet_nonces::table)
        .values((
            columns::id.eq(Uuid::new_v4()),
            columns::eth_address.eq(address),
            columns::nonce.eq(nonce),
            columns::message.eq(message),
            columns::expires_at.eq(Utc::now() + chrono::Duration::minutes(NONCE_TTL_MINUTES)),
        ))
        .execute(conn)?;

    Ok(())
}

/// Marks the challenge matching `(address, message)` as consumed, returning `true` if it
/// was found, unconsumed, and unexpired. Matching on the full signed message (rather than
/// just the embedded nonce) means the caller doesn't need to parse the nonce back out.
pub fn consume_nonce(
    conn: &mut PgConnection,
    address: &str,
    signed_message: &str,
) -> Result<bool, QueryError> {
    use crate::schema::wallet_nonces::dsl::*;

    let rowcount = diesel::update(wallet_nonces)
        .filter(eth_address.eq(address))
        .filter(message.eq(signed_message))
        .filter(consumed.eq(false))
        .filter(expires_at.gt(Utc::now()))
        .set(consumed.eq(true))
        .execute(conn)?;

    Ok(rowcount > 0)
}

pub fn find_by_eth_address(
    conn: &mut PgConnection,
    address: &str,
) -> Result<Option<User>, QueryError> {
    use crate::schema::users::dsl::*;

    users
        .filter(eth_address.eq(address))
        .get_result(conn)
        .optional()
        .map_err(QueryError::from)
}

fn wallet_handle(address: &str, attempt: u32) -> String {
    let short = format!("{}..{}", &address[..6], &address[address.len() - 4..]);
    if attempt == 0 {
        short
    } else {
        format!("{short}-{attempt}")
    }
}

/// Creates a new account for a wallet address that has just completed sign-in-with-ethereum
/// for the first time. `password_hash` is the hash of a random password nobody knows, since
/// wallet-only accounts authenticate exclusively via signature, not password.
pub fn create_wallet_user(
    conn: &mut PgConnection,
    address: &str,
    password_hash: PasswordHashString,
) -> Result<UserId, QueryError> {
    use crate::schema::users::{self, columns};

    let user_id = UserId::new();

    for attempt in 0..5 {
        let handle = wallet_handle(address, attempt);
        let result = diesel::insert_into(users::table)
            .values((
                columns::id.eq(user_id),
                columns::password_hash.eq(password_hash.as_str()),
                columns::handle.eq(&handle),
                columns::eth_address.eq(address),
            ))
            .execute(conn);

        match result {
            Ok(_) => return Ok(user_id),
            Err(e) => {
                let err = QueryError::from(e);
                if !matches!(err, QueryError::UniqueViolation) {
                    return Err(err);
                }
            }
        }
    }

    Err(QueryError::UniqueViolation)
}
