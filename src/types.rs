//! Core types for the UBA library

use bitcoin::Network;
use hex;
use rand;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for UBA generation and retrieval
#[derive(Debug, Clone)]
pub struct UbaConfig {
    /// Bitcoin network to use (Mainnet, Testnet, etc.)
    pub network: Network,
    /// Whether to encrypt the address data in Nostr notes
    pub encrypt_data: bool,
    /// Optional encryption key (32 bytes) for encrypting JSON data sent to relays
    /// If None, no encryption is applied (backward compatible)
    pub encryption_key: Option<[u8; 32]>,
    /// Timeout for relay operations in seconds
    pub relay_timeout: u64,
    /// Maximum number of addresses to generate per address type (default fallback)
    pub max_addresses_per_type: usize,
    /// Specific address counts per type (overrides max_addresses_per_type if set)
    pub address_counts: HashMap<AddressType, usize>,
    /// Optional custom relay URLs to use instead of default public relays
    /// If None, will use DEFAULT_PUBLIC_RELAYS
    pub custom_relays: Option<Vec<String>>,
}

impl UbaConfig {
    /// Set the number of addresses to generate for a specific address type
    pub fn set_address_count(&mut self, address_type: AddressType, count: usize) {
        self.address_counts.insert(address_type, count);
    }

    /// Get the number of addresses to generate for a specific address type
    pub fn get_address_count(&self, address_type: &AddressType) -> usize {
        self.address_counts
            .get(address_type)
            .copied()
            .unwrap_or(self.max_addresses_per_type)
    }

    /// Set address counts for all Bitcoin L1 types at once
    pub fn set_bitcoin_l1_counts(&mut self, count: usize) {
        self.set_address_count(AddressType::P2PKH, count);
        self.set_address_count(AddressType::P2SH, count);
        self.set_address_count(AddressType::P2WPKH, count);
        self.set_address_count(AddressType::P2TR, count);
    }

    /// Set counts for all address types at once
    pub fn set_all_counts(&mut self, count: usize) {
        self.set_bitcoin_l1_counts(count);
        self.set_address_count(AddressType::Liquid, count);
        self.set_address_count(AddressType::Lightning, count);
        self.set_address_count(AddressType::Nostr, count);
    }

    /// Set encryption key from a hex string
    ///
    /// # Arguments
    /// * `key_hex` - 64-character hex string representing 32 bytes
    ///
    /// # Returns
    /// * `Ok(())` if key was set successfully
    /// * `Err` if hex string is invalid or wrong length
    pub fn set_encryption_key_from_hex(&mut self, key_hex: &str) -> Result<(), crate::UbaError> {
        if key_hex.len() != 64 {
            return Err(crate::UbaError::InvalidEncryptionKey(
                "Encryption key must be exactly 64 hex characters (32 bytes)".to_string(),
            ));
        }

        let key_bytes = hex::decode(key_hex).map_err(|e| {
            crate::UbaError::InvalidEncryptionKey(format!("Invalid hex string: {}", e))
        })?;

        if key_bytes.len() != 32 {
            return Err(crate::UbaError::InvalidEncryptionKey(
                "Encryption key must be exactly 32 bytes".to_string(),
            ));
        }

        let mut key_array = [0u8; 32];
        key_array.copy_from_slice(&key_bytes);
        self.encryption_key = Some(key_array);
        Ok(())
    }

    /// Set encryption key from raw bytes
    pub fn set_encryption_key(&mut self, key: [u8; 32]) {
        self.encryption_key = Some(key);
    }

    /// Generate a random encryption key
    pub fn generate_random_encryption_key(&mut self) -> [u8; 32] {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);
        self.encryption_key = Some(key);
        key
    }

    /// Check if encryption is enabled
    pub fn is_encryption_enabled(&self) -> bool {
        self.encryption_key.is_some()
    }

    /// Get encryption key as hex string (for display/storage)
    pub fn get_encryption_key_hex(&self) -> Option<String> {
        self.encryption_key.map(hex::encode)
    }

    /// Set custom relay URLs
    pub fn set_custom_relays(&mut self, relays: Vec<String>) {
        self.custom_relays = Some(relays);
    }

    /// Add a custom relay URL
    pub fn add_custom_relay(&mut self, relay_url: String) {
        match &mut self.custom_relays {
            Some(relays) => relays.push(relay_url),
            None => self.custom_relays = Some(vec![relay_url]),
        }
    }

    /// Get relay URLs to use (custom or default)
    pub fn get_relay_urls(&self) -> Vec<String> {
        self.custom_relays
            .clone()
            .unwrap_or_else(default_public_relays)
    }

    /// Reset to use default public relays
    pub fn use_default_relays(&mut self) {
        self.custom_relays = None;
    }
}

impl Default for UbaConfig {
    fn default() -> Self {
        Self {
            network: Network::Bitcoin,
            encrypt_data: false,
            encryption_key: None,
            relay_timeout: 10,
            max_addresses_per_type: 1,
            address_counts: HashMap::new(),
            custom_relays: None,
        }
    }
}

/// Represents different types of Bitcoin addresses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AddressType {
    /// Legacy P2PKH addresses (starts with 1)
    P2PKH,
    /// SegWit P2SH-wrapped addresses (starts with 3)
    P2SH,
    /// Native SegWit addresses (starts with bc1)
    P2WPKH,
    /// Taproot addresses (starts with bc1p)
    P2TR,
    /// Lightning Network invoice/address
    Lightning,
    /// Liquid sidechain address
    Liquid,
    /// Nostr public key
    Nostr,
}

impl AddressType {
    /// Get a human-readable description of the address type
    pub fn description(&self) -> &'static str {
        match self {
            AddressType::P2PKH => "Legacy Bitcoin address (P2PKH)",
            AddressType::P2SH => "SegWit-wrapped Bitcoin address (P2SH)",
            AddressType::P2WPKH => "Native SegWit Bitcoin address (P2WPKH)",
            AddressType::P2TR => "Taproot Bitcoin address (P2TR)",
            AddressType::Lightning => "Lightning Network address/invoice",
            AddressType::Liquid => "Liquid sidechain address",
            AddressType::Nostr => "Nostr public key (npub format)",
        }
    }
}

/// Collection of Bitcoin addresses across different layers and types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinAddresses {
    /// Mapping of address types to their corresponding addresses
    pub addresses: HashMap<AddressType, Vec<String>>,
    /// Optional metadata for the address collection
    pub metadata: Option<AddressMetadata>,
    /// Timestamp when the addresses were generated
    pub created_at: u64,
    /// Version of the address format for future compatibility
    pub version: u32,
}

impl BitcoinAddresses {
    /// Create a new empty BitcoinAddresses collection
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
            metadata: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 1,
        }
    }

    /// Add an address of a specific type
    pub fn add_address(&mut self, address_type: AddressType, address: String) {
        self.addresses
            .entry(address_type)
            .or_default()
            .push(address);
    }

    /// Get all addresses of a specific type
    pub fn get_addresses(&self, address_type: &AddressType) -> Option<&Vec<String>> {
        self.addresses.get(address_type)
    }

    /// Get all addresses as a flat vector
    pub fn get_all_addresses(&self) -> Vec<String> {
        self.addresses
            .values()
            .flat_map(|addresses| addresses.iter().cloned())
            .collect()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.addresses.is_empty()
    }

    /// Get the total number of addresses
    pub fn len(&self) -> usize {
        self.addresses.values().map(|v| v.len()).sum()
    }
}

impl Default for BitcoinAddresses {
    fn default() -> Self {
        Self::new()
    }
}

/// Optional metadata for address collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressMetadata {
    /// User-defined label for the address collection
    pub label: Option<String>,
    /// Description of the wallet or purpose
    pub description: Option<String>,
    /// Extended public key used for derivation (if applicable)
    pub xpub: Option<String>,
    /// Derivation paths used for address generation
    pub derivation_paths: Option<Vec<String>>,
}

/// Parsed UBA components
#[derive(Debug, Clone)]
pub struct ParsedUba {
    /// The Nostr event ID that contains the address data
    pub nostr_id: String,
    /// Optional label extracted from the UBA
    pub label: Option<String>,
}

/// UBA generation request
#[derive(Debug, Clone)]
pub struct UbaGenerationRequest {
    /// The seed phrase or private key material
    pub seed: String,
    /// Optional label for the UBA
    pub label: Option<String>,
    /// List of Nostr relay URLs
    pub relay_urls: Vec<String>,
    /// Configuration for the generation process
    pub config: UbaConfig,
}

/// UBA retrieval request
#[derive(Debug, Clone)]
pub struct UbaRetrievalRequest {
    /// The UBA string to parse and retrieve
    pub uba: String,
    /// List of Nostr relay URLs to query
    pub relay_urls: Vec<String>,
    /// Configuration for the retrieval process
    pub config: UbaConfig,
}

/// Get a curated list of reliable public Nostr relays
///
/// These relays are selected for reliability and geographical distribution.
/// Users can override this list by setting custom_relays in UbaConfig.
pub fn default_public_relays() -> Vec<String> {
    vec![
        // Reliable relays with good uptime and performance
        "wss://relay.damus.io".to_string(), // Damus (Cloudflare)
        "wss://nos.lol".to_string(),        // NOS (Hetzner)
        "wss://relay.snort.social".to_string(), // Snort (Cloudflare)
        "wss://nostr.wine".to_string(),     // Nostr Wine (Cloudflare)
        "wss://relay.nostr.band".to_string(), // Nostr Band (Hetzner) - supports search
        "wss://nostr.mutinywallet.com".to_string(), // Mutiny Wallet (Amazon)
        "wss://relay.primal.net".to_string(), // Primal (Cloudflare)
        "wss://relay.nostrati.com".to_string(), // Nostrati (Digital Ocean)
        "wss://nostr.sethforprivacy.com".to_string(), // Seth for Privacy (Privacy-focused)
        "wss://offchain.pub".to_string(),   // Offchain Pub (MULTACOM)
        "wss://relay.nostrplebs.com".to_string(), // Nostr Plebs (Hetzner)
        "wss://purplepag.es".to_string(),   // Purple Pages (Constant Company)
    ]
}

/// Extended public relay list for high-availability scenarios
///
/// This includes additional relays for redundancy and broader network coverage.
pub fn extended_public_relays() -> Vec<String> {
    let mut relays = default_public_relays();
    relays.extend(vec![
        "wss://relay.bitcoinpark.com".to_string(), // Bitcoin Park (Fastly)
        "wss://lightningrelay.com".to_string(),    // Lightning Relay (IONOS)
        "wss://relay.orangepill.dev".to_string(),  // Orange Pill (Oracle)
        "wss://nostr.bitcoiner.social".to_string(), // Bitcoiner Social (MULTACOM)
        "wss://relay.exit.pub".to_string(),        // Exit Pub (Amazon)
        "wss://purplerelay.com".to_string(),       // Purple Relay (Fastly)
        "wss://brb.io".to_string(),                // BRB (Cloudflare)
        "wss://nostr.milou.lol".to_string(),       // Milou (Cloudflare)
        "wss://relayable.org".to_string(),         // Relayable (Hetzner)
        "wss://relay.mostr.pub".to_string(),       // Mostr Pub (Cloudflare)
    ]);
    relays
}
