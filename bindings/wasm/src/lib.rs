//! WebAssembly bindings for the UBA library
//! 
//! This module provides JavaScript/TypeScript compatible bindings for the UBA library
//! using wasm-bindgen. It exposes the main functionality for generating and retrieving
//! Unified Bitcoin Addresses.
//!
//! Note: WASM builds only support address generation and UBA parsing.
//! Nostr relay functionality is not available in WASM due to networking limitations.

use wasm_bindgen::prelude::*;
use js_sys::Array;
use serde_json;

use uba::types::{AddressType, BitcoinAddresses, UbaConfig, ParsedUba};
use uba::encryption::{derive_encryption_key, generate_random_key};
use uba::{UbaError, AddressGenerator, Network};

// Initialize panic hook for better error messages in the browser
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Parse a UBA string and extract its components (WASM-compatible version)
fn parse_uba_internal(uba: &str) -> Result<ParsedUba, UbaError> {
    if !uba.starts_with("UBA:") {
        return Err(UbaError::InvalidUbaFormat("UBA string must start with 'UBA:'".to_string()));
    }

    let content = &uba[4..]; // Remove "UBA:" prefix
    
    // Split by '&' to separate nostr_id from query parameters
    let parts: Vec<&str> = content.split('&').collect();
    let nostr_id = parts[0];
    
    // Validate nostr_id format (should be 64 character hex string)
    if nostr_id.len() != 64 || !nostr_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(UbaError::InvalidUbaFormat("Invalid Nostr ID format".to_string()));
    }
    
    // Parse query parameters if present
    let mut label = None;
    if parts.len() > 1 {
        for part in &parts[1..] {
            if let Some((key, value)) = part.split_once('=') {
                match key {
                    "label" => {
                        let decoded_label = urlencoding::decode(value)
                            .map_err(|_| UbaError::InvalidUbaFormat("Invalid label encoding".to_string()))?;
                        label = Some(decoded_label.to_string());
                    }
                    _ => {} // Ignore unknown parameters
                }
            }
        }
    }
    
    Ok(ParsedUba {
        nostr_id: nostr_id.to_string(),
        label,
    })
}

/// JavaScript-compatible configuration object for UBA operations
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsUbaConfig {
    inner: UbaConfig,
}

#[wasm_bindgen]
impl JsUbaConfig {
    /// Create a new default configuration
    #[wasm_bindgen(constructor)]
    pub fn new() -> JsUbaConfig {
        JsUbaConfig {
            inner: UbaConfig::default(),
        }
    }

    /// Set the Bitcoin network (0 = Bitcoin, 1 = Testnet, 2 = Signet, 3 = Regtest)
    #[wasm_bindgen(setter = network)]
    pub fn set_network(&mut self, network: u8) {
        self.inner.network = match network {
            0 => Network::Bitcoin,
            1 => Network::Testnet,
            2 => Network::Signet,
            3 => Network::Regtest,
            _ => Network::Bitcoin,
        };
    }

    /// Get the Bitcoin network (0 = Bitcoin, 1 = Testnet, 2 = Signet, 3 = Regtest)
    #[wasm_bindgen(getter = network)]
    pub fn get_network(&self) -> u8 {
        match self.inner.network {
            Network::Bitcoin => 0,
            Network::Testnet => 1,
            Network::Signet => 2,
            Network::Regtest => 3,
            _ => 0, // Default to Bitcoin for unknown networks
        }
    }

    /// Set whether to encrypt the data
    #[wasm_bindgen(setter = encrypt_data)]
    pub fn set_encrypt_data(&mut self, encrypt: bool) {
        self.inner.encrypt_data = encrypt;
    }

    /// Get whether data encryption is enabled
    #[wasm_bindgen(getter = encrypt_data)]
    pub fn get_encrypt_data(&self) -> bool {
        self.inner.encrypt_data
    }

    /// Set relay timeout in seconds
    #[wasm_bindgen(setter = relay_timeout)]
    pub fn set_relay_timeout(&mut self, timeout: u64) {
        self.inner.relay_timeout = timeout;
    }

    /// Get relay timeout in seconds
    #[wasm_bindgen(getter = relay_timeout)]
    pub fn get_relay_timeout(&self) -> u64 {
        self.inner.relay_timeout
    }

    /// Set maximum addresses per type
    #[wasm_bindgen(setter = max_addresses_per_type)]
    pub fn set_max_addresses_per_type(&mut self, count: usize) {
        self.inner.max_addresses_per_type = count;
    }

    /// Get maximum addresses per type
    #[wasm_bindgen(getter = max_addresses_per_type)]
    pub fn get_max_addresses_per_type(&self) -> usize {
        self.inner.max_addresses_per_type
    }

    /// Set encryption key from hex string
    #[wasm_bindgen]
    pub fn set_encryption_key_hex(&mut self, key_hex: &str) -> Result<(), JsValue> {
        self.inner.set_encryption_key_from_hex(key_hex)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Generate a random encryption key and return it as hex
    #[wasm_bindgen]
    pub fn generate_random_encryption_key(&mut self) -> String {
        let key = self.inner.generate_random_encryption_key();
        hex::encode(key)
    }

    /// Get encryption key as hex string
    #[wasm_bindgen]
    pub fn get_encryption_key_hex(&self) -> Option<String> {
        self.inner.get_encryption_key_hex()
    }

    /// Set custom relay URLs
    #[wasm_bindgen]
    pub fn set_custom_relays(&mut self, relays: &Array) {
        let relay_vec: Vec<String> = relays
            .iter()
            .filter_map(|val| val.as_string())
            .collect();
        self.inner.set_custom_relays(relay_vec);
    }

    /// Add a custom relay URL
    #[wasm_bindgen]
    pub fn add_custom_relay(&mut self, relay_url: &str) {
        self.inner.add_custom_relay(relay_url.to_string());
    }

    /// Get relay URLs as a JavaScript array
    #[wasm_bindgen]
    pub fn get_relay_urls(&self) -> Array {
        let urls = self.inner.get_relay_urls();
        let array = Array::new();
        for url in urls {
            array.push(&JsValue::from_str(&url));
        }
        array
    }

    /// Set address count for a specific address type
    /// Address types: 0=P2PKH, 1=P2SH, 2=P2WPKH, 3=P2TR, 4=Lightning, 5=Liquid
    #[wasm_bindgen]
    pub fn set_address_count(&mut self, address_type: u8, count: usize) {
        let addr_type = match address_type {
            0 => AddressType::P2PKH,
            1 => AddressType::P2SH,
            2 => AddressType::P2WPKH,
            3 => AddressType::P2TR,
            4 => AddressType::Lightning,
            5 => AddressType::Liquid,
            _ => return,
        };
        self.inner.set_address_count(addr_type, count);
    }

    /// Set counts for all Bitcoin L1 types at once
    #[wasm_bindgen]
    pub fn set_bitcoin_l1_counts(&mut self, count: usize) {
        self.inner.set_bitcoin_l1_counts(count);
    }

    /// Set counts for all address types at once
    #[wasm_bindgen]
    pub fn set_all_counts(&mut self, count: usize) {
        self.inner.set_all_counts(count);
    }

    /// Reset to use default public relays
    #[wasm_bindgen]
    pub fn use_default_relays(&mut self) {
        self.inner.use_default_relays();
    }
}

/// JavaScript-compatible result for address retrieval
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct JsBitcoinAddresses {
    inner: BitcoinAddresses,
}

#[wasm_bindgen]
impl JsBitcoinAddresses {
    /// Get all addresses as a flat array
    #[wasm_bindgen]
    pub fn get_all_addresses(&self) -> Array {
        let addresses = self.inner.get_all_addresses();
        let array = Array::new();
        for addr in addresses {
            array.push(&JsValue::from_str(&addr));
        }
        array
    }

    /// Get addresses by type
    /// Address types: 0=P2PKH, 1=P2SH, 2=P2WPKH, 3=P2TR, 4=Lightning, 5=Liquid
    #[wasm_bindgen]
    pub fn get_addresses_by_type(&self, address_type: u8) -> Option<Array> {
        let addr_type = match address_type {
            0 => AddressType::P2PKH,
            1 => AddressType::P2SH,
            2 => AddressType::P2WPKH,
            3 => AddressType::P2TR,
            4 => AddressType::Lightning,
            5 => AddressType::Liquid,
            _ => return None,
        };
        
        if let Some(addresses) = self.inner.get_addresses(&addr_type) {
            let array = Array::new();
            for addr in addresses {
                array.push(&JsValue::from_str(addr));
            }
            Some(array)
        } else {
            None
        }
    }

    /// Get the number of addresses
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the collection is empty
    #[wasm_bindgen]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get creation timestamp
    #[wasm_bindgen]
    pub fn get_created_at(&self) -> u64 {
        self.inner.created_at
    }

    /// Get version
    #[wasm_bindgen]
    pub fn get_version(&self) -> u32 {
        self.inner.version
    }

    /// Convert to JSON string
    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.inner)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

/// Generate Bitcoin addresses from a seed (WASM-compatible, no Nostr networking)
/// 
/// This function only generates addresses locally and does not interact with Nostr relays.
/// Use the native version for full UBA functionality with Nostr storage.
/// 
/// Note: This function may not work if secp256k1-sys compilation fails.
/// In that case, only utility functions (parsing, encryption) will be available.
#[wasm_bindgen]
pub fn generate_addresses(
    seed: &str,
    label: Option<String>,
    config: Option<JsUbaConfig>,
) -> Result<JsBitcoinAddresses, JsValue> {
    let final_config = config.map(|c| c.inner).unwrap_or_default();
    
    // Try to create address generator - this may panic if secp256k1 is not available
    // Wrap in catch_unwind to prevent panics from crossing the WASM boundary
    let generator_result = std::panic::catch_unwind(|| {
        AddressGenerator::new(final_config.clone())
    });
    
    match generator_result {
        Ok(generator) => {
            match generator.generate_addresses(seed, label) {
                Ok(addresses) => Ok(JsBitcoinAddresses { inner: addresses }),
                Err(e) => {
                    // If secp256k1 fails, provide helpful error message
                    if e.to_string().contains("secp256k1") {
                        Err(JsValue::from_str(&format!(
                            "Cryptographic library compilation failed. This is a known limitation in WASM builds. \
                            Error: {}. \
                            \
                            Solutions: \
                            1. Use the JavaScript address generation service instead \
                            2. Set up proper WASM compilation environment with LLVM \
                            3. Use the native Rust library for full functionality \
                            4. Use pre-generated addresses with create_addresses_from_data()",
                            e
                        )))
                    } else {
                        Err(JsValue::from_str(&e.to_string()))
                    }
                }
            }
        }
        Err(panic_info) => {
            // Convert panic to a proper JS error
            let panic_message = if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred during address generation".to_string()
            };
            
            Err(JsValue::from_str(&format!(
                "Address generation failed due to cryptographic library initialization error. \
                This is typically caused by secp256k1-sys compilation issues in WASM builds. \
                Panic details: {}. \
                \
                Solutions: \
                1. Use create_addresses_from_data() with pre-generated addresses \
                2. Use create_addresses_from_arrays() to manually construct address collections \
                3. Set up proper WASM compilation environment with LLVM support \
                4. Use the native Rust library for full functionality \
                5. Check is_crypto_available() before calling this function",
                panic_message
            )))
        }
    }
}

/// Create a BitcoinAddresses object from pre-generated address data
/// This is useful when secp256k1 compilation fails but you have addresses from other sources
#[wasm_bindgen]
pub fn create_addresses_from_data(
    addresses_json: &str,
) -> Result<JsBitcoinAddresses, JsValue> {
    let addresses: BitcoinAddresses = serde_json::from_str(addresses_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid address data JSON: {}", e)))?;
    
    Ok(JsBitcoinAddresses { inner: addresses })
}

/// Create a BitcoinAddresses object from individual address arrays
/// This allows JavaScript applications to create UBA-compatible address collections
/// when the built-in address generation fails
#[wasm_bindgen]
pub fn create_addresses_from_arrays(
    p2pkh_addresses: Option<Array>,
    p2sh_addresses: Option<Array>,
    p2wpkh_addresses: Option<Array>,
    p2tr_addresses: Option<Array>,
    liquid_addresses: Option<Array>,
    lightning_addresses: Option<Array>,
    label: Option<String>,
) -> Result<JsBitcoinAddresses, JsValue> {
    use std::collections::HashMap;
    
    let mut address_map = HashMap::new();
    
    // Helper function to convert JS array to Vec<String>
    let js_array_to_vec = |arr: Option<Array>| -> Vec<String> {
        arr.map(|a| {
            (0..a.length())
                .filter_map(|i| a.get(i).as_string())
                .collect()
        }).unwrap_or_default()
    };
    
    // Convert each address type
    if let Some(addrs) = p2pkh_addresses {
        let vec = js_array_to_vec(Some(addrs));
        if !vec.is_empty() {
            address_map.insert(AddressType::P2PKH, vec);
        }
    }
    
    if let Some(addrs) = p2sh_addresses {
        let vec = js_array_to_vec(Some(addrs));
        if !vec.is_empty() {
            address_map.insert(AddressType::P2SH, vec);
        }
    }
    
    if let Some(addrs) = p2wpkh_addresses {
        let vec = js_array_to_vec(Some(addrs));
        if !vec.is_empty() {
            address_map.insert(AddressType::P2WPKH, vec);
        }
    }
    
    if let Some(addrs) = p2tr_addresses {
        let vec = js_array_to_vec(Some(addrs));
        if !vec.is_empty() {
            address_map.insert(AddressType::P2TR, vec);
        }
    }
    
    if let Some(addrs) = liquid_addresses {
        let vec = js_array_to_vec(Some(addrs));
        if !vec.is_empty() {
            address_map.insert(AddressType::Liquid, vec);
        }
    }
    
    if let Some(addrs) = lightning_addresses {
        let vec = js_array_to_vec(Some(addrs));
        if !vec.is_empty() {
            address_map.insert(AddressType::Lightning, vec);
        }
    }
    
    // Create metadata if label provided
    let metadata = label.map(|l| uba::types::AddressMetadata {
        label: Some(l),
        description: None,
        xpub: None,
        derivation_paths: None,
    });
    
    // Create BitcoinAddresses structure
    let addresses = BitcoinAddresses {
        addresses: address_map,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: 1,
        metadata,
    };
    
            Ok(JsBitcoinAddresses { inner: addresses })
        }

/// Check if secp256k1 cryptographic functions are available in this WASM build
/// Returns true if address generation should work, false if only utilities are available
#[wasm_bindgen]
pub fn is_crypto_available() -> bool {
    // Try to create an address generator to test if secp256k1 is available
    match std::panic::catch_unwind(|| {
        let config = uba::types::UbaConfig::default();
        let _generator = AddressGenerator::new(config);
    }) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Get information about what functionality is available in this WASM build
#[wasm_bindgen]
pub fn get_build_info() -> JsValue {
    let crypto_available = is_crypto_available();
    
    let info = js_sys::Object::new();
    js_sys::Reflect::set(&info, &"cryptoAvailable".into(), &crypto_available.into()).unwrap();
    js_sys::Reflect::set(&info, &"version".into(), &"0.1.0".into()).unwrap();
    js_sys::Reflect::set(&info, &"target".into(), &"wasm32-unknown-unknown".into()).unwrap();
    
    let features = js_sys::Array::new();
    features.push(&"uba_parsing".into());
    features.push(&"encryption_utilities".into());
    features.push(&"address_utilities".into());
    
    if crypto_available {
        features.push(&"address_generation".into());
    } else {
        features.push(&"manual_address_creation".into());
    }
    
    js_sys::Reflect::set(&info, &"availableFeatures".into(), &features).unwrap();
    
    let limitations = js_sys::Array::new();
    limitations.push(&"no_nostr_networking".into());
    
    if !crypto_available {
        limitations.push(&"no_secp256k1_crypto".into());
    }
    
    js_sys::Reflect::set(&info, &"limitations".into(), &limitations).unwrap();
    
    info.into()
}

/// Parse a UBA string and extract its components
#[wasm_bindgen]
pub fn parse_uba_string(uba: &str) -> Result<JsValue, JsValue> {
    let parsed = parse_uba_internal(uba)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"nostrId".into(), &parsed.nostr_id.into())?;
    if let Some(label) = parsed.label {
        js_sys::Reflect::set(&obj, &"label".into(), &label.into())?;
    }
    
    Ok(obj.into())
}

/// Derive an encryption key from a passphrase using HKDF
#[wasm_bindgen]
pub fn derive_encryption_key_from_passphrase(
    passphrase: &str,
    salt: Option<String>,
) -> String {
    let salt_bytes = salt.map(|s| s.into_bytes());
    let key = derive_encryption_key(passphrase, salt_bytes.as_deref());
    hex::encode(key)
}

/// Generate a random 32-byte encryption key
#[wasm_bindgen]
pub fn generate_random_encryption_key() -> String {
    let key = generate_random_key();
    hex::encode(key)
}

/// Get the default public Nostr relays
#[wasm_bindgen]
pub fn get_default_public_relays() -> Array {
    let relays = uba::types::DEFAULT_PUBLIC_RELAYS;
    let array = Array::new();
    for relay in relays {
        array.push(&JsValue::from_str(relay));
    }
    array
}

/// Get an extended list of public Nostr relays
#[wasm_bindgen]
pub fn get_extended_public_relays() -> Array {
    let relays = uba::types::EXTENDED_PUBLIC_RELAYS;
    let array = Array::new();
    for relay in relays {
        array.push(&JsValue::from_str(relay));
    }
    array
}

/// Constants for address types
#[wasm_bindgen]
pub struct AddressTypes;

#[wasm_bindgen]
impl AddressTypes {
    #[wasm_bindgen(getter)]
    pub fn P2PKH() -> u8 { 0 }
    #[wasm_bindgen(getter)]
    pub fn P2SH() -> u8 { 1 }
    #[wasm_bindgen(getter)]
    pub fn P2WPKH() -> u8 { 2 }
    #[wasm_bindgen(getter)]
    pub fn P2TR() -> u8 { 3 }
    #[wasm_bindgen(getter)]
    pub fn Lightning() -> u8 { 4 }
    #[wasm_bindgen(getter)]
    pub fn Liquid() -> u8 { 5 }
}

/// Constants for Bitcoin networks
#[wasm_bindgen]
pub struct Networks;

#[wasm_bindgen]
impl Networks {
    #[wasm_bindgen(getter)]
    pub fn Bitcoin() -> u8 { 0 }
    #[wasm_bindgen(getter)]
    pub fn Testnet() -> u8 { 1 }
    #[wasm_bindgen(getter)]
    pub fn Signet() -> u8 { 2 }
    #[wasm_bindgen(getter)]
    pub fn Regtest() -> u8 { 3 }
} 