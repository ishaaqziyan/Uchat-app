pub mod api_client;
pub mod cookie;
pub use api_client::ApiClient;

use serde::Deserialize;
use wasm_bindgen::JsCast;
use web_sys::{History, HtmlDocument, Window};

#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("request timeout")]
    Timeout,

    #[error("bad request: {0}")]
    BadRequest(#[from] uchat_endpoint::RequestFailed),
}

#[derive(Clone, Deserialize, PartialEq)]
pub struct ApiResponse {
    message: String,
    status: String,
}

#[macro_export]
macro_rules! maybe_class {
    ($class:expr, $condition:expr) => {
        if $condition {
            $class
        } else {
            ""
        }
    };
}
pub use maybe_class;

// async_handler and sync_handler macros are removed - use closures directly in 0.6+
// Example replacements:
// async_handler!(&cx, [api_client], move |_| async move { ... })
// becomes:
// move |_| spawn(async move { ... })
//
// sync_handler!([state], move |ev| { ... })
// becomes:
// move |ev| { ... }

pub fn window() -> Window {
    web_sys::window().expect("missing Window object")
}

pub fn document() -> HtmlDocument {
    web_sys::window()
        .expect("missing Window object")
        .document()
        .expect("missing Document object")
        .dyn_into::<HtmlDocument>()
        .expect("failed to convert Document into HtmlDocument")
}

pub fn history() -> History {
    window().history().expect("missing History object")
}

pub fn load_history_state<T>() -> Option<T>
where
    T: serde::de::DeserializeOwned + std::fmt::Debug,
{
    history()
        .state()
        .ok()
        .and_then(|state| state.as_string())
        .and_then(|state| serde_json::from_str(&state).ok())
}
