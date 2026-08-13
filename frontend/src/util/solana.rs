use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
const BASE58_ALPHABET = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';

function bytesToBase58(bytes) {
    let digits = [0];
    for (let i = 0; i < bytes.length; i++) {
        let carry = bytes[i];
        for (let j = 0; j < digits.length; j++) {
            carry += digits[j] << 8;
            digits[j] = carry % 58;
            carry = (carry / 58) | 0;
        }
        while (carry > 0) {
            digits.push(carry % 58);
            carry = (carry / 58) | 0;
        }
    }
    let leadingZeros = 0;
    for (let i = 0; i < bytes.length && bytes[i] === 0; i++) leadingZeros++;
    return BASE58_ALPHABET[0].repeat(leadingZeros)
        + digits.reverse().map((d) => BASE58_ALPHABET[d]).join('');
}

let selected = null;

function pickProvider() {
    if (window.phantom && window.phantom.solana && window.phantom.solana.isPhantom) {
        return window.phantom.solana;
    }
    if (window.solana && window.solana.isPhantom) return window.solana;
    if (window.solflare && window.solflare.isSolflare) return window.solflare;
    if (window.solana) return window.solana;
    return null;
}

export async function solana_is_available() {
    return pickProvider() !== null;
}

export async function solana_connect() {
    selected = pickProvider();
    if (!selected) {
        throw new Error('No Solana wallet detected');
    }
    const resp = await selected.connect();
    return resp.publicKey.toString();
}

export async function solana_sign_message(message) {
    if (!selected) {
        selected = pickProvider();
    }
    if (!selected) {
        throw new Error('No Solana wallet detected');
    }
    const encoded = new TextEncoder().encode(message);
    const signed = await selected.signMessage(encoded, 'utf8');
    return bytesToBase58(signed.signature);
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn solana_is_available() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn solana_connect() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn solana_sign_message(message: String) -> Result<JsValue, JsValue>;
}

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("No Solana wallet detected. Install Phantom or a compatible wallet extension.")]
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
    solana_is_available().await.map_or(false, |v| v.is_truthy())
}

/// Prompts the wallet's connect UI and returns the selected base58 public key.
pub async fn connect() -> Result<String, WalletError> {
    let account = solana_connect().await.map_err(|e| {
        let message = js_error_to_string(e);
        if message.contains("No Solana wallet detected") {
            WalletError::NotAvailable
        } else {
            WalletError::Request(message)
        }
    })?;

    account
        .as_string()
        .ok_or(WalletError::UnexpectedResponse)
}

/// Prompts the wallet to sign `message`, returning the base58-encoded signature.
pub async fn sign(message: &str) -> Result<String, WalletError> {
    let signature = solana_sign_message(message.to_string())
        .await
        .map_err(|e| WalletError::Request(js_error_to_string(e)))?;

    signature
        .as_string()
        .ok_or(WalletError::UnexpectedResponse)
}
