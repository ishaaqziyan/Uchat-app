use wasm_bindgen::prelude::*;

// Wallet extensions (MetaMask, Coinbase Wallet, etc.) each want to own `window.ethereum`.
// With more than one installed, the object is either overwritten by whichever extension
// injects last, or wrapped in a picker (e.g. Coinbase's `evmAsk.js`) that can fail outright.
// EIP-6963 lets every extension announce itself independently instead, so we can target
// MetaMask by its `rdns` and bypass that conflict entirely.
#[wasm_bindgen(inline_js = r#"
const providers = new Map();
let selected = null;

window.addEventListener('eip6963:announceProvider', (event) => {
    providers.set(event.detail.info.uuid, event.detail);
});
window.dispatchEvent(new Event('eip6963:requestProvider'));

async function discover() {
    if (providers.size === 0) {
        await new Promise((resolve) => {
            window.dispatchEvent(new Event('eip6963:requestProvider'));
            setTimeout(resolve, 150);
        });
    }
}

function pickProvider() {
    const metamask = [...providers.values()].find((p) => p.info.rdns === 'io.metamask');
    if (metamask) return metamask.provider;
    if (providers.size > 0) return providers.values().next().value.provider;
    if (typeof window.ethereum !== 'undefined') return window.ethereum;
    return null;
}

export async function eth_is_available() {
    await discover();
    return pickProvider() !== null;
}

export async function eth_request_accounts() {
    await discover();
    selected = pickProvider();
    if (!selected) {
        throw new Error('No Ethereum wallet detected');
    }
    const accounts = await selected.request({ method: 'eth_requestAccounts' });
    return accounts[0];
}

export async function eth_personal_sign(message, address) {
    if (!selected) {
        await discover();
        selected = pickProvider();
    }
    if (!selected) {
        throw new Error('No Ethereum wallet detected');
    }
    return await selected.request({
        method: 'personal_sign',
        params: [message, address],
    });
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn eth_is_available() -> Result<JsValue, JsValue>;

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

pub async fn is_available() -> bool {
    eth_is_available().await.map_or(false, |v| v.is_truthy())
}

/// Prompts the wallet's connect UI and returns the selected `0x`-prefixed lowercase address.
pub async fn connect() -> Result<String, WalletError> {
    let account = eth_request_accounts().await.map_err(|e| {
        let message = js_error_to_string(e);
        if message.contains("No Ethereum wallet detected") {
            WalletError::NotAvailable
        } else {
            WalletError::Request(message)
        }
    })?;

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
