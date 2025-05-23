//! Bitcoin address generation from seeds

use crate::error::{Result, UbaError};
use crate::types::{AddressType, BitcoinAddresses, UbaConfig, AddressMetadata};

use bitcoin::{
    PrivateKey, PublicKey, Address, XOnlyPublicKey,
    bip32::{Xpriv, DerivationPath, ChildNumber},
};
use bip39::Mnemonic;
use std::str::FromStr;

// Conditional imports for different crypto backends
#[cfg(not(target_arch = "wasm32"))]
use bitcoin::secp256k1::Secp256k1;

#[cfg(target_arch = "wasm32")]
use k256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::sec1::ToEncodedPoint,
    SecretKey as K256SecretKey,
    PublicKey as K256PublicKey,
};

// Liquid support (native only)
#[cfg(feature = "native")]
use elements::Address as LiquidAddress;

// Lightning support (native only)
#[cfg(feature = "native")]
use secp256k1::PublicKey as Secp256k1PublicKey;

/// Secp256k1 context abstraction for different backends
#[cfg(not(target_arch = "wasm32"))]
type SecpContext = Secp256k1<bitcoin::secp256k1::All>;

#[cfg(target_arch = "wasm32")]
type SecpContext = ();

/// Address generator for creating Bitcoin addresses from seeds
pub struct AddressGenerator {
    config: UbaConfig,
    #[cfg(not(target_arch = "wasm32"))]
    secp: SecpContext,
    #[cfg(target_arch = "wasm32")]
    secp: SecpContext,
}

impl AddressGenerator {
    /// Create a new address generator with the given configuration
    pub fn new(config: UbaConfig) -> Self {
        Self {
            config,
            #[cfg(not(target_arch = "wasm32"))]
            secp: Secp256k1::new(),
            #[cfg(target_arch = "wasm32")]
            secp: (),
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
    pub fn generate_addresses(&self, seed_input: &str, label: Option<String>) -> Result<BitcoinAddresses> {
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
        
        // Generate L2 addresses (native only)
        #[cfg(feature = "native")]
        {
        self.generate_liquid_addresses(&master_key, &mut addresses)?;
        self.generate_lightning_addresses(&master_key, &mut addresses)?;
        }

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
                return Err(UbaError::InvalidSeed("Private key must be 32 bytes".to_string()));
            }
            
            // Create a master key from the private key (simplified approach)
            Xpriv::new_master(self.config.network, &key_bytes)
                .map_err(|e| UbaError::AddressGeneration(e.to_string()))
        }
    }

    /// Generate legacy P2PKH addresses
    fn generate_legacy_addresses(&self, master_key: &Xpriv, addresses: &mut BitcoinAddresses) -> Result<()> {
        let derivation_path = DerivationPath::from_str("m/44'/0'/0'/0")?;
        let count = self.config.get_address_count(&AddressType::P2PKH);
        
        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = self.derive_child_key(master_key, &child_path)?;
            
            let address = self.create_p2pkh_address(&child_key)?;
            addresses.add_address(AddressType::P2PKH, address);
        }
        
        Ok(())
    }

    /// Generate SegWit addresses (both P2SH-wrapped and native)
    fn generate_segwit_addresses(&self, master_key: &Xpriv, addresses: &mut BitcoinAddresses) -> Result<()> {
        // P2SH-wrapped SegWit (P2WPKH-in-P2SH)
        let p2sh_path = DerivationPath::from_str("m/49'/0'/0'/0")?;
        let p2sh_count = self.config.get_address_count(&AddressType::P2SH);
        
        for i in 0..p2sh_count {
            let child_path = p2sh_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = self.derive_child_key(master_key, &child_path)?;
            
            let address = self.create_p2shwpkh_address(&child_key)?;
            addresses.add_address(AddressType::P2SH, address);
        }

        // Native SegWit (P2WPKH)
        let p2wpkh_path = DerivationPath::from_str("m/84'/0'/0'/0")?;
        let p2wpkh_count = self.config.get_address_count(&AddressType::P2WPKH);
        
        for i in 0..p2wpkh_count {
            let child_path = p2wpkh_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = self.derive_child_key(master_key, &child_path)?;
            
            let address = self.create_p2wpkh_address(&child_key)?;
            addresses.add_address(AddressType::P2WPKH, address);
        }
        
        Ok(())
    }

    /// Generate Taproot addresses
    fn generate_taproot_addresses(&self, master_key: &Xpriv, addresses: &mut BitcoinAddresses) -> Result<()> {
        let derivation_path = DerivationPath::from_str("m/86'/0'/0'/0")?;
        let count = self.config.get_address_count(&AddressType::P2TR);
        
        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = self.derive_child_key(master_key, &child_path)?;
            
            let address = self.create_p2tr_address(&child_key)?;
            addresses.add_address(AddressType::P2TR, address);
        }
        
        Ok(())
    }

    // Helper methods for different crypto backends

    #[cfg(not(target_arch = "wasm32"))]
    fn derive_child_key(&self, master_key: &Xpriv, path: &DerivationPath) -> Result<Xpriv> {
        master_key.derive_priv(&self.secp, path)
            .map_err(|e| UbaError::AddressGeneration(e.to_string()))
    }

    #[cfg(target_arch = "wasm32")]
    fn derive_child_key(&self, master_key: &Xpriv, path: &DerivationPath) -> Result<Xpriv> {
        // For WASM, we'll use the bitcoin crate's derivation which doesn't require secp context
        // This is a simplified approach - in a full implementation you'd want to use k256 directly
        master_key.derive_priv(&bitcoin::secp256k1::Secp256k1::new(), path)
            .map_err(|e| UbaError::AddressGeneration(e.to_string()))
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_p2pkh_address(&self, child_key: &Xpriv) -> Result<String> {
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&self.secp, &private_key);
        let address = Address::p2pkh(&public_key, self.config.network);
        Ok(address.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn create_p2pkh_address(&self, child_key: &Xpriv) -> Result<String> {
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&bitcoin::secp256k1::Secp256k1::new(), &private_key);
        let address = Address::p2pkh(&public_key, self.config.network);
        Ok(address.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_p2shwpkh_address(&self, child_key: &Xpriv) -> Result<String> {
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&self.secp, &private_key);
        let address = Address::p2shwpkh(&public_key, self.config.network)?;
        Ok(address.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn create_p2shwpkh_address(&self, child_key: &Xpriv) -> Result<String> {
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&bitcoin::secp256k1::Secp256k1::new(), &private_key);
        let address = Address::p2shwpkh(&public_key, self.config.network)?;
        Ok(address.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_p2wpkh_address(&self, child_key: &Xpriv) -> Result<String> {
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&self.secp, &private_key);
        let address = Address::p2wpkh(&public_key, self.config.network)?;
        Ok(address.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn create_p2wpkh_address(&self, child_key: &Xpriv) -> Result<String> {
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&bitcoin::secp256k1::Secp256k1::new(), &private_key);
        let address = Address::p2wpkh(&public_key, self.config.network)?;
        Ok(address.to_string())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn create_p2tr_address(&self, child_key: &Xpriv) -> Result<String> {
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&self.secp, &private_key);
        let xonly_pubkey = XOnlyPublicKey::from(public_key);
        let address = Address::p2tr(&self.secp, xonly_pubkey, None, self.config.network);
        Ok(address.to_string())
    }

    #[cfg(target_arch = "wasm32")]
    fn create_p2tr_address(&self, child_key: &Xpriv) -> Result<String> {
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let private_key = PrivateKey::new(child_key.private_key, self.config.network);
        let public_key = PublicKey::from_private_key(&secp, &private_key);
        let xonly_pubkey = XOnlyPublicKey::from(public_key);
        let address = Address::p2tr(&secp, xonly_pubkey, None, self.config.network);
        Ok(address.to_string())
    }

    /// Generate Liquid sidechain addresses (native only)
    #[cfg(feature = "native")]
    fn generate_liquid_addresses(&self, master_key: &Xpriv, addresses: &mut BitcoinAddresses) -> Result<()> {
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
                    _ => elements::bitcoin::Network::Bitcoin, // Default to Bitcoin for unknown networks
                }
            );
            
            let elements_public_key = elements::bitcoin::PublicKey::from_private_key(
                &secp256k1::Secp256k1::new(),
                &elements_private_key
            );
            
            // Generate Liquid address with proper parameters for mainnet/testnet
            let liquid_address = match self.config.network {
                bitcoin::Network::Bitcoin => {
                    // For Liquid mainnet, create confidential address with proper parameters
                    let address_params = &elements::AddressParams::LIQUID;
                    
                    // For proper Liquid mainnet addresses, we should use confidential transactions
                    // Generate a blinding public key from the master key for this address
                    let blinding_private_key = {
                        let blinding_path = derivation_path.child(ChildNumber::from_normal_idx((i + 1000) as u32)?);
                        let blinding_key = master_key.derive_priv(&self.secp, &blinding_path)?;
                        blinding_key.private_key
                    };
                    let blinding_public_key = secp256k1::PublicKey::from_secret_key(&self.secp, &blinding_private_key);
                    
                    // Create confidential address with blinding key (using secp256k1::PublicKey directly)
                    LiquidAddress::p2wpkh(&elements_public_key, Some(blinding_public_key), address_params)
                }
                _ => {
                    // For testnet/regtest, use appropriate parameters
                    let address_params = &elements::AddressParams::LIQUID_TESTNET;
                    LiquidAddress::p2wpkh(&elements_public_key, None, address_params)
                }
            };
            
            addresses.add_address(AddressType::Liquid, liquid_address.to_string());
        }
        
        Ok(())
    }

    /// Generate Lightning Network addresses (native only)
    #[cfg(feature = "native")]
    fn generate_lightning_addresses(&self, master_key: &Xpriv, addresses: &mut BitcoinAddresses) -> Result<()> {
        // Use BIP84 path for Lightning addresses: m/84'/1'/0'/0
        // This generates the public keys that can be used for Lightning node IDs
        let derivation_path = DerivationPath::from_str("m/84'/1'/0'/0")?;
        let count = self.config.get_address_count(&AddressType::Lightning);
        
        for i in 0..count {
            let child_path = derivation_path.child(ChildNumber::from_normal_idx(i as u32)?);
            let child_key = master_key.derive_priv(&self.secp, &child_path)?;
            
            // For Lightning, we generate a node public key that can be used as a node identifier
            let lightning_pubkey = Secp256k1PublicKey::from_secret_key(&self.secp, &child_key.private_key);
            
            // Format as a Lightning node address (simplified - in practice this would include network info)
            let lightning_address = format!("{}@lightning.node", hex::encode(lightning_pubkey.serialize()));
            
            addresses.add_address(AddressType::Lightning, lightning_address);
        }
        
        Ok(())
    }

    /// Get the derivation paths used for address generation
    fn get_derivation_paths(&self) -> Vec<String> {
        vec![
            "m/44'/0'/0'/0".to_string(),  // P2PKH
            "m/49'/0'/0'/0".to_string(),  // P2SH-wrapped SegWit
            "m/84'/0'/0'/0".to_string(),  // Native SegWit
            "m/86'/0'/0'/0".to_string(),  // Taproot
            "m/84'/1776'/0'/0".to_string(), // Liquid
            "m/84'/1'/0'/0".to_string(),   // Lightning
        ]
    }
}

// Error conversions
impl From<bitcoin::bip32::Error> for UbaError {
    fn from(err: bitcoin::bip32::Error) -> Self {
        UbaError::AddressGeneration(err.to_string())
    }
}

#[cfg(feature = "native")]
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
        let generator = AddressGenerator::new(config.clone());
        
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let addresses = generator.generate_addresses(seed, Some("test".to_string())).unwrap();
        
        assert!(!addresses.is_empty());
        assert!(addresses.get_addresses(&AddressType::P2PKH).is_some());
        assert!(addresses.get_addresses(&AddressType::P2WPKH).is_some());
        assert!(addresses.get_addresses(&AddressType::P2TR).is_some());
        
        // Check that we have the expected number of addresses
        let p2pkh_addresses = addresses.get_addresses(&AddressType::P2PKH).unwrap();
        assert_eq!(p2pkh_addresses.len(), config.get_address_count(&AddressType::P2PKH));
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_liquid_address_generation() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config.clone());
        
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let addresses = generator.generate_addresses(seed, Some("test".to_string())).unwrap();
        
        // Check that Liquid addresses are generated
        assert!(addresses.get_addresses(&AddressType::Liquid).is_some());
        let liquid_addresses = addresses.get_addresses(&AddressType::Liquid).unwrap();
        assert!(!liquid_addresses.is_empty());
        
        // Liquid addresses should start with appropriate prefix
        for addr in liquid_addresses {
            assert!(addr.starts_with("lq1") || addr.starts_with("ex1") || addr.starts_with("ert1"));
        }
    }

    #[cfg(feature = "native")]
    #[test]
    fn test_lightning_address_generation() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config.clone());
        
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let addresses = generator.generate_addresses(seed, Some("test".to_string())).unwrap();
        
        // Check that Lightning addresses are generated
        assert!(addresses.get_addresses(&AddressType::Lightning).is_some());
        let lightning_addresses = addresses.get_addresses(&AddressType::Lightning).unwrap();
        assert!(!lightning_addresses.is_empty());
        
        // Lightning addresses should contain node identifier format
        for addr in lightning_addresses {
            assert!(addr.contains("@lightning.node"));
        }
    }

    #[test]
    fn test_invalid_seed() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config.clone());
        
        let result = generator.generate_addresses("invalid seed", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_deterministic_address_generation() {
        let config = UbaConfig::default();
        let generator = AddressGenerator::new(config.clone());
        
        let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        
        let addresses1 = generator.generate_addresses(seed, Some("test".to_string())).unwrap();
        let addresses2 = generator.generate_addresses(seed, Some("test".to_string())).unwrap();
        
        // Same seed should produce same addresses
        assert_eq!(
            addresses1.get_addresses(&AddressType::P2PKH),
            addresses2.get_addresses(&AddressType::P2PKH)
        );
        assert_eq!(
            addresses1.get_addresses(&AddressType::P2WPKH),
            addresses2.get_addresses(&AddressType::P2WPKH)
        );
    }
} 