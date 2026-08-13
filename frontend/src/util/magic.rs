use wasm_bindgen::prelude::*;

// Requires `<script src="https://auth.magic.link/sdk">` and
// `<script src="https://auth.magic.link/sdk/extension/solana">` loaded in index.html,
// which expose `window.Magic` and the global `SolanaExtension` constructor.
//
// NOTE: `magic.solana.signMessage()`'s return shape isn't pinned down in Magic's public
// docs at the time of writing. `magic_sign_message` below defensively handles the shapes
// that are plausible (raw bytes, `{ signature: bytes }`, already-base58 string) but this
// needs a live smoke test against a real Magic publishable key before shipping.
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

let magicInstance = null;

function getMagic(apiKey, rpcUrl) {
    if (typeof window.Magic === 'undefined' || typeof SolanaExtension === 'undefined') {
        throw new Error('Magic SDK not loaded');
    }
    if (!magicInstance) {
        magicInstance = new window.Magic(apiKey, {
            extensions: [new SolanaExtension({ rpcUrl })],
        });
    }
    return magicInstance;
}

export async function magic_is_available() {
    return typeof window.Magic !== 'undefined' && typeof SolanaExtension !== 'undefined';
}

export async function magic_login_with_email(apiKey, rpcUrl, email) {
    const magic = getMagic(apiKey, rpcUrl);
    await magic.auth.loginWithEmailOTP({ email });
    const metadata = await magic.user.getMetadata();
    return metadata.publicAddress;
}

export async function magic_sign_message(apiKey, rpcUrl, message) {
    const magic = getMagic(apiKey, rpcUrl);
    const signed = await magic.solana.signMessage(message);

    if (typeof signed === 'string') return signed;
    if (signed instanceof Uint8Array) return bytesToBase58(signed);
    if (signed && signed.signature) return bytesToBase58(new Uint8Array(signed.signature));
    throw new Error('Unexpected signMessage response shape from Magic SDK');
}

export async function magic_logout(apiKey, rpcUrl) {
    const magic = getMagic(apiKey, rpcUrl);
    await magic.user.logout();
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn magic_is_available() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn magic_login_with_email(
        api_key: String,
        rpc_url: String,
        email: String,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn magic_sign_message(
        api_key: String,
        rpc_url: String,
        message: String,
    ) -> Result<JsValue, JsValue>;
}

#[derive(Debug, thiserror::Error)]
pub enum MagicError {
    #[error("Magic SDK not loaded. Check your network connection.")]
    NotAvailable,

    #[error("Magic publishable key not configured (set MAGIC_PUBLISHABLE_KEY at build time)")]
    NotConfigured,

    #[error("Magic request failed: {0}")]
    Request(String),

    #[error("Magic returned an unexpected response")]
    UnexpectedResponse,
}

const RPC_URL: &str = "https://api.mainnet-beta.solana.com";

fn api_key() -> Result<&'static str, MagicError> {
    match option_env!("MAGIC_PUBLISHABLE_KEY") {
        Some(key) if !key.is_empty() => Ok(key),
        _ => Err(MagicError::NotConfigured),
    }
}

fn js_error_to_string(err: JsValue) -> String {
    err.as_string()
        .or_else(|| {
            js_sys::Reflect::get(&err, &JsValue::from_str("message"))
                .ok()
                .and_then(|m| m.as_string())
        })
        .unwrap_or_else(|| "unknown Magic error".to_string())
}

pub async fn is_available() -> bool {
    magic_is_available().await.map_or(false, |v| v.is_truthy())
}

/// Prompts Magic's email-OTP flow and returns the user's embedded Solana public key
/// (base58), once they've confirmed the login link sent to their inbox.
pub async fn login_with_email(email: &str) -> Result<String, MagicError> {
    let key = api_key()?;
    let address = magic_login_with_email(key.to_string(), RPC_URL.to_string(), email.to_string())
        .await
        .map_err(|e| {
            let message = js_error_to_string(e);
            if message.contains("Magic SDK not loaded") {
                MagicError::NotAvailable
            } else {
                MagicError::Request(message)
            }
        })?;

    address.as_string().ok_or(MagicError::UnexpectedResponse)
}

/// Signs `message` with the logged-in Magic user's embedded Solana key, returning the
/// base58-encoded signature.
pub async fn sign(message: &str) -> Result<String, MagicError> {
    let key = api_key()?;
    let signature =
        magic_sign_message(key.to_string(), RPC_URL.to_string(), message.to_string())
            .await
            .map_err(|e| MagicError::Request(js_error_to_string(e)))?;

    signature.as_string().ok_or(MagicError::UnexpectedResponse)
}
