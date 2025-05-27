//! Bitcoin address generation from seeds

use crate::error::{Result, UbaError};
use crate::types::{AddressMetadata, AddressType, BitcoinAddresses, UbaConfig};

use bip39::Mnemonic;
use bitcoin::{
    bip32::{ChildNumber, DerivationPath, Xpriv},
    secp256k1::Secp256k1,
    Address, PrivateKey, PublicKey, XOnlyPublicKey,
};
use std::str::FromStr;

// Liquid support
use elements::Address as LiquidAddress;

// Lightning support
use secp256k1::PublicKey as Secp256k1PublicKey;

// Nostr support
use nostr::{self, ToBech32};

/// Address generator for creating Bitcoin addresses from seeds
pub struct AddressGenerator {
    config: UbaConfig,
    secp: Secp256k1<bitcoin::secp256k1::All>,
}

impl AddressGenerator {
    /// Create a new address generator with the given configuration
    pub fn new(config: UbaConfig) -> Self {
        Self {
            config,
            secp: Secp256k1::new(),
        }
    }

    /// Generate Bitcoin addresses from a seed phrase or private key
    ///
    /// # Arguments
    /// * `seed_input` - BIP39 mnemonic phrase or hex-encoded private key
    /// * `label` - Optional label for the address collection
    ///
    /// # Returns
    /// A `BitcoinAddresses` collection containing addresses for different types
    pub fn generate_addresses(
        &self,
        seed_input: &str,
        label: Option<String>,
    ) -> Result<BitcoinAddresses> {
        let master_key = self.derive_master_key(seed_input)?;
        let mut addresses = BitcoinAddresses::new();

        // Set metadata
        addresses.metadata = Some(AddressMetadata {
            label: label.clone(),
            description: Some("UBA generated address collection".to_string()),
            xpub: None, // We don't expose the xpub for privacy
            derivation_paths: Some(self.get_derivation_paths()),
        });

        // Generate addresses for each supported type
        self.generate_legacy_addresses(&master_key, &mut addresses)?;
        self.generate_segwit_addresses(&master_key, &mut addresses)?;
        self.generate_taproot_addresses(&master_key, &mut addresses)?;

        // Generate L2 addresses
        self.generate_liquid_addresses(&master_key, &mut addresses)?;
        self.generate_lightning_addresses(&master_key, &mut addresses)?;

        // Generate Nostr public key
        self.generate_nostr_addresses(&master_key, &mut addresses)?;

        Ok(addresses)
    }

    /// Derive the master extended private key from seed input
    fn derive_master_key(&self, seed_input: &str) -> Result<Xpriv> {
        // Try to parse as BIP39 mnemonic first
        if let Ok(mnemonic) = Mnemonic::from_str(seed_input) {
            let seed = mnemonic.to_seed("");
            Xpriv::new_master(self.config.network, &seed)
                .map_err(|e| UbaError::AddressGeneration(e.to_string()))
        } else {
            // Try to parse as hex-encoded private key
            let key_bytes = hex::decode(seed_input.trim())?;
            if key_bytes.len() != 32 {
                return Err(UbaError::InvalidSeed(
                    "Private key must be 32 bytes".to_string(),
                ));
            }

            // Create a master key from the private key (simplified approach)
            Xpriv::new_master(self.config.network, &key_bytes)
                .map_err(|e| UbaError::AddressGeneration(e.to_string()))
        }
    }

    /// Generate legacy P2PKH addresses
    fn generate_legacy_addresses(
        &self,
        master_key: &Xpriv,
        addresses: &mut BitcoinAddresses,
    ) -> Result<()> {
        let derivation_path = DerivationPath::from_str("m/44'/0'/0'/0")?;
        let count = self.config.get_address_count(&AddressType::P2PKH);

        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;

            let private_key = PrivateKey::new(child_key.private_key, self.config.network);
            let public_key = PublicKey::from_private_key(&self.secp, &private_key);
            let address = Address::p2pkh(&public_key, self.config.network);

            addresses.add_address(AddressType::P2PKH, address.to_string());
        }

        Ok(())
    }

    /// Generate SegWit addresses (both P2SH-wrapped and native)
    fn generate_segwit_addresses(
        &self,
        master_key: &Xpriv,
        addresses: &mut BitcoinAddresses,
    ) -> Result<()> {
        // P2SH-wrapped SegWit (P2WPKH-in-P2SH)
        let p2sh_path = DerivationPath::from_str("m/49'/0'/0'/0")?;
        let p2sh_count = self.config.get_address_count(&AddressType::P2SH);

        for i in 0..p2sh_count {
            let child_path = p2sh_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;

            let private_key = PrivateKey::new(child_key.private_key, self.config.network);
            let public_key = PublicKey::from_private_key(&self.secp, &private_key);
            let address = Address::p2shwpkh(&public_key, self.config.network)?;

            addresses.add_address(AddressType::P2SH, address.to_string());
        }

        // Native SegWit (P2WPKH)
        let p2wpkh_path = DerivationPath::from_str("m/84'/0'/0'/0")?;
        let p2wpkh_count = self.config.get_address_count(&AddressType::P2WPKH);

        for i in 0..p2wpkh_count {
            let child_path = p2wpkh_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;

            let private_key = PrivateKey::new(child_key.private_key, self.config.network);
            let public_key = PublicKey::from_private_key(&self.secp, &private_key);
            let address = Address::p2wpkh(&public_key, self.config.network)?;

            addresses.add_address(AddressType::P2WPKH, address.to_string());
        }

        Ok(())
    }

    /// Generate Taproot addresses
    fn generate_taproot_addresses(
        &self,
        master_key: &Xpriv,
        addresses: &mut BitcoinAddresses,
    ) -> Result<()> {
        let derivation_path = DerivationPath::from_str("m/86'/0'/0'/0")?;
        let count = self.config.get_address_count(&AddressType::P2TR);

        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;

            let private_key = PrivateKey::new(child_key.private_key, self.config.network);
            let public_key = PublicKey::from_private_key(&self.secp, &private_key);
            let xonly_pubkey = XOnlyPublicKey::from(public_key);
            let address = Address::p2tr(&self.secp, xonly_pubkey, None, self.config.network);

            addresses.add_address(AddressType::P2TR, address.to_string());
        }

        Ok(())
    }

    /// Generate Liquid sidechain addresses
    fn generate_liquid_addresses(
        &self,
        master_key: &Xpriv,
        addresses: &mut BitcoinAddresses,
    ) -> Result<()> {
        // Use BIP84 path for Liquid SegWit addresses: m/84'/1776'/0'/0
        // 1776 is the coin type for Liquid Network
        let derivation_path = DerivationPath::from_str("m/84'/1776'/0'/0")?;
        let count = self.config.get_address_count(&AddressType::Liquid);

        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;

            // For Liquid addresses, we need to generate them differently to get the correct prefix
            // Convert the private key to elements format first
            let elements_private_key = elements::bitcoin::PrivateKey::new(
                child_key.private_key,
                match self.config.network {
                    bitcoin::Network::Bitcoin => elements::bitcoin::Network::Bitcoin,
                    bitcoin::Network::Testnet => elements::bitcoin::Network::Testnet,
                    bitcoin::Network::Signet => elements::bitcoin::Network::Signet,
                    bitcoin::Network::Regtest => elements::bitcoin::Network::Regtest,
                    _ => elements::bitcoin::Network::Testnet, // Default to testnet for unknown networks
                },
            );

            let elements_public_key = elements::bitcoin::PublicKey::from_private_key(
                &secp256k1::Secp256k1::new(),
                &elements_private_key,
            );

            // Generate Liquid address with proper parameters for mainnet/testnet
            let liquid_address = match self.config.network {
                bitcoin::Network::Bitcoin => {
                    // For Liquid mainnet, create confidential address with proper parameters
                    let address_params = &elements::AddressParams::LIQUID;

                    // For proper Liquid mainnet addresses, we should use confidential transactions
                    // Generate a blinding public key from the master key for this address
                    let blinding_private_key = {
                        let blinding_path =
                            derivation_path.child(ChildNumber::from_normal_idx((i + 1000) as u32)?);
                        let blinding_key = master_key.derive_priv(&self.secp, &blinding_path)?;
                        blinding_key.private_key
                    };
                    let blinding_public_key =
                        secp256k1::PublicKey::from_secret_key(&self.secp, &blinding_private_key);

                    // Create confidential address with blinding key (using secp256k1::PublicKey directly)
                    LiquidAddress::p2wpkh(
                        &elements_public_key,
                        Some(blinding_public_key),
                        address_params,
                    )
                }
                _ => {
                    // For testnet/regtest, use appropriate parameters
                    let address_params = match self.config.network {
                        bitcoin::Network::Testnet | bitcoin::Network::Signet => {
                            &elements::AddressParams::LIQUID_TESTNET
                        }
                        bitcoin::Network::Regtest => &elements::AddressParams::ELEMENTS,
                        _ => &elements::AddressParams::LIQUID_TESTNET,
                    };

                    // Create non-confidential address for testnet (simpler for testing)
                    LiquidAddress::p2wpkh(&elements_public_key, None, address_params)
                }
            };

            addresses.add_address(AddressType::Liquid, liquid_address.to_string());
        }

        Ok(())
    }

    /// Generate Lightning Network node addresses
    fn generate_lightning_addresses(
        &self,
        master_key: &Xpriv,
        addresses: &mut BitcoinAddresses,
    ) -> Result<()> {
        // Use a specific derivation path for Lightning node keys: m/1017'/0'/0'
        // 1017 is used for Lightning node identity keys
        let derivation_path = DerivationPath::from_str("m/1017'/0'/0'")?;
        let count = self.config.get_address_count(&AddressType::Lightning);

        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;

            // Convert to secp256k1 public key for Lightning
            let lightning_pubkey =
                Secp256k1PublicKey::from_secret_key(&self.secp, &child_key.private_key);

            // Format as Lightning node public key (33 bytes compressed, hex encoded)
            let lightning_node_id = hex::encode(lightning_pubkey.serialize());

            // Lightning addresses are typically the node public key
            // In the future, this could also include:
            // - BOLT12 offers
            // - Lightning addresses (email-like format)
            // - Channel information

            addresses.add_address(AddressType::Lightning, lightning_node_id);
        }

        Ok(())
    }

    /// Generate Nostr public key
    fn generate_nostr_addresses(
        &self,
        master_key: &Xpriv,
        addresses: &mut BitcoinAddresses,
    ) -> Result<()> {
        // Use a specific derivation path for Nostr keys: m/44'/1237'/0'/0
        // 1237 is a proposed coin type for Nostr (not officially assigned)
        let derivation_path = DerivationPath::from_str("m/44'/1237'/0'/0")?;
        let count = self.config.get_address_count(&AddressType::Nostr);

        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;

            // Convert the private key to a Nostr public key
            // Nostr uses secp256k1 keys, same as Bitcoin
            let nostr_secret_key = nostr::SecretKey::from_slice(
                &child_key.private_key.secret_bytes(),
            )
            .map_err(|e| {
                UbaError::AddressGeneration(format!("Failed to create Nostr secret key: {}", e))
            })?;

            let nostr_keys = nostr::Keys::new(nostr_secret_key);
            let nostr_public_key = nostr_keys.public_key();

            // Convert to npub format (Bech32-encoded public key)
            let npub_address = nostr_public_key.to_bech32().map_err(|e| {
                UbaError::AddressGeneration(format!("Failed to create npub address: {}", e))
            })?;

            addresses.add_address(AddressType::Nostr, npub_address);
        }

        Ok(())
    }

    /// Get the derivation paths used for address generation
    fn get_derivation_paths(&self) -> Vec<String> {
        vec![
            "m/44'/0'/0'/0".to_string(),    // Legacy
            "m/49'/0'/0'/0".to_string(),    // P2SH-wrapped SegWit
            "m/84'/0'/0'/0".to_string(),    // Native SegWit
            "m/86'/0'/0'/0".to_string(),    // Taproot
            "m/84'/1776'/0'/0".to_string(), // Liquid
            "m/1017'/0'/0'".to_string(),    // Lightning
            "m/44'/1237'/0'/0".to_string(), // Nostr
        ]
    }
}

impl From<bitcoin::bip32::Error> for UbaError {
    fn from(err: bitcoin::bip32::Error) -> Self {
        UbaError::AddressGeneration(err.to_string())
    }
}

impl From<elements::AddressError> for UbaError {
    fn from(err: elements::AddressError) -> Self {
        UbaError::AddressGeneration(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_generation_from_mnemonic() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config);

        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = generator.generate_addresses(mnemonic, Some("test".to_string()));

        assert!(result.is_ok());
        let addresses = result.unwrap();
        assert!(!addresses.is_empty());

        // Check that we have all address types
        assert!(addresses.get_addresses(&AddressType::P2PKH).is_some());
        assert!(addresses.get_addresses(&AddressType::P2WPKH).is_some());
        assert!(addresses.get_addresses(&AddressType::P2TR).is_some());
        assert!(addresses.get_addresses(&AddressType::Liquid).is_some());
        assert!(addresses.get_addresses(&AddressType::Lightning).is_some());
        assert!(addresses.get_addresses(&AddressType::Nostr).is_some());

        // Verify we have the expected number of addresses per type (default is now 1)
        assert_eq!(
            addresses.get_addresses(&AddressType::P2PKH).unwrap().len(),
            1
        );
        assert_eq!(
            addresses.get_addresses(&AddressType::Liquid).unwrap().len(),
            1
        );
        assert_eq!(
            addresses
                .get_addresses(&AddressType::Lightning)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            addresses.get_addresses(&AddressType::Nostr).unwrap().len(),
            1
        );
    }

    #[test]
    fn test_liquid_address_generation() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config);

        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = generator.generate_addresses(mnemonic, None);

        assert!(result.is_ok());
        let addresses = result.unwrap();

        let liquid_addresses = addresses.get_addresses(&AddressType::Liquid).unwrap();
        assert!(!liquid_addresses.is_empty());

        // Liquid addresses should start with appropriate prefixes
        for addr in liquid_addresses {
            // Liquid mainnet addresses typically start with 'lq1' or similar
            assert!(addr.len() > 10, "Liquid address should be reasonably long");
        }
    }

    #[test]
    fn test_lightning_address_generation() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config);

        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = generator.generate_addresses(mnemonic, None);

        assert!(result.is_ok());
        let addresses = result.unwrap();

        let lightning_addresses = addresses.get_addresses(&AddressType::Lightning).unwrap();
        assert!(!lightning_addresses.is_empty());

        // Lightning node IDs should be 66 character hex strings (33 bytes * 2)
        for addr in lightning_addresses {
            assert_eq!(
                addr.len(),
                66,
                "Lightning node ID should be 66 hex characters"
            );
            assert!(
                addr.chars().all(|c| c.is_ascii_hexdigit()),
                "Lightning node ID should be valid hex"
            );
        }
    }

    #[test]
    fn test_nostr_address_generation() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config);

        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = generator.generate_addresses(mnemonic, None);

        assert!(result.is_ok());
        let addresses = result.unwrap();

        let nostr_addresses = addresses.get_addresses(&AddressType::Nostr).unwrap();
        assert!(!nostr_addresses.is_empty());

        // Nostr public keys should be in npub format (Bech32-encoded)
        for addr in nostr_addresses {
            assert!(
                addr.starts_with("npub1"),
                "Nostr public key should start with 'npub1', got: {}",
                addr
            );
            assert!(
                addr.len() > 10,
                "Nostr npub address should be reasonably long"
            );
        }
    }

    #[test]
    fn test_invalid_seed() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config);

        let result = generator.generate_addresses("invalid seed phrase", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_address_generation() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config);

        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let result1 = generator.generate_addresses(mnemonic, Some("test".to_string()));
        let result2 = generator.generate_addresses(mnemonic, Some("test".to_string()));

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let addresses1 = result1.unwrap();
        let addresses2 = result2.unwrap();

        // Same seed should produce same addresses
        assert_eq!(
            addresses1.get_addresses(&AddressType::P2PKH),
            addresses2.get_addresses(&AddressType::P2PKH)
        );
        assert_eq!(
            addresses1.get_addresses(&AddressType::Liquid),
            addresses2.get_addresses(&AddressType::Liquid)
        );
        assert_eq!(
            addresses1.get_addresses(&AddressType::Lightning),
            addresses2.get_addresses(&AddressType::Lightning)
        );
        assert_eq!(
            addresses1.get_addresses(&AddressType::Nostr),
            addresses2.get_addresses(&AddressType::Nostr)
        );
    }

    #[test]
    fn test_nostr_address_included_in_collection() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config);

        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let result = generator.generate_addresses(mnemonic, Some("test-collection".to_string()));

        assert!(result.is_ok());
        let addresses = result.unwrap();

        // Verify that Nostr addresses are included in the collection
        assert!(addresses.get_addresses(&AddressType::Nostr).is_some());

        // Verify that the Nostr address is included in the flat list of all addresses
        let all_addresses = addresses.get_all_addresses();
        let nostr_addresses = addresses.get_addresses(&AddressType::Nostr).unwrap();

        // The Nostr public key should be in the flat list
        assert!(all_addresses.contains(&nostr_addresses[0]));

        // Verify the total count includes Nostr addresses
        assert_eq!(addresses.len(), 7); // P2PKH, P2SH, P2WPKH, P2TR, Liquid, Lightning, Nostr
    }
}
