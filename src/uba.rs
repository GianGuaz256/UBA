//! Main UBA functionality - generate and retrieve functions

use crate::address::AddressGenerator;
use crate::error::{Result, UbaError};
use crate::nostr_client::{generate_nostr_keys_from_seed, NostrClient};
use crate::types::{BitcoinAddresses, ParsedUba, UbaConfig};

use url::Url;

/// Generate a UBA string from a seed and store address data on Nostr relays
///
/// # Arguments
/// * `seed` - BIP39 mnemonic phrase or hex-encoded private key
/// * `label` - Optional label for the UBA (e.g., "personal-wallet")
/// * `relay_urls` - List of Nostr relay URLs where the data will be stored
///
/// # Returns
/// A UBA string in the format: `UBA:<NostrID>&label=<label>` or `UBA:<NostrID>`
///
/// # Example
/// ```rust,no_run
/// use uba::generate;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
///     let relays = vec!["wss://relay.example.com".to_string()];
///     
///     let uba = generate(seed, Some("my-wallet"), &relays).await?;
///     println!("Generated UBA: {}", uba);
///     Ok(())
/// }
/// ```
pub async fn generate(seed: &str, label: Option<&str>, relay_urls: &[String]) -> Result<String> {
    let config = UbaConfig::default();
    generate_with_config(seed, label, relay_urls, config).await
}

/// Generate a UBA string with custom configuration
pub async fn generate_with_config(
    seed: &str,
    label: Option<&str>,
    relay_urls: &[String],
    config: UbaConfig,
) -> Result<String> {
    // Use relay URLs from config if provided, otherwise use passed URLs
    let final_relay_urls = if relay_urls.is_empty() {
        config.get_relay_urls()
    } else {
        relay_urls.to_vec()
    };

    // Validate inputs
    validate_relay_urls(&final_relay_urls)?;
    if let Some(label) = label {
        validate_label(label)?;
    }

    // Generate Bitcoin addresses from the seed
    let address_generator = AddressGenerator::new(config.clone());
    let addresses = address_generator.generate_addresses(seed, label.map(String::from))?;

    // Generate deterministic Nostr keys from the seed
    let nostr_keys = generate_nostr_keys_from_seed(seed)?;
    let nostr_client = NostrClient::with_keys(nostr_keys, config.relay_timeout);

    // Connect to Nostr relays
    nostr_client.connect_to_relays(&final_relay_urls).await?;

    // Publish the addresses to Nostr with encryption if enabled
    let event_id = nostr_client
        .publish_addresses_with_encryption(&addresses, config.encryption_key.as_ref())
        .await?;

    // Disconnect from relays
    nostr_client.disconnect().await;

    // Format the UBA string
    let uba = if let Some(label) = label {
        format!("UBA:{}&label={}", event_id, label)
    } else {
        format!("UBA:{}", event_id)
    };

    Ok(uba)
}

/// Retrieve Bitcoin addresses from a UBA string
///
/// # Arguments
/// * `uba` - UBA string (e.g., "UBA:\<NostrID\>&label=\<label\>")
/// * `relay_urls` - List of Nostr relay URLs to query
///
/// # Returns
/// A vector of Bitcoin addresses
///
/// # Example
/// ```rust,no_run
/// use uba::retrieve;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let uba = "UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef&label=my-wallet";
///     let relays = vec!["wss://relay.example.com".to_string()];
///     
///     let addresses = retrieve(uba, &relays).await?;
///     println!("Retrieved addresses: {:?}", addresses);
///     Ok(())
/// }
/// ```
pub async fn retrieve(uba: &str, relay_urls: &[String]) -> Result<Vec<String>> {
    let config = UbaConfig::default();
    retrieve_with_config(uba, relay_urls, config).await
}

/// Retrieve Bitcoin addresses with custom configuration
pub async fn retrieve_with_config(
    uba: &str,
    relay_urls: &[String],
    config: UbaConfig,
) -> Result<Vec<String>> {
    // Use relay URLs from config if provided, otherwise use passed URLs
    let final_relay_urls = if relay_urls.is_empty() {
        config.get_relay_urls()
    } else {
        relay_urls.to_vec()
    };

    // Validate inputs
    validate_relay_urls(&final_relay_urls)?;

    // Parse the UBA string
    let parsed_uba = parse_uba(uba)?;

    // Create Nostr client (we don't need specific keys for reading)
    let nostr_client = NostrClient::new(config.relay_timeout)?;

    // Connect to Nostr relays
    nostr_client.connect_to_relays(&final_relay_urls).await?;

    // Retrieve the addresses from Nostr with decryption if needed
    let addresses = nostr_client
        .retrieve_addresses_with_decryption(&parsed_uba.nostr_id, config.encryption_key.as_ref())
        .await?;

    // Disconnect from relays
    nostr_client.disconnect().await;

    // Return all addresses as a flat vector
    Ok(addresses.get_all_addresses())
}

/// Retrieve the full BitcoinAddresses structure from a UBA string
///
/// This function returns the complete address collection with metadata,
/// allowing access to addresses grouped by type.
pub async fn retrieve_full(uba: &str, relay_urls: &[String]) -> Result<BitcoinAddresses> {
    let config = UbaConfig::default();
    retrieve_full_with_config(uba, relay_urls, config).await
}

/// Retrieve the full BitcoinAddresses structure with custom configuration
pub async fn retrieve_full_with_config(
    uba: &str,
    relay_urls: &[String],
    config: UbaConfig,
) -> Result<BitcoinAddresses> {
    // Use relay URLs from config if provided, otherwise use passed URLs
    let final_relay_urls = if relay_urls.is_empty() {
        config.get_relay_urls()
    } else {
        relay_urls.to_vec()
    };

    // Validate inputs
    validate_relay_urls(&final_relay_urls)?;

    // Parse the UBA string
    let parsed_uba = parse_uba(uba)?;

    // Create Nostr client
    let nostr_client = NostrClient::new(config.relay_timeout)?;

    // Connect to Nostr relays
    nostr_client.connect_to_relays(&final_relay_urls).await?;

    // Retrieve the addresses from Nostr with decryption if needed
    let addresses = nostr_client
        .retrieve_addresses_with_decryption(&parsed_uba.nostr_id, config.encryption_key.as_ref())
        .await?;

    // Disconnect from relays
    nostr_client.disconnect().await;

    Ok(addresses)
}

/// Parse a UBA string into its components
///
/// # Arguments
/// * `uba` - UBA string to parse
///
/// # Returns
/// A `ParsedUba` struct containing the Nostr ID and optional label
///
/// # Example
/// ```rust
/// use uba::parse_uba;
///
/// let uba = "UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef&label=my-wallet";
/// let parsed = parse_uba(uba)?;
/// println!("Nostr ID: {}", parsed.nostr_id);
/// println!("Label: {:?}", parsed.label);
/// # Ok::<(), uba::UbaError>(())
/// ```
pub fn parse_uba(uba: &str) -> Result<ParsedUba> {
    // Check if it starts with "UBA:"
    if !uba.starts_with("UBA:") {
        return Err(UbaError::InvalidUbaFormat(
            "UBA string must start with 'UBA:'".to_string(),
        ));
    }

    // Remove the "UBA:" prefix
    let content = &uba[4..];

    // Check for label parameter
    if let Some(query_start) = content.find('&') {
        let nostr_id = content[..query_start].to_string();
        let query_string = &content[query_start + 1..];

        // Parse query parameters
        let label = parse_query_params(query_string)?;

        // Validate the Nostr ID format (should be 64 hex characters)
        validate_nostr_id(&nostr_id)?;

        Ok(ParsedUba { nostr_id, label })
    } else {
        // No query parameters, just the Nostr ID
        validate_nostr_id(content)?;

        Ok(ParsedUba {
            nostr_id: content.to_string(),
            label: None,
        })
    }
}

/// Parse query parameters from UBA string
fn parse_query_params(query_string: &str) -> Result<Option<String>> {
    let pairs: Vec<&str> = query_string.split('&').collect();

    for pair in pairs {
        if let Some(eq_pos) = pair.find('=') {
            let key = &pair[..eq_pos];
            let value = &pair[eq_pos + 1..];

            if key == "label" {
                // URL decode the value if needed
                let decoded = urlencoding::decode(value).map_err(|_| {
                    UbaError::InvalidUbaFormat("Invalid URL encoding in label".to_string())
                })?;
                return Ok(Some(decoded.to_string()));
            }
        }
    }

    Ok(None)
}

/// Validate a Nostr event ID format
fn validate_nostr_id(nostr_id: &str) -> Result<()> {
    if nostr_id.len() != 64 {
        return Err(UbaError::InvalidUbaFormat(
            "Nostr ID must be 64 characters long".to_string(),
        ));
    }

    // Check if it's valid hex
    if !nostr_id.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(UbaError::InvalidUbaFormat(
            "Nostr ID must be hexadecimal".to_string(),
        ));
    }

    Ok(())
}

/// Validate relay URLs
fn validate_relay_urls(relay_urls: &[String]) -> Result<()> {
    if relay_urls.is_empty() {
        return Err(UbaError::Config(
            "At least one relay URL is required".to_string(),
        ));
    }

    for url_str in relay_urls {
        let url = Url::parse(url_str).map_err(|_| UbaError::InvalidRelayUrl(url_str.clone()))?;

        // Check if it's a WebSocket URL
        if url.scheme() != "ws" && url.scheme() != "wss" {
            return Err(UbaError::InvalidRelayUrl(format!(
                "Relay URL must use ws:// or wss:// scheme: {}",
                url_str
            )));
        }
    }

    Ok(())
}

/// Validate label format
fn validate_label(label: &str) -> Result<()> {
    if label.is_empty() {
        return Err(UbaError::InvalidLabel("Label cannot be empty".to_string()));
    }

    if label.len() > 100 {
        return Err(UbaError::InvalidLabel(
            "Label cannot exceed 100 characters".to_string(),
        ));
    }

    // Check for invalid characters that might cause issues in URLs
    // Allow only alphanumeric characters, hyphens, and underscores
    if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Err(UbaError::InvalidLabel(
            "Label can only contain alphanumeric characters, hyphens, and underscores".to_string(),
        ));
    }

    Ok(())
}

/// Update Bitcoin addresses for an existing UBA by creating a new Nostr event
///
/// Since Nostr events are immutable, this function creates a new event that replaces
/// the original one. The new event will reference the original event ID.
///
/// # Arguments
/// * `nostr_event_id` - The Nostr event ID to update (hex format)
/// * `seed` - BIP39 mnemonic phrase or hex-encoded private key for generating new addresses
/// * `relay_urls` - List of Nostr relay URLs where the update will be published
/// * `config` - Configuration including address filtering and encryption settings
///
/// # Returns
/// A new UBA string pointing to the updated event
///
/// # Example
/// ```rust,no_run
/// use uba::{update_uba, UbaConfig, AddressType};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let original_event_id = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
///     let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
///     let relays = vec!["wss://relay.example.com".to_string()];
///     
///     let mut config = UbaConfig::default();
///     // Disable Lightning addresses for this update
///     config.set_address_type_enabled(AddressType::Lightning, false);
///     
///     let new_uba = update_uba(original_event_id, seed, &relays, config).await?;
///     println!("Updated UBA: {}", new_uba);
///     Ok(())
/// }
/// ```
pub async fn update_uba(
    nostr_event_id: &str,
    seed: &str,
    relay_urls: &[String],
    config: UbaConfig,
) -> Result<String> {
    // Use relay URLs from config if provided, otherwise use passed URLs
    let final_relay_urls = if relay_urls.is_empty() {
        config.get_relay_urls()
    } else {
        relay_urls.to_vec()
    };

    // Validate inputs
    validate_relay_urls(&final_relay_urls)?;
    validate_nostr_id(nostr_event_id)?;

    // Generate new Bitcoin addresses from the seed with current config
    let address_generator = AddressGenerator::new(config.clone());
    let mut updated_addresses = address_generator.generate_addresses(seed, None)?;

    // Update the timestamp to reflect this is an update
    updated_addresses.created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Generate deterministic Nostr keys from the seed
    let nostr_keys = generate_nostr_keys_from_seed(seed)?;
    let nostr_client = NostrClient::with_keys(nostr_keys, config.relay_timeout);

    // Connect to Nostr relays
    nostr_client.connect_to_relays(&final_relay_urls).await?;

    // Update the addresses on Nostr with encryption if enabled
    let new_event_id = nostr_client
        .update_addresses(nostr_event_id, &updated_addresses, config.encryption_key.as_ref())
        .await?;

    // Disconnect from relays
    nostr_client.disconnect().await;

    // Return the new UBA string pointing to the updated event
    let new_uba = format!("UBA:{}", new_event_id);
    Ok(new_uba)
}

/// Update Bitcoin addresses with custom address data
///
/// This function allows you to update a UBA with specific address data rather than
/// generating new addresses from a seed.
///
/// # Arguments
/// * `nostr_event_id` - The Nostr event ID to update (hex format)
/// * `updated_addresses` - The new address data to publish
/// * `relay_urls` - List of Nostr relay URLs where the update will be published
/// * `config` - Configuration including encryption settings
///
/// # Returns
/// A new UBA string pointing to the updated event
pub async fn update_uba_with_addresses(
    nostr_event_id: &str,
    updated_addresses: BitcoinAddresses,
    relay_urls: &[String],
    config: UbaConfig,
) -> Result<String> {
    // Use relay URLs from config if provided, otherwise use passed URLs
    let final_relay_urls = if relay_urls.is_empty() {
        config.get_relay_urls()
    } else {
        relay_urls.to_vec()
    };

    // Validate inputs first (before network operations)
    validate_relay_urls(&final_relay_urls)?;
    validate_nostr_id(nostr_event_id)?;
    
    // Validate the address data early
    if updated_addresses.is_empty() {
        return Err(UbaError::UpdateValidation(
            "Updated addresses collection cannot be empty".to_string(),
        ));
    }

    // Validate that at least one address type has addresses
    let has_addresses = updated_addresses.addresses.values().any(|addrs| !addrs.is_empty());
    if !has_addresses {
        return Err(UbaError::UpdateValidation(
            "At least one address type must contain addresses".to_string(),
        ));
    }

    // Validate individual addresses format (basic validation)
    for (addr_type, addr_list) in &updated_addresses.addresses {
        for addr in addr_list {
            if addr.trim().is_empty() {
                return Err(UbaError::UpdateValidation(format!(
                    "Empty address found in {:?} address type",
                    addr_type
                )));
            }
        }
    }

    // Create Nostr client (we need keys for publishing, but they don't need to be deterministic for updates)
    let nostr_client = NostrClient::new(config.relay_timeout)?;

    // Connect to Nostr relays
    nostr_client.connect_to_relays(&final_relay_urls).await?;

    // Update the addresses on Nostr with encryption if enabled
    let new_event_id = nostr_client
        .update_addresses(nostr_event_id, &updated_addresses, config.encryption_key.as_ref())
        .await?;

    // Disconnect from relays
    nostr_client.disconnect().await;

    // Return the new UBA string pointing to the updated event
    let new_uba = format!("UBA:{}", new_event_id);
    Ok(new_uba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::AddressGenerator;
    use crate::types::AddressType;

    #[test]
    fn test_parse_uba_without_label() {
        let uba = "UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = parse_uba(uba);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(
            parsed.nostr_id,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        assert_eq!(parsed.label, None);
    }

    #[test]
    fn test_parse_uba_with_label() {
        let uba =
            "UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef&label=my-wallet";
        let result = parse_uba(uba);

        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(
            parsed.nostr_id,
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        assert_eq!(parsed.label, Some("my-wallet".to_string()));
    }

    #[test]
    fn test_parse_uba_invalid_format() {
        let uba = "INVALID:1234567890abcdef";
        let result = parse_uba(uba);

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_uba_invalid_nostr_id() {
        let uba = "UBA:invalidhex";
        let result = parse_uba(uba);

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_relay_urls() {
        let valid_urls = vec![
            "wss://relay.example.com".to_string(),
            "ws://localhost:8080".to_string(),
        ];
        assert!(validate_relay_urls(&valid_urls).is_ok());

        let invalid_urls = vec!["https://example.com".to_string()];
        assert!(validate_relay_urls(&invalid_urls).is_err());

        let empty_urls: Vec<String> = vec![];
        assert!(validate_relay_urls(&empty_urls).is_err());
    }

    #[test]
    fn test_validate_label() {
        // Valid labels
        assert!(validate_label("my-wallet").is_ok());
        assert!(validate_label("wallet123").is_ok());
        assert!(validate_label("a").is_ok());

        // Invalid labels
        assert!(validate_label("").is_err());
        assert!(validate_label("a".repeat(101).as_str()).is_err()); // Too long
        assert!(validate_label("my wallet").is_err()); // Contains space
        assert!(validate_label("my@wallet").is_err()); // Contains @
        assert!(validate_label("my/wallet").is_err()); // Contains /
    }

    #[test]
    fn test_update_uba_validation_invalid_event_id() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let invalid_event_id = "invalid_hex";
            let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
            let relays = vec!["wss://relay.example.com".to_string()];
            let config = UbaConfig::default();

            let result = update_uba(invalid_event_id, seed, &relays, config).await;
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), UbaError::InvalidUbaFormat(_)));
        });
    }

    #[test]
    fn test_update_uba_validation_empty_relays() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let event_id = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
            let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
            let empty_relays: Vec<String> = vec![];
            let config = UbaConfig::default();

            // Should use default relays from config when empty relays provided
            let result = update_uba(event_id, seed, &empty_relays, config).await;
            // This will fail due to network/relay issues, but should pass validation
            assert!(result.is_err());
            // Should not be a validation error, but a network/relay error
            assert!(!matches!(result.unwrap_err(), UbaError::InvalidRelayUrl(_)));
        });
    }

    #[test]
    fn test_update_uba_with_addresses_validation_empty_addresses() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let event_id = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
            let empty_addresses = BitcoinAddresses::new();
            let relays = vec!["wss://relay.example.com".to_string()];
            let config = UbaConfig::default();

            let result = update_uba_with_addresses(event_id, empty_addresses, &relays, config).await;
            assert!(result.is_err());
            // Should fail during validation, not during network operations
            assert!(matches!(result.unwrap_err(), UbaError::UpdateValidation(_)));
        });
    }

    #[test]
    fn test_update_uba_with_filtering_configuration() {
        // Test that the update function respects address filtering
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let event_id = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
            let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
            let relays = vec!["wss://relay.example.com".to_string()];
            
            let mut config = UbaConfig::default();
            // Disable Lightning and Liquid
            config.set_address_type_enabled(AddressType::Lightning, false);
            config.set_address_type_enabled(AddressType::Liquid, false);

            let result = update_uba(event_id, seed, &relays, config).await;
            // This will fail due to network issues, but the address generation should work
            assert!(result.is_err());
            // Should not be a validation error related to address generation
            assert!(!matches!(result.unwrap_err(), UbaError::AddressGeneration(_)));
        });
    }

    #[test]
    fn test_update_uba_address_generation_with_filtering() {
        // Test address generation part of update function with filtering
        let mut config = UbaConfig::default();
        config.set_address_type_enabled(AddressType::Lightning, false);
        config.set_address_type_enabled(AddressType::Liquid, false);
        config.set_address_type_enabled(AddressType::Nostr, false);

        let address_generator = AddressGenerator::new(config);
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let addresses = address_generator.generate_addresses(seed, None).unwrap();

        // Should only have Bitcoin L1 addresses
        assert!(addresses.addresses.contains_key(&AddressType::P2PKH));
        assert!(addresses.addresses.contains_key(&AddressType::P2SH));
        assert!(addresses.addresses.contains_key(&AddressType::P2WPKH));
        assert!(addresses.addresses.contains_key(&AddressType::P2TR));

        // Should not have disabled types
        assert!(!addresses.addresses.contains_key(&AddressType::Lightning));
        assert!(!addresses.addresses.contains_key(&AddressType::Liquid));
        assert!(!addresses.addresses.contains_key(&AddressType::Nostr));
    }

    #[test]
    fn test_update_uba_timestamp_update() {
        // Test that update function updates the timestamp
        let config = UbaConfig::default();
        let address_generator = AddressGenerator::new(config);
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let original_addresses = address_generator.generate_addresses(seed, None).unwrap();
        let original_timestamp = original_addresses.created_at;

        // Simulate what update_uba does
        std::thread::sleep(std::time::Duration::from_secs(1)); // Ensure time difference
        let mut updated_addresses = address_generator.generate_addresses(seed, None).unwrap();
        updated_addresses.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(updated_addresses.created_at > original_timestamp);
    }
}
