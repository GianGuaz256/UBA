//! Encryption demonstration without network connectivity
//! 
//! This example shows the encryption functionality without requiring
//! actual relay connections.

use uba::{
    UbaConfig, AddressGenerator, derive_encryption_key, generate_random_key, 
    default_public_relays, UbaEncryption
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 UBA Encryption Features Demo\n");

    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Example 1: Basic address generation (no encryption)
    println!("📊 Example 1: Basic address generation");
    let config1 = UbaConfig::default();
    let generator1 = AddressGenerator::new(config1);
    let addresses1 = generator1.generate_addresses(seed, Some("demo-wallet".to_string()))?;
    
    println!("Generated {} addresses across all layers", addresses1.get_all_addresses().len());
    
    // Count addresses by type
    let btc_l1_count = addresses1.get_addresses(&uba::AddressType::P2PKH).map(|v| v.len()).unwrap_or(0) +
                      addresses1.get_addresses(&uba::AddressType::P2SH).map(|v| v.len()).unwrap_or(0) +
                      addresses1.get_addresses(&uba::AddressType::P2WPKH).map(|v| v.len()).unwrap_or(0) +
                      addresses1.get_addresses(&uba::AddressType::P2TR).map(|v| v.len()).unwrap_or(0);
    let liquid_count = addresses1.get_addresses(&uba::AddressType::Liquid).map(|v| v.len()).unwrap_or(0);
    let lightning_count = addresses1.get_addresses(&uba::AddressType::Lightning).map(|v| v.len()).unwrap_or(0);
    
    println!("Bitcoin L1: {}", btc_l1_count);
    println!("Liquid: {}", liquid_count);
    println!("Lightning: {}\n", lightning_count);

    // Example 2: Encryption key derivation
    println!("🔑 Example 2: Encryption key derivation");
    let passphrase = "my-secret-passphrase-2024";
    let derived_key = derive_encryption_key(passphrase, None);
    println!("Passphrase: {}", passphrase);
    println!("Derived key (hex): {}", hex::encode(derived_key));
    
    // Test encryption/decryption
    let encryption = UbaEncryption::new(derived_key);
    let test_data = "Hello, encrypted UBA world!";
    let encrypted = encryption.encrypt(test_data)?;
    let decrypted = encryption.decrypt(&encrypted)?;
    println!("Original: {}", test_data);
    println!("Encrypted: {}", encrypted);
    println!("Decrypted: {}\n", decrypted);

    // Example 3: Random encryption key
    println!("🎲 Example 3: Random encryption key");
    let random_key = generate_random_key();
    println!("Random key (hex): {}", hex::encode(random_key));
    
    let encryption2 = UbaEncryption::new(random_key);
    let json_data = r#"{"test": "data", "numbers": [1, 2, 3]}"#;
    let encrypted_json = encryption2.encrypt(json_data)?;
    let decrypted_json = encryption2.decrypt(&encrypted_json)?;
    println!("JSON data: {}", json_data);
    println!("Encrypted JSON: {}", encrypted_json);
    println!("Decrypted JSON: {}\n", decrypted_json);

    // Example 4: Configuration with encryption
    println!("⚙️ Example 4: UBA configuration with encryption");
    let mut config = UbaConfig::default();
    config.set_encryption_key(derived_key);
    config.set_all_counts(3); // 3 addresses per type for demo
    
    println!("Encryption enabled: {}", config.is_encryption_enabled());
    println!("Encryption key hex: {}", config.get_encryption_key_hex().unwrap());
    
    let generator = AddressGenerator::new(config.clone());
    let addresses = generator.generate_addresses(seed, Some("encrypted-demo".to_string()))?;
    println!("Generated {} addresses with encryption config\n", addresses.get_all_addresses().len());

    // Example 5: Relay configuration
    println!("📡 Example 5: Relay configuration");
    let default_relays = default_public_relays();
    println!("Default public relays ({} total):", default_relays.len());
    for (i, relay) in default_relays.iter().take(5).enumerate() {
        println!("   {}. {}", i + 1, relay);
    }
    println!("   ... and {} more", default_relays.len() - 5);
    
    let mut config_with_relays = UbaConfig::default();
    config_with_relays.set_custom_relays(vec![
        "wss://my-relay.com".to_string(),
        "wss://relay.damus.io".to_string(),
    ]);
    println!("\nCustom relays: {:?}", config_with_relays.get_relay_urls());
    
    config_with_relays.add_custom_relay("wss://additional-relay.com".to_string());
    println!("After adding relay: {:?}", config_with_relays.get_relay_urls());

    // Example 6: Address count configuration
    println!("\n🔢 Example 6: Address count configuration");
    let mut config_counts = UbaConfig::default();
    
    // Set different counts for different types
    config_counts.set_address_count(uba::AddressType::P2WPKH, 10);
    config_counts.set_address_count(uba::AddressType::Liquid, 5);
    config_counts.set_address_count(uba::AddressType::Lightning, 3);
    
    println!("P2WPKH count: {}", config_counts.get_address_count(&uba::AddressType::P2WPKH));
    println!("Liquid count: {}", config_counts.get_address_count(&uba::AddressType::Liquid));
    println!("Lightning count: {}", config_counts.get_address_count(&uba::AddressType::Lightning));
    
    let generator_counts = AddressGenerator::new(config_counts);
    let addresses_counts = generator_counts.generate_addresses(seed, Some("custom-counts".to_string()))?;
    println!("Total addresses with custom counts: {}", addresses_counts.get_all_addresses().len());

    println!("\n✅ All encryption and configuration features demonstrated successfully!");
    Ok(())
} 