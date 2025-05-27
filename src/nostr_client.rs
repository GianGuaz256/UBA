//! Nostr client for publishing and retrieving UBA data

use crate::encryption::{decrypt_if_needed, encrypt_if_enabled};
use crate::error::{Result, UbaError};
use crate::types::BitcoinAddresses;

use nostr::{EventBuilder, EventId, Filter, Keys, Kind, Tag, Url};
use nostr_sdk::Client;
use serde_json;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::timeout;

/// Nostr client for UBA operations
pub struct NostrClient {
    client: Client,
    keys: Keys,
    timeout_duration: Duration,
}

impl NostrClient {
    /// Create a new Nostr client with generated keys
    pub fn new(timeout_seconds: u64) -> Result<Self> {
        let keys = Keys::generate();
        let client = Client::new(&keys);

        Ok(Self {
            client,
            keys,
            timeout_duration: Duration::from_secs(timeout_seconds),
        })
    }

    /// Create a new Nostr client with provided keys
    pub fn with_keys(keys: Keys, timeout_seconds: u64) -> Self {
        let client = Client::new(&keys);

        Self {
            client,
            keys,
            timeout_duration: Duration::from_secs(timeout_seconds),
        }
    }

    /// Connect to the specified relay URLs
    pub async fn connect_to_relays(&self, relay_urls: &[String]) -> Result<()> {
        for url_str in relay_urls {
            let url =
                Url::parse(url_str).map_err(|_| UbaError::InvalidRelayUrl(url_str.clone()))?;

            self.client
                .add_relay(url)
                .await
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?;
        }

        // Connect to all added relays
        self.client.connect().await;

        // Wait a moment for connections to establish
        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(())
    }

    /// Publish Bitcoin addresses as a Nostr event and return the event ID
    pub async fn publish_addresses(
        &self,
        addresses: &BitcoinAddresses,
        encrypt: bool,
    ) -> Result<String> {
        let content = if encrypt {
            // For now, we'll just serialize as JSON
            // TODO: Implement proper encryption using Nostr's NIP-04 or similar
            serde_json::to_string(addresses)?
        } else {
            serde_json::to_string(addresses)?
        };

        // Create a custom event for UBA data
        // Using Kind 1000-9999 range for application-specific events
        let kind = Kind::Custom(30000); // Parametrized replaceable event

        let mut tags = Vec::new();

        // Add a tag to identify this as UBA data
        tags.push(
            Tag::parse(&["uba", "bitcoin-addresses"])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        // Add metadata tags if available
        if let Some(metadata) = &addresses.metadata {
            if let Some(label) = &metadata.label {
                tags.push(
                    Tag::parse(&["label", label])
                        .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
                );
            }
        }

        // Add version tag
        tags.push(
            Tag::parse(&["version", &addresses.version.to_string()])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        let event = EventBuilder::new(kind, content, tags)
            .to_event(&self.keys)
            .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        // Publish the event with timeout
        let event_id = timeout(self.timeout_duration, self.client.send_event(event))
            .await
            .map_err(|_| UbaError::Timeout)?
            .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        Ok(event_id.to_hex())
    }

    /// Publish Bitcoin addresses with optional encryption
    pub async fn publish_addresses_with_encryption(
        &self,
        addresses: &BitcoinAddresses,
        encryption_key: Option<&[u8; 32]>,
    ) -> Result<String> {
        // Serialize addresses to JSON
        let json_content = serde_json::to_string(addresses)?;

        // Encrypt if key is provided
        let content = encrypt_if_enabled(&json_content, encryption_key)?;

        // Create a custom event for UBA data
        let kind = Kind::Custom(30000); // Parametrized replaceable event

        let mut tags = Vec::new();

        // Add a tag to identify this as UBA data
        tags.push(
            Tag::parse(&["uba", "bitcoin-addresses"])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        // Add encryption indicator if encrypted
        if encryption_key.is_some() {
            tags.push(
                Tag::parse(&["encrypted", "true"])
                    .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
            );
        }

        // Add metadata tags if available
        if let Some(metadata) = &addresses.metadata {
            if let Some(label) = &metadata.label {
                tags.push(
                    Tag::parse(&["label", label])
                        .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
                );
            }
        }

        // Add version tag
        tags.push(
            Tag::parse(&["version", &addresses.version.to_string()])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        let event = EventBuilder::new(kind, content, tags)
            .to_event(&self.keys)
            .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        // Publish the event with timeout
        let event_id = timeout(self.timeout_duration, self.client.send_event(event))
            .await
            .map_err(|_| UbaError::Timeout)?
            .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        Ok(event_id.to_hex())
    }

    /// Update Bitcoin addresses by creating a new event that replaces the old one
    /// 
    /// Since Nostr events are immutable, this creates a new event with updated content
    /// and includes a tag referencing the original event as "replaced"
    pub async fn update_addresses(
        &self,
        original_event_id: &str,
        updated_addresses: &BitcoinAddresses,
        encryption_key: Option<&[u8; 32]>,
    ) -> Result<String> {
        // First, verify the original event exists and we can access it
        self.verify_event_exists(original_event_id).await?;

        // Validate the updated addresses
        self.validate_address_update(updated_addresses)?;

        // Serialize addresses to JSON
        let json_content = serde_json::to_string(updated_addresses)?;

        // Encrypt if key is provided
        let content = encrypt_if_enabled(&json_content, encryption_key)?;

        // Create a custom event for UBA data
        let kind = Kind::Custom(30000); // Parametrized replaceable event

        let mut tags = Vec::new();

        // Add a tag to identify this as UBA data
        tags.push(
            Tag::parse(&["uba", "bitcoin-addresses"])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        // Add a tag to reference the original event being replaced
        tags.push(
            Tag::parse(&["replaces", original_event_id])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        // Add encryption indicator if encrypted
        if encryption_key.is_some() {
            tags.push(
                Tag::parse(&["encrypted", "true"])
                    .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
            );
        }

        // Add metadata tags if available
        if let Some(metadata) = &updated_addresses.metadata {
            if let Some(label) = &metadata.label {
                tags.push(
                    Tag::parse(&["label", label])
                        .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
                );
            }
        }

        // Add version tag
        tags.push(
            Tag::parse(&["version", &updated_addresses.version.to_string()])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        // Add update timestamp
        tags.push(
            Tag::parse(&["updated_at", &updated_addresses.created_at.to_string()])
                .map_err(|e| UbaError::NostrRelay(e.to_string()))?,
        );

        let event = EventBuilder::new(kind, content, tags)
            .to_event(&self.keys)
            .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        // Publish the event with timeout
        let event_id = timeout(self.timeout_duration, self.client.send_event(event))
            .await
            .map_err(|_| UbaError::Timeout)?
            .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        Ok(event_id.to_hex())
    }

    /// Verify that an event exists and is accessible
    async fn verify_event_exists(&self, event_id_hex: &str) -> Result<()> {
        let event_id = EventId::from_hex(event_id_hex)
            .map_err(|e| UbaError::InvalidUbaFormat(format!("Invalid event ID: {}", e)))?;

        // Create a filter to find the specific event
        let filter = Filter::new()
            .id(event_id)
            .kind(Kind::Custom(30000))
            .limit(1);

        // Try to retrieve the event
        let events = timeout(
            self.timeout_duration,
            self.client
                .get_events_of(vec![filter], Some(self.timeout_duration)),
        )
        .await
        .map_err(|_| UbaError::Timeout)?
        .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        if events.is_empty() {
            return Err(UbaError::EventNotFound(format!(
                "Event with ID {} not found",
                event_id_hex
            )));
        }

        Ok(())
    }

    /// Validate the updated address data
    fn validate_address_update(&self, addresses: &BitcoinAddresses) -> Result<()> {
        // Check if addresses collection is not empty
        if addresses.is_empty() {
            return Err(UbaError::UpdateValidation(
                "Updated addresses collection cannot be empty".to_string(),
            ));
        }

        // Validate that at least one address type has addresses
        let has_addresses = addresses.addresses.values().any(|addrs| !addrs.is_empty());
        if !has_addresses {
            return Err(UbaError::UpdateValidation(
                "At least one address type must contain addresses".to_string(),
            ));
        }

        // Validate individual addresses format (basic validation)
        for (addr_type, addr_list) in &addresses.addresses {
            for addr in addr_list {
                if addr.trim().is_empty() {
                    return Err(UbaError::UpdateValidation(format!(
                        "Empty address found in {:?} address type",
                        addr_type
                    )));
                }
            }
        }

        Ok(())
    }

    /// Retrieve Bitcoin addresses from a Nostr event ID
    pub async fn retrieve_addresses(&self, event_id_hex: &str) -> Result<BitcoinAddresses> {
        let event_id = EventId::from_hex(event_id_hex)
            .map_err(|e| UbaError::InvalidUbaFormat(format!("Invalid event ID: {}", e)))?;

        // Create a filter to find the specific event
        let filter = Filter::new()
            .id(event_id)
            .kind(Kind::Custom(30000))
            .limit(1);

        // Subscribe to the filter with timeout
        let events = timeout(
            self.timeout_duration,
            self.client
                .get_events_of(vec![filter], Some(self.timeout_duration)),
        )
        .await
        .map_err(|_| UbaError::Timeout)?
        .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        if events.is_empty() {
            return Err(UbaError::NoteNotFound(event_id_hex.to_string()));
        }

        let event = &events[0];

        // Verify this is UBA data by checking tags
        let has_uba_tag = event.tags.iter().any(|tag| {
            let tag_vec = tag.as_vec();
            tag_vec.len() >= 2 && tag_vec[0] == "uba" && tag_vec[1] == "bitcoin-addresses"
        });

        if !has_uba_tag {
            return Err(UbaError::InvalidUbaFormat(
                "Event is not UBA data".to_string(),
            ));
        }

        // Deserialize the content
        let addresses: BitcoinAddresses =
            serde_json::from_str(&event.content).map_err(UbaError::Json)?;

        Ok(addresses)
    }

    /// Retrieve Bitcoin addresses with optional decryption
    pub async fn retrieve_addresses_with_decryption(
        &self,
        event_id_hex: &str,
        encryption_key: Option<&[u8; 32]>,
    ) -> Result<BitcoinAddresses> {
        let event_id = EventId::from_hex(event_id_hex)
            .map_err(|e| UbaError::InvalidUbaFormat(format!("Invalid event ID: {}", e)))?;

        // Create a filter to find the specific event
        let filter = Filter::new()
            .id(event_id)
            .kind(Kind::Custom(30000))
            .limit(1);

        // Subscribe to the filter with timeout
        let events = timeout(
            self.timeout_duration,
            self.client
                .get_events_of(vec![filter], Some(self.timeout_duration)),
        )
        .await
        .map_err(|_| UbaError::Timeout)?
        .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

        if events.is_empty() {
            return Err(UbaError::NoteNotFound(event_id_hex.to_string()));
        }

        let event = &events[0];

        // Verify this is UBA data by checking tags
        let has_uba_tag = event.tags.iter().any(|tag| {
            let tag_vec = tag.as_vec();
            tag_vec.len() >= 2 && tag_vec[0] == "uba" && tag_vec[1] == "bitcoin-addresses"
        });

        if !has_uba_tag {
            return Err(UbaError::InvalidUbaFormat(
                "Event is not UBA data".to_string(),
            ));
        }

        // Check if content is encrypted
        let is_encrypted = event.tags.iter().any(|tag| {
            let tag_vec = tag.as_vec();
            tag_vec.len() >= 2 && tag_vec[0] == "encrypted" && tag_vec[1] == "true"
        });

        // Decrypt if needed
        let content = if is_encrypted || encryption_key.is_some() {
            decrypt_if_needed(&event.content, encryption_key)?
        } else {
            event.content.clone()
        };

        // Deserialize the content
        let addresses: BitcoinAddresses = serde_json::from_str(&content).map_err(UbaError::Json)?;

        Ok(addresses)
    }

    /// Get the public key of this client
    pub fn public_key(&self) -> String {
        self.keys.public_key().to_hex()
    }

    /// Disconnect from all relays
    pub async fn disconnect(&self) {
        let _ = self.client.disconnect().await;
    }
}

/// Generate a deterministic Nostr key from a seed
pub fn generate_nostr_keys_from_seed(seed: &str) -> Result<Keys> {
    // Use the seed to generate deterministic keys
    // This ensures the same seed always produces the same Nostr identity
    use bitcoin::hashes::{sha256, Hash};

    let seed_bytes = if seed.len() == 64 {
        // Assume hex-encoded
        hex::decode(seed)?
    } else {
        // Use BIP39 seed
        let mnemonic = bip39::Mnemonic::from_str(seed)?;
        mnemonic.to_seed("").to_vec()
    };

    // Hash the seed to get a 32-byte key
    let hash = sha256::Hash::hash(&seed_bytes);
    let secret_key = nostr::SecretKey::from_slice(hash.as_ref())
        .map_err(|e| UbaError::NostrRelay(e.to_string()))?;

    Ok(Keys::new(secret_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AddressType;

    #[tokio::test]
    async fn test_nostr_client_creation() {
        let client = NostrClient::new(10);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_deterministic_key_generation() {
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let keys1 = generate_nostr_keys_from_seed(seed);
        let keys2 = generate_nostr_keys_from_seed(seed);

        assert!(keys1.is_ok());
        assert!(keys2.is_ok());
        assert_eq!(keys1.unwrap().public_key(), keys2.unwrap().public_key());
    }

    #[test]
    fn test_bitcoin_addresses_serialization() {
        let mut addresses = BitcoinAddresses::new();
        addresses.add_address(AddressType::P2PKH, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string());
        addresses.add_address(AddressType::P2WPKH, "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string());

        let json = serde_json::to_string(&addresses).unwrap();
        let deserialized: BitcoinAddresses = serde_json::from_str(&json).unwrap();

        assert_eq!(addresses.len(), deserialized.len());
        assert_eq!(
            addresses.get_addresses(&AddressType::P2PKH),
            deserialized.get_addresses(&AddressType::P2PKH)
        );
    }

    #[test]
    fn test_validate_address_update_empty_collection() {
        let client = NostrClient::new(10).unwrap();
        let empty_addresses = BitcoinAddresses::new();
        
        let result = client.validate_address_update(&empty_addresses);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbaError::UpdateValidation(_)));
    }

    #[test]
    fn test_validate_address_update_no_addresses_in_types() {
        let client = NostrClient::new(10).unwrap();
        let mut addresses = BitcoinAddresses::new();
        // Add empty address lists
        addresses.addresses.insert(AddressType::P2PKH, vec![]);
        addresses.addresses.insert(AddressType::Lightning, vec![]);
        
        let result = client.validate_address_update(&addresses);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbaError::UpdateValidation(_)));
    }

    #[test]
    fn test_validate_address_update_empty_address_string() {
        let client = NostrClient::new(10).unwrap();
        let mut addresses = BitcoinAddresses::new();
        addresses.add_address(AddressType::P2PKH, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string());
        addresses.add_address(AddressType::P2PKH, "".to_string()); // Empty address
        
        let result = client.validate_address_update(&addresses);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbaError::UpdateValidation(_)));
    }

    #[test]
    fn test_validate_address_update_whitespace_only_address() {
        let client = NostrClient::new(10).unwrap();
        let mut addresses = BitcoinAddresses::new();
        addresses.add_address(AddressType::P2PKH, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string());
        addresses.add_address(AddressType::P2PKH, "   ".to_string()); // Whitespace only
        
        let result = client.validate_address_update(&addresses);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbaError::UpdateValidation(_)));
    }

    #[test]
    fn test_validate_address_update_valid_addresses() {
        let client = NostrClient::new(10).unwrap();
        let mut addresses = BitcoinAddresses::new();
        addresses.add_address(AddressType::P2PKH, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string());
        addresses.add_address(AddressType::P2WPKH, "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string());
        addresses.add_address(AddressType::Lightning, "lnbc1pvjluezpp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqdpl2pkx2ctnv5sxxmmwwd5kgetjypeh2ursdae8g6twvus8g6rfwvs8qun0dfjkxaq8rkx3yf5tcsyz3d73gafnh3cax9rn449d9p5uxz9ezhhypd0elx87sjle52x86fux2ypatgddc6k63n7erqz25le42c4u4ecky03ylcqca784w".to_string());
        
        let result = client.validate_address_update(&addresses);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_address_update_mixed_valid_invalid() {
        let client = NostrClient::new(10).unwrap();
        let mut addresses = BitcoinAddresses::new();
        addresses.add_address(AddressType::P2PKH, "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa".to_string());
        addresses.add_address(AddressType::Lightning, "".to_string()); // Invalid empty
        
        let result = client.validate_address_update(&addresses);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), UbaError::UpdateValidation(_)));
    }
}
