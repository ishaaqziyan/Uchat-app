use serde::{Deserialize, Serialize};

pub mod user;
pub mod post;

pub trait Endpoint {
    const URL: &'static str;
    fn url(&self) -> &'static str {
        Self::URL
    }
}

macro_rules! route {
    ($url:literal => $request_type:ty) => {
        impl Endpoint for $request_type{
            const URL: &'static str = $url;
        }
    };
}

#[derive(thiserror::Error, Debug, Deserialize, Serialize)]
#[error("{msg}")]
pub struct RequestFailed {
    pub msg: String,
}

// Public Routes
route!("/account/create"=> user::endpoint::CreateUser);
route!("/account/login"=> user::endpoint::Login);


// Authorized Routes
route!("/post/new"=> post::endpoint::NewPost);
route!("/posts/trending"=> post::endpoint::TrendingPosts);



