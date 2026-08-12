#![allow(non_local_definitions)]
#[cfg(feature = "query")]
#[macro_use]
extern crate diesel_derive_newtype;

pub mod ids;
pub mod post;
pub mod user;
pub mod wallet;

pub use user::{Password, Username};
pub use wallet::EthAddress;

pub trait UserFacingError {
    fn formatted_error(&self) -> &'static str;
}
