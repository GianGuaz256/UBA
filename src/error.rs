//! Error types for the UBA library

use thiserror::Error;

/// Result type alias for UBA operations
pub type Result<T> = std::result::Result<T, UbaError>;

/// Comprehensive error types for UBA operations
#[derive(Error, Debug)]
pub enum UbaError {
    /// Invalid seed format or content
    #[error("Invalid seed: {0}")]
    InvalidSeed(String),

    /// Invalid UBA string format
    #[error("Invalid UBA format: {0}")]
    InvalidUbaFormat(String),

    /// Nostr relay connection or communication error
    #[error("Nostr relay error: {0}")]
    NostrRelay(String),

    /// Bitcoin address generation error
    #[error("Bitcoin address generation error: {0}")]
    AddressGeneration(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Network error (connection, timeout, etc.)
    #[error("Network error: {0}")]
    Network(String),

    /// Note not found on any relay
    #[error("Note not found with ID: {0}")]
    NoteNotFound(String),

    /// Invalid relay URL
    #[error("Invalid relay URL: {0}")]
    InvalidRelayUrl(String),

    /// Encryption/decryption error
    #[error("Encryption error: {0}")]
    Encryption(String),

    /// Invalid encryption key format or length
    #[error("Invalid encryption key: {0}")]
    InvalidEncryptionKey(String),

    /// Invalid label format
    #[error("Invalid label: {0}")]
    InvalidLabel(String),

    /// Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// BIP39 mnemonic error
    #[error("BIP39 error: {0}")]
    Bip39(String),

    /// Hex decoding error
    #[error("Hex decoding error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Event not found error
    #[error("Event not found: {0}")]
    EventNotFound(String),

    /// Update validation error
    #[error("Update validation error: {0}")]
    UpdateValidation(String),

    /// Invalid update data
    #[error("Invalid update data: {0}")]
    InvalidUpdateData(String),
}

impl From<bitcoin::address::Error> for UbaError {
    fn from(err: bitcoin::address::Error) -> Self {
        UbaError::AddressGeneration(err.to_string())
    }
}

impl From<bitcoin::secp256k1::Error> for UbaError {
    fn from(err: bitcoin::secp256k1::Error) -> Self {
        UbaError::AddressGeneration(err.to_string())
    }
}

impl From<nostr::key::Error> for UbaError {
    fn from(err: nostr::key::Error) -> Self {
        UbaError::NostrRelay(err.to_string())
    }
}

impl From<nostr_sdk::client::Error> for UbaError {
    fn from(err: nostr_sdk::client::Error) -> Self {
        UbaError::NostrRelay(err.to_string())
    }
}

impl From<bip39::Error> for UbaError {
    fn from(err: bip39::Error) -> Self {
        UbaError::Bip39(err.to_string())
    }
}
