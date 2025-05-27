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

    /// Rate limiting error
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Input validation error
    #[error("Input validation failed: {0}")]
    InputValidation(String),

    /// Connection retry exhausted
    #[error("Connection retry exhausted: {0}")]
    RetryExhausted(String),

    /// System time error
    #[error("System time error: {0}")]
    SystemTime(String),

    /// Key derivation error
    #[error("Key derivation error: {0}")]
    KeyDerivation(String),
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

impl From<std::time::SystemTimeError> for UbaError {
    fn from(err: std::time::SystemTimeError) -> Self {
        UbaError::SystemTime(err.to_string())
    }
}

impl From<hkdf::InvalidLength> for UbaError {
    fn from(err: hkdf::InvalidLength) -> Self {
        UbaError::KeyDerivation(err.to_string())
    }
}

/// Input validation utilities
pub mod validation {
    use super::*;
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    /// Rate limiter for preventing abuse
    pub struct RateLimiter {
        requests: HashMap<String, Vec<Instant>>,
        max_requests: usize,
        window: Duration,
    }

    impl RateLimiter {
        /// Create a new rate limiter
        pub fn new(max_requests: usize, window: Duration) -> Self {
            Self {
                requests: HashMap::new(),
                max_requests,
                window,
            }
        }

        /// Check if a request is allowed for the given identifier
        pub fn is_allowed(&mut self, identifier: &str) -> Result<()> {
            let now = Instant::now();
            let requests = self.requests.entry(identifier.to_string()).or_default();

            // Remove old requests outside the window
            requests.retain(|&time| now.duration_since(time) < self.window);

            if requests.len() >= self.max_requests {
                return Err(UbaError::RateLimit(format!(
                    "Rate limit exceeded: {} requests in {:?}",
                    self.max_requests, self.window
                )));
            }

            requests.push(now);
            Ok(())
        }

        /// Clean up old entries to prevent memory leaks
        pub fn cleanup(&mut self) {
            let now = Instant::now();
            self.requests.retain(|_, requests| {
                requests.retain(|&time| now.duration_since(time) < self.window);
                !requests.is_empty()
            });
        }
    }

    /// Validate a seed phrase
    pub fn validate_seed(seed: &str) -> Result<()> {
        if seed.trim().is_empty() {
            return Err(UbaError::InputValidation("Seed cannot be empty".to_string()));
        }

        if seed.len() > 1000 {
            return Err(UbaError::InputValidation("Seed too long".to_string()));
        }

        // Check if it's a valid BIP39 mnemonic
        if let Err(e) = bip39::Mnemonic::parse(seed) {
            return Err(UbaError::InputValidation(format!("Invalid BIP39 mnemonic: {}", e)));
        }

        Ok(())
    }

    /// Validate a label
    pub fn validate_label(label: &str) -> Result<()> {
        if label.is_empty() {
            return Err(UbaError::InputValidation("Label cannot be empty".to_string()));
        }

        if label.len() > 100 {
            return Err(UbaError::InputValidation("Label too long (max 100 characters)".to_string()));
        }

        // Check for invalid characters
        if label.chars().any(|c| c.is_control() || c == '\n' || c == '\r') {
            return Err(UbaError::InputValidation("Label contains invalid characters".to_string()));
        }

        Ok(())
    }

    /// Validate relay URLs
    pub fn validate_relay_urls(urls: &[String]) -> Result<()> {
        if urls.is_empty() {
            return Err(UbaError::InputValidation("At least one relay URL is required".to_string()));
        }

        if urls.len() > 20 {
            return Err(UbaError::InputValidation("Too many relay URLs (max 20)".to_string()));
        }

        for url in urls {
            validate_relay_url(url)?;
        }

        Ok(())
    }

    /// Validate a single relay URL
    pub fn validate_relay_url(url: &str) -> Result<()> {
        if url.is_empty() {
            return Err(UbaError::InputValidation("Relay URL cannot be empty".to_string()));
        }

        if !url.starts_with("wss://") && !url.starts_with("ws://") {
            return Err(UbaError::InputValidation("Relay URL must use ws:// or wss://".to_string()));
        }

        // Parse URL to validate format
        url::Url::parse(url).map_err(|e| {
            UbaError::InputValidation(format!("Invalid relay URL format: {}", e))
        })?;

        Ok(())
    }

    /// Validate UBA format
    pub fn validate_uba_format(uba: &str) -> Result<()> {
        if uba.is_empty() {
            return Err(UbaError::InputValidation("UBA cannot be empty".to_string()));
        }

        if !uba.starts_with("UBA:") {
            return Err(UbaError::InputValidation("UBA must start with 'UBA:'".to_string()));
        }

        let content = &uba[4..]; // Remove "UBA:" prefix
        if content.is_empty() {
            return Err(UbaError::InputValidation("UBA content cannot be empty".to_string()));
        }

        // Basic validation of the event ID part
        let parts: Vec<&str> = content.split('&').collect();
        let event_id = parts[0];
        
        if event_id.len() != 64 {
            return Err(UbaError::InputValidation("Invalid event ID length".to_string()));
        }

        if !event_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(UbaError::InputValidation("Event ID must be hexadecimal".to_string()));
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_rate_limiter() {
            let mut limiter = RateLimiter::new(2, Duration::from_secs(1));
            
            assert!(limiter.is_allowed("user1").is_ok());
            assert!(limiter.is_allowed("user1").is_ok());
            assert!(limiter.is_allowed("user1").is_err()); // Should be rate limited
            
            // Different user should be allowed
            assert!(limiter.is_allowed("user2").is_ok());
        }

        #[test]
        fn test_validate_seed() {
            assert!(validate_seed("").is_err());
            assert!(validate_seed("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about").is_ok());
            assert!(validate_seed("invalid seed").is_err());
        }

        #[test]
        fn test_validate_label() {
            assert!(validate_label("").is_err());
            assert!(validate_label("valid-label").is_ok());
            assert!(validate_label(&"x".repeat(101)).is_err());
            assert!(validate_label("label\nwith\nnewlines").is_err());
        }

        #[test]
        fn test_validate_relay_url() {
            assert!(validate_relay_url("").is_err());
            assert!(validate_relay_url("wss://relay.damus.io").is_ok());
            assert!(validate_relay_url("ws://localhost:8080").is_ok());
            assert!(validate_relay_url("https://relay.damus.io").is_err());
            assert!(validate_relay_url("invalid-url").is_err());
        }

        #[test]
        fn test_validate_uba_format() {
            assert!(validate_uba_format("").is_err());
            assert!(validate_uba_format("UBA:").is_err());
            assert!(validate_uba_format("invalid").is_err());
            assert!(validate_uba_format("UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef").is_ok());
            assert!(validate_uba_format("UBA:invalid").is_err());
        }
    }
}
