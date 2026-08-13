use nutype::nutype;
use once_cell::sync::OnceCell;
use regex::Regex;

use crate::UserFacingError;

static ETH_ADDRESS_REGEX: OnceCell<Regex> = OnceCell::new();

fn is_valid_eth_address(address: &str) -> bool {
    let regex =
        ETH_ADDRESS_REGEX.get_or_init(|| Regex::new(r#"^0x[0-9a-f]{40}$"#).unwrap());

    regex.is_match(address)
}

#[nutype(validate(with = is_valid_eth_address))]
#[derive(AsRef, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EthAddress(String);

impl UserFacingError for EthAddressError {
    fn formatted_error(&self) -> &'static str {
        match self {
            EthAddressError::Invalid => "Wallet address is not a valid lowercase Ethereum address.",
        }
    }
}

fn is_valid_solana_address(address: &str) -> bool {
    match bs58::decode(address).into_vec() {
        Ok(bytes) => bytes.len() == 32,
        Err(_) => false,
    }
}

#[nutype(validate(with = is_valid_solana_address))]
#[derive(AsRef, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SolanaAddress(String);

impl UserFacingError for SolanaAddressError {
    fn formatted_error(&self) -> &'static str {
        match self {
            SolanaAddressError::Invalid => "Wallet address is not a valid Solana address.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_eth_address() {
        assert!(EthAddress::new("0x0102030405060708090a0b0c0d0e0f1011121314").is_ok());
    }

    #[test]
    fn invalid_eth_address() {
        assert_eq!(EthAddress::new(""), Err(EthAddressError::Invalid));
        assert_eq!(
            EthAddress::new("0x0102030405060708090A0B0C0D0E0F1011121314"),
            Err(EthAddressError::Invalid)
        );
        assert_eq!(
            EthAddress::new("0x123"),
            Err(EthAddressError::Invalid)
        );
    }

    #[test]
    fn valid_solana_address() {
        assert!(SolanaAddress::new("11111111111111111111111111111111").is_ok());
    }

    #[test]
    fn invalid_solana_address() {
        assert_eq!(SolanaAddress::new(""), Err(SolanaAddressError::Invalid));
        assert_eq!(
            SolanaAddress::new("not-base58!!"),
            Err(SolanaAddressError::Invalid)
        );
    }
}
