use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function eth_is_available() {
    return typeof window.ethereum !== 'undefined';
}

export async function eth_request_accounts() {
    const accounts = await window.ethereum.request({ method: 'eth_requestAccounts' });
    return accounts[0];
}

export async function eth_personal_sign(message, address) {
    return await window.ethereum.request({
        method: 'personal_sign',
        params: [message, address],
    });
}
"#)]
extern "C" {
    fn eth_is_available() -> bool;

    #[wasm_bindgen(catch)]
    async fn eth_request_accounts() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn eth_personal_sign(message: String, address: String) -> Result<JsValue, JsValue>;
}

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("No Ethereum wallet detected. Install MetaMask or a compatible wallet extension.")]
    NotAvailable,

    #[error("Wallet request failed: {0}")]
    Request(String),

    #[error("Wallet returned an unexpected response")]
    UnexpectedResponse,
}

fn js_error_to_string(err: JsValue) -> String {
    err.as_string()
        .or_else(|| {
            js_sys::Reflect::get(&err, &JsValue::from_str("message"))
                .ok()
                .and_then(|m| m.as_string())
        })
        .unwrap_or_else(|| "unknown wallet error".to_string())
}

pub fn is_available() -> bool {
    eth_is_available()
}

/// Prompts the wallet's connect UI and returns the selected `0x`-prefixed lowercase address.
pub async fn connect() -> Result<String, WalletError> {
    if !is_available() {
        return Err(WalletError::NotAvailable);
    }

    let account = eth_request_accounts()
        .await
        .map_err(|e| WalletError::Request(js_error_to_string(e)))?;

    account
        .as_string()
        .map(|address| address.to_lowercase())
        .ok_or(WalletError::UnexpectedResponse)
}

/// Prompts the wallet to sign `message` as `address` via `personal_sign`, returning the
/// `0x`-prefixed signature hex.
pub async fn sign(address: &str, message: &str) -> Result<String, WalletError> {
    let signature = eth_personal_sign(message.to_string(), address.to_string())
        .await
        .map_err(|e| WalletError::Request(js_error_to_string(e)))?;

    signature
        .as_string()
        .ok_or(WalletError::UnexpectedResponse)
}
