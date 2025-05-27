//! ChaCha20Poly1305 encryption implementation for UBA
//!
//! This module provides optional encryption functionality using ChaCha20Poly1305
//! authenticated encryption with HKDF for key derivation. This is intended for
//! advanced use cases where users need to encrypt UBA data before storage.
//!
//! **Note**: UBA's core mission is to simplify Bitcoin address sharing through
//! public, discoverable addresses. Encryption is optional and should only be
//! used when specifically needed for privacy-sensitive applications.
//!
//! Currently supports:
//! - Basic ChaCha20Poly1305 encryption/decryption
//! - Key derivation from passphrases
//! 
//! Future roadmap may include:
//! - NIP-04 encryption for Nostr compatibility (if community demand exists)
//! - NIP-17 Gift Wrap encryption for advanced privacy use cases
//! - Selective metadata encryption (keeping addresses public)

use crate::{Result, UbaError};
use base64::{engine::general_purpose, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Key, Nonce,
};
use hkdf::Hkdf;
use rand::RngCore;
use sha2::Sha256;

/// Encryption context for UBA operations
pub struct UbaEncryption {
    cipher: ChaCha20Poly1305,
}

impl UbaEncryption {
    /// Create a new encryption context with the given key
    pub fn new(key: [u8; 32]) -> Self {
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
        Self { cipher }
    }

    /// Encrypt data using ChaCha20Poly1305
    ///
    /// # Arguments
    /// * `data` - The data to encrypt (typically JSON)
    ///
    /// # Returns
    /// * `Ok(String)` - Base64 encoded encrypted data with nonce
    /// * `Err(UbaError)` - Encryption error
    pub fn encrypt(&self, data: &str) -> Result<String> {
        // Generate random 12-byte nonce for ChaCha20Poly1305
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let ciphertext = self
            .cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| UbaError::Encryption(format!("Failed to encrypt: {}", e)))?;

        // Combine nonce + ciphertext and encode as base64
        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(&combined))
    }

    /// Decrypt data using ChaCha20Poly1305
    ///
    /// # Arguments
    /// * `encrypted_data` - Base64 encoded encrypted data with nonce
    ///
    /// # Returns
    /// * `Ok(String)` - Decrypted plaintext data
    /// * `Err(UbaError)` - Decryption error
    pub fn decrypt(&self, encrypted_data: &str) -> Result<String> {
        // Decode base64
        let combined = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|e| UbaError::Encryption(format!("Failed to decode base64: {}", e)))?;

        if combined.len() < 12 {
            return Err(UbaError::Encryption(
                "Encrypted data too short, missing nonce".to_string(),
            ));
        }

        // Split nonce and ciphertext
        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| UbaError::Encryption(format!("Failed to decrypt: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| UbaError::Encryption(format!("Invalid UTF-8 in plaintext: {}", e)))
    }
}

/// Derive an encryption key from a passphrase using HKDF with proper error handling
///
/// This function derives a 32-byte encryption key from a passphrase using HKDF-SHA256.
/// This allows users to use memorable passphrases instead of raw 32-byte keys.
///
/// # Arguments
/// * `passphrase` - User-provided passphrase
/// * `salt` - Optional salt (if None, uses default UBA salt)
///
/// # Returns
/// * Result containing 32-byte derived key or error
pub fn derive_encryption_key_safe(passphrase: &str, salt: Option<&[u8]>) -> Result<[u8; 32]> {
    let default_salt = b"UBA-encryption-salt-v1";
    let used_salt = salt.unwrap_or(default_salt);

    let hk = Hkdf::<Sha256>::new(Some(used_salt), passphrase.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(b"UBA-encryption-key", &mut key)?;

    Ok(key)
}

/// Derive an encryption key from a passphrase using HKDF (backward compatibility)
///
/// This function derives a 32-byte encryption key from a passphrase using HKDF-SHA256.
/// This allows users to use memorable passphrases instead of raw 32-byte keys.
///
/// # Arguments
/// * `passphrase` - User-provided passphrase
/// * `salt` - Optional salt (if None, uses default UBA salt)
///
/// # Returns
/// * 32-byte derived key
/// 
/// # Panics
/// This function panics if key derivation fails. For error handling, use `derive_encryption_key_safe`.
pub fn derive_encryption_key(passphrase: &str, salt: Option<&[u8]>) -> [u8; 32] {
    derive_encryption_key_safe(passphrase, salt)
        .expect("Key derivation should not fail with valid inputs")
}

/// Generate a random 32-byte encryption key
pub fn generate_random_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    key
}

/// Utility function to encrypt JSON data if encryption is enabled
///
/// # Arguments
/// * `json_data` - The JSON string to potentially encrypt
/// * `encryption_key` - Optional encryption key
///
/// # Returns
/// * Encrypted data if key provided, original data if not
pub fn encrypt_if_enabled(json_data: &str, encryption_key: Option<&[u8; 32]>) -> Result<String> {
    match encryption_key {
        Some(key) => {
            let encryption = UbaEncryption::new(*key);
            encryption.encrypt(json_data)
        }
        None => Ok(json_data.to_string()),
    }
}

/// Utility function to decrypt JSON data if it was encrypted
///
/// # Arguments
/// * `data` - The potentially encrypted data
/// * `encryption_key` - Optional encryption key
///
/// # Returns
/// * Decrypted data if key provided and data is encrypted, original data otherwise
pub fn decrypt_if_needed(data: &str, encryption_key: Option<&[u8; 32]>) -> Result<String> {
    match encryption_key {
        Some(key) => {
            // Try to decrypt - if it fails, assume it's unencrypted
            let encryption = UbaEncryption::new(*key);
            encryption.decrypt(data).or_else(|_| Ok(data.to_string()))
        }
        None => Ok(data.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let key = generate_random_key();
        let encryption = UbaEncryption::new(key);

        let original = "Hello, encrypted world!";
        let encrypted = encryption.encrypt(original).unwrap();
        let decrypted = encryption.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_key_derivation() {
        let passphrase = "my secret passphrase";
        let key1 = derive_encryption_key_safe(passphrase, None).unwrap();
        let key2 = derive_encryption_key_safe(passphrase, None).unwrap();

        // Same passphrase should derive same key
        assert_eq!(key1, key2);

        // Different passphrase should derive different key
        let key3 = derive_encryption_key_safe("different passphrase", None).unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_json_encryption() {
        let json = r#"{"addresses": {"P2PKH": ["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"]}}"#;
        let key = generate_random_key();

        let encrypted = encrypt_if_enabled(json, Some(&key)).unwrap();
        let decrypted = decrypt_if_needed(&encrypted, Some(&key)).unwrap();

        assert_eq!(json, decrypted);
    }

    #[test]
    fn test_no_encryption_passthrough() {
        let json = r#"{"addresses": {"P2PKH": ["1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"]}}"#;

        let result = encrypt_if_enabled(json, None).unwrap();
        assert_eq!(json, result);

        let result = decrypt_if_needed(json, None).unwrap();
        assert_eq!(json, result);
    }

    #[test]
    fn test_key_derivation_safe() {
        let passphrase = "my secret passphrase";
        let key1 = derive_encryption_key_safe(passphrase, None).unwrap();
        let key2 = derive_encryption_key_safe(passphrase, None).unwrap();

        // Same passphrase should derive same key
        assert_eq!(key1, key2);

        // Different passphrase should derive different key
        let key3 = derive_encryption_key_safe("different passphrase", None).unwrap();
        assert_ne!(key1, key3);
    }
}
