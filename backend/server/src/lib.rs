use axum::extract::FromRef;
use uchat_query::{AsyncConnection, AsyncConnectionPool, QueryError};

pub mod error;
pub mod extractor;
pub mod handler;
pub mod logging;
pub mod router;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub db_pool: AsyncConnectionPool,
    pub signing_keys: uchat_crypto::sign::Keys,
    pub rng: rand::rngs::StdRng,
}

impl AppState {
    pub async fn connect(&self) -> Result<AsyncConnection<'_>, QueryError> {
        self.db_pool.get().await
    }
}

pub mod cli {
    use color_eyre::{eyre::Context, Help};
    use rand::{CryptoRng, RngCore};
    use uchat_crypto::sign::{encode_private_key, EncodedPrivateKey, Keys};

    pub fn gen_keys<R>(rng: &mut R) -> color_eyre::Result<(EncodedPrivateKey, Keys)>
    where
        R: CryptoRng + RngCore,
    {
        let (private_key, keys) = Keys::generate(rng)?;
        let private_key = encode_private_key(private_key)?;
        Ok((private_key, keys))
    }

    pub fn load_keys() -> color_eyre::Result<Keys> {
        let private_key = std::env::var("API_PRIVATE_KEY")
            .wrap_err("failed to locate private API key")
            .suggestion("set API_PRIVATE_KEY environment variable")?;

        Ok(Keys::from_encoded(private_key)?)
    }
}

#[cfg(test)]
pub mod tests {
    use hyper::StatusCode;
    use uchat_domain::{Password, Username};
    use uchat_endpoint::{
        user::endpoint::{CreateUser, CreateUserOk},
        Endpoint,
    };

    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    pub mod util {
        use axum::{
            response::{IntoResponse, Response},
            Router,
        };
        use hyper::Request;
        use serde::Serialize;
        use tower::ServiceExt;
        use uchat_crypto::sign::Keys;
        use uchat_query::AsyncConnectionPool;

        use crate::AppState;

        pub async fn new_state() -> AppState {
            let connection_url = dotenvy::var("TEST_DATABASE_URL")
                .expect("TEST_DATABASE_URL must be set in order to run tests");
            let mut rng = uchat_crypto::new_rng();
            AppState {
                db_pool: AsyncConnectionPool::new(connection_url).await.unwrap(),
                signing_keys: Keys::generate(&mut rng).unwrap().1,
                rng,
            }
        }

        pub async fn new_router() -> Router {
            let state = new_state().await;
            crate::router::new_router(state)
        }

        pub async fn api_request_with_router<P>(router: Router, uri: &str, payload: P) -> Response
        where
            P: Serialize,
        {
            let payload = serde_json::to_string(&payload).unwrap();
            router
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .header("Content-Type", "application/json")
                        .uri(uri)
                        .body(payload.into())
                        .unwrap(),
                )
                .await
                .unwrap()
                .into_response()
        }

        pub async fn api_request<P>(uri: &str, payload: P) -> Response
        where
            P: Serialize,
        {
            let router = new_router().await;
            api_request_with_router(router, uri, payload).await
        }

        pub async fn api_request_auth<P>(uri: &str, payload: P, session_id: &str, signature: &str) -> Response
        where
            P: Serialize,
        {
            let router = new_router().await;
            api_request_auth_with_router(router, uri, payload, session_id, signature).await
        }

        pub async fn api_request_auth_with_router<P>(router: Router, uri: &str, payload: P, session_id: &str, signature: &str) -> Response
        where
            P: Serialize,
        {
            let payload = serde_json::to_string(&payload).unwrap();
            let cookie_header = format!("{}={}; {}={}", uchat_cookie::SESSION_ID, session_id, uchat_cookie::SESSION_SIGNATURE, signature);
            router
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .header("Content-Type", "application/json")
                        .header("Cookie", cookie_header)
                        .uri(uri)
                        .body(payload.into())
                        .unwrap(),
                )
                .await
                .unwrap()
                .into_response()
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn create_user() -> Result<()> {
// ... unchanged ...
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        let username: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        // user doesnt exist yet
        {
            let payload = CreateUser {
                password: Password::new("password")?,
                username: Username::new(&username)?,
            };

            let response = util::api_request(CreateUser::URL, payload).await;

            assert_eq!(StatusCode::CREATED, response.status());

            let response = hyper::body::to_bytes(response.into_body()).await?;
            let response: CreateUserOk = serde_json::from_slice(&response)?;

            assert_eq!(username, response.username.into_inner());
        }

        // try to add duplicate user
        {
            let payload = CreateUser {
                password: Password::new("password")?,
                username: Username::new(username)?,
            };
            let response = util::api_request(CreateUser::URL, payload).await;

            assert_eq!(StatusCode::CONFLICT, response.status());
        }

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn login_user() -> Result<()> {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};
        use uchat_endpoint::user::endpoint::Login;

        let username: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        
        let password = "password";

        // Create the user first
        let create_payload = CreateUser {
            password: Password::new(password)?,
            username: Username::new(&username)?,
        };
        util::api_request(CreateUser::URL, create_payload).await;

        // Valid login
        let login_payload = Login {
            password: Password::new(password)?,
            username: Username::new(&username)?,
        };
        let response = util::api_request(Login::URL, login_payload).await;
        assert_eq!(StatusCode::OK, response.status());

        // Invalid login (wrong password)
        let login_payload = Login {
            password: Password::new("wrongpass")?,
            username: Username::new(&username)?,
        };
        let response = util::api_request(Login::URL, login_payload).await;
        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn new_post() -> Result<()> {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};
        use uchat_endpoint::user::endpoint::{Login, LoginOk};
        use uchat_endpoint::post::endpoint::NewPost;
        use uchat_endpoint::post::types::NewPostOptions;
        use uchat_domain::post::{Headline, Message};

        let state = util::new_state().await;

        let username: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        let password = "password";

        let create_payload = CreateUser {
            password: Password::new(password)?,
            username: Username::new(&username)?,
        };
        util::api_request_with_router(crate::router::new_router(state.clone()), CreateUser::URL, create_payload).await;

        let login_payload = Login {
            password: Password::new(password)?,
            username: Username::new(&username)?,
        };
        let response = util::api_request_with_router(crate::router::new_router(state.clone()), Login::URL, login_payload).await;
        
        let response_body = hyper::body::to_bytes(response.into_body()).await?;
        let login_ok: LoginOk = serde_json::from_slice(&response_body)?;

        let session_id = login_ok.session_id.to_string();
        let signature = login_ok.session_signature;

        let new_post_payload = NewPost {
            content: uchat_endpoint::post::types::Content::Chat(uchat_endpoint::post::types::Chat {
                headline: Some(Headline::new("My first post")?),
                message: Message::new("Hello world from test!")?,
            }),
            options: NewPostOptions::default(),
        };

        let response = util::api_request_auth_with_router(
            crate::router::new_router(state.clone()),
            NewPost::URL,
            new_post_payload,
            &session_id,
            &signature,
        ).await;

        assert_eq!(StatusCode::OK, response.status());

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn forgot_password() -> Result<()> {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};
        use uchat_endpoint::user::endpoint::{UpdateProfile, ForgotPassword};
        use uchat_endpoint::Update;

        let state = util::new_state().await;

        let username1: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();
        let username2: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();

        // Create users
        let create_payload1 = CreateUser {
            password: Password::new("password")?,
            username: Username::new(&username1)?,
        };
        util::api_request_with_router(crate::router::new_router(state.clone()), CreateUser::URL, create_payload1).await;
        
        let create_payload2 = CreateUser {
            password: Password::new("password")?,
            username: Username::new(&username2)?,
        };
        util::api_request_with_router(crate::router::new_router(state.clone()), CreateUser::URL, create_payload2).await;

        // Login user1
        let login_payload = uchat_endpoint::user::endpoint::Login {
            password: Password::new("password")?,
            username: Username::new(&username1)?,
        };
        let response = util::api_request_with_router(crate::router::new_router(state.clone()), uchat_endpoint::user::endpoint::Login::URL, login_payload).await;
        let response_body = hyper::body::to_bytes(response.into_body()).await?;
        let login_ok: uchat_endpoint::user::endpoint::LoginOk = serde_json::from_slice(&response_body)?;

        // Set security question
        let update_profile_payload = UpdateProfile {
            display_name: Update::NoChange,
            email: Update::NoChange,
            password: Update::NoChange,
            profile_image: Update::NoChange,
            security_question: Update::Change("question".to_string()),
            security_answer: Update::Change("answer".to_string()),
        };
        let response = util::api_request_auth_with_router(
            crate::router::new_router(state.clone()),
            UpdateProfile::URL,
            update_profile_payload,
            &login_ok.session_id.to_string(),
            &login_ok.session_signature,
        ).await;
        assert_eq!(StatusCode::OK, response.status());

        // Test forgot password (fails because no chat)
        let forgot_password_payload = ForgotPassword {
            username: Username::new(&username1)?,
            chatted_with_username: Username::new(&username2)?,
            security_answer: "answer".to_string(),
            new_password: Password::new("newpassword")?,
        };
        let response = util::api_request_with_router(crate::router::new_router(state.clone()), ForgotPassword::URL, forgot_password_payload.clone()).await;
        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        // Send a message directly using query
        {
            let mut conn = state.db_pool.get().await.unwrap();
            let user1_id = uchat_query::user::find(&mut conn, &Username::new(&username1).unwrap()).unwrap().id;
            let user2_id = uchat_query::user::find(&mut conn, &Username::new(&username2).unwrap()).unwrap().id;
            uchat_query::chat::send_message(&mut conn, user1_id, user2_id, "hello".to_string()).unwrap();
        }

        // Test forgot password (fails because wrong security answer)
        let forgot_password_wrong_answer_payload = ForgotPassword {
            username: Username::new(&username1)?,
            chatted_with_username: Username::new(&username2)?,
            security_answer: "wronganswer".to_string(),
            new_password: Password::new("newpassword")?,
        };
        let response = util::api_request_with_router(crate::router::new_router(state.clone()), ForgotPassword::URL, forgot_password_wrong_answer_payload).await;
        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        // Test forgot password (fails because chatted_with_username does not exist)
        let forgot_password_bad_user_payload = ForgotPassword {
            username: Username::new(&username1)?,
            chatted_with_username: Username::new("nonexistent123")?,
            security_answer: "answer".to_string(),
            new_password: Password::new("newpassword")?,
        };
        let response = util::api_request_with_router(crate::router::new_router(state.clone()), ForgotPassword::URL, forgot_password_bad_user_payload).await;
        assert_eq!(StatusCode::BAD_REQUEST, response.status());

        // Test forgot password (succeeds now)
        let response = util::api_request_with_router(crate::router::new_router(state.clone()), ForgotPassword::URL, forgot_password_payload).await;
        assert_eq!(StatusCode::OK, response.status());

        // Test login with new password
        let login_payload = uchat_endpoint::user::endpoint::Login {
            password: Password::new("newpassword")?,
            username: Username::new(&username1)?,
        };
        let response = util::api_request_with_router(crate::router::new_router(state.clone()), uchat_endpoint::user::endpoint::Login::URL, login_payload).await;
        assert_eq!(StatusCode::OK, response.status());

        Ok(())
    }
}
