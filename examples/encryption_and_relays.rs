//! Example demonstrating UBA encryption and custom relay configuration
//!
//! This example shows how to:
//! 1. Use encryption to secure address data on Nostr relays
//! 2. Configure custom relays vs using default public relays
//! 3. Generate and retrieve encrypted UBA data

use uba::{
    default_public_relays, derive_encryption_key, generate_random_key, generate_with_config,
    retrieve_with_config, UbaConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 UBA Encryption and Relay Configuration Example\n");

    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Example 1: Using default public relays without encryption
    println!("📡 Example 1: Default public relays (no encryption)");
    let mut config1 = UbaConfig::default();
    config1.set_all_counts(3); // Generate 3 addresses per type for faster demo

    let uba1 = generate_with_config(seed, Some("demo-wallet"), &[], config1.clone()).await?;
    println!("Generated UBA: {}", uba1);

    let addresses1 = retrieve_with_config(&uba1, &[], config1).await?;
    println!("Retrieved {} addresses\n", addresses1.len());

    // Example 2: Using encryption with a derived key
    println!("🔑 Example 2: Encryption with passphrase-derived key");
    let mut config2 = UbaConfig::default();
    config2.set_all_counts(3);

    // Derive encryption key from a passphrase
    let passphrase = "my-secret-passphrase-2024";
    let encryption_key = derive_encryption_key(passphrase, None);
    config2.set_encryption_key(encryption_key);

    println!("Encryption enabled: {}", config2.is_encryption_enabled());
    println!(
        "Encryption key (hex): {}",
        config2.get_encryption_key_hex().unwrap()
    );

    let uba2 = generate_with_config(seed, Some("encrypted-wallet"), &[], config2.clone()).await?;
    println!("Generated encrypted UBA: {}", uba2);

    let addresses2 = retrieve_with_config(&uba2, &[], config2).await?;
    println!(
        "Retrieved {} addresses from encrypted data\n",
        addresses2.len()
    );

    // Example 3: Using random encryption key with custom relays
    println!("🎲 Example 3: Random encryption key + custom relays");
    let mut config3 = UbaConfig::default();
    config3.set_all_counts(3);

    // Generate random encryption key
    let random_key = generate_random_key();
    config3.set_encryption_key(random_key);

    // Use a subset of reliable relays
    let custom_relays = vec![
        "wss://relay.damus.io".to_string(),
        "wss://nos.lol".to_string(),
        "wss://relay.snort.social".to_string(),
    ];
    config3.set_custom_relays(custom_relays.clone());

    println!("Using custom relays: {:?}", custom_relays);
    println!("Random encryption key: {}", hex::encode(random_key));

    let uba3 = generate_with_config(seed, Some("custom-setup"), &[], config3.clone()).await?;
    println!("Generated UBA with custom setup: {}", uba3);

    let addresses3 = retrieve_with_config(&uba3, &[], config3).await?;
    println!("Retrieved {} addresses\n", addresses3.len());

    // Example 4: Demonstrating relay configuration options
    println!("⚙️  Example 4: Relay configuration options");
    let mut config4 = UbaConfig::default();

    // Show default relays
    println!(
        "Default public relays ({} total):",
        default_public_relays().len()
    );
    for (i, relay) in default_public_relays().iter().enumerate() {
        println!("  {}. {}", i + 1, relay);
    }

    // Add custom relay to defaults
    config4.add_custom_relay("wss://my-personal-relay.com".to_string());
    println!("\nAfter adding custom relay:");
    for (i, relay) in config4.get_relay_urls().iter().enumerate() {
        println!("  {}. {}", i + 1, relay);
    }

    // Reset to defaults
    config4.use_default_relays();
    println!(
        "\nAfter resetting to defaults: {} relays",
        config4.get_relay_urls().len()
    );

    // Example 5: Error handling for wrong encryption key
    println!("\n🚨 Example 5: Error handling with wrong encryption key");
    let mut wrong_config = UbaConfig::default();
    let wrong_key = generate_random_key(); // Different key
    wrong_config.set_encryption_key(wrong_key);

    // Try to retrieve with wrong key (should still work due to fallback)
    match retrieve_with_config(&uba2, &[], wrong_config).await {
        Ok(addresses) => println!(
            "Retrieved {} addresses (fallback to unencrypted)",
            addresses.len()
        ),
        Err(e) => println!("Error with wrong key: {}", e),
    }

    println!("\n✅ All examples completed successfully!");
    Ok(())
}
