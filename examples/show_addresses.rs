//! Example showing generated addresses across all Bitcoin layers
//!
//! This example demonstrates the offline generation of addresses
//! for Bitcoin L1, Liquid sidechain, and Lightning Network
//! with configurable address counts per type

use uba::{AddressGenerator, AddressType, UbaConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 UBA Configurable Multi-Layer Address Generation Demo\n");

    // Example seed phrase (DO NOT use this in production!)
    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    println!("📝 Seed phrase: {}", seed);

    // Demonstrate different configuration scenarios
    demo_default_config(seed)?;
    demo_custom_counts(seed)?;
    demo_layer_specific_config(seed)?;

    Ok(())
}

/// Demo with default configuration (1 address per type)
fn demo_default_config(seed: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 SCENARIO 1: Default Configuration (1 address per type)");
    println!("{}", "=".repeat(60));

    let config = UbaConfig::default();
    let generator = AddressGenerator::new(config);

    let addresses = generator.generate_addresses(seed, Some("default-config".to_string()))?;

    println!("✅ Generated {} total addresses", addresses.len());
    display_address_summary(&addresses);

    Ok(())
}

/// Demo with custom counts for different address types
fn demo_custom_counts(seed: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 SCENARIO 2: Custom Address Counts");
    println!("{}", "=".repeat(60));

    let mut config = UbaConfig::default();

    // Set different counts for different address types
    config.set_address_count(AddressType::P2PKH, 10); // 10 legacy addresses
    config.set_address_count(AddressType::P2WPKH, 15); // 15 SegWit addresses
    config.set_address_count(AddressType::P2TR, 3); // 3 Taproot addresses
    config.set_address_count(AddressType::Liquid, 7); // 7 Liquid addresses
    config.set_address_count(AddressType::Lightning, 2); // 2 Lightning node IDs
    config.set_address_count(AddressType::Nostr, 1); // 1 Nostr public key
                                                     // P2SH will use default (1)

    println!("Configuration:");
    println!("   P2PKH (Legacy): 10 addresses");
    println!("   P2SH (SegWit-wrapped): 1 address (default)");
    println!("   P2WPKH (Native SegWit): 15 addresses");
    println!("   P2TR (Taproot): 3 addresses");
    println!("   Liquid: 7 addresses");
    println!("   Lightning: 2 addresses");
    println!("   Nostr: 1 address");

    let generator = AddressGenerator::new(config);
    let addresses = generator.generate_addresses(seed, Some("custom-counts".to_string()))?;

    println!("\n✅ Generated {} total addresses", addresses.len());
    display_address_summary(&addresses);

    Ok(())
}

/// Demo with layer-specific configuration
fn demo_layer_specific_config(seed: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 SCENARIO 3: Layer-Specific Configuration");
    println!("{}", "=".repeat(60));

    let mut config = UbaConfig::default();

    // Focus on Bitcoin L1 with many addresses, minimal L2
    config.set_bitcoin_l1_counts(8); // 8 addresses for all Bitcoin L1 types
    config.set_address_count(AddressType::Liquid, 1); // Just 1 Liquid address
    config.set_address_count(AddressType::Lightning, 1); // Just 1 Lightning node ID
    config.set_address_count(AddressType::Nostr, 1); // Just 1 Nostr public key

    println!("Configuration (Bitcoin L1 focused):");
    println!("   All Bitcoin L1 types: 8 addresses each");
    println!("   Liquid: 1 address");
    println!("   Lightning: 1 address");
    println!("   Nostr: 1 address");

    let generator = AddressGenerator::new(config);
    let addresses = generator.generate_addresses(seed, Some("l1-focused".to_string()))?;

    println!("\n✅ Generated {} total addresses", addresses.len());
    display_address_summary(&addresses);

    // Show a few sample addresses from each type
    println!("\n📋 Sample Addresses:");
    show_sample_addresses(&addresses);

    Ok(())
}

/// Display summary of address counts by type
fn display_address_summary(addresses: &uba::BitcoinAddresses) {
    println!("\n📊 Address Count Summary:");

    let types = [
        (AddressType::P2PKH, "Bitcoin Legacy (P2PKH)"),
        (AddressType::P2SH, "Bitcoin SegWit-wrapped (P2SH)"),
        (AddressType::P2WPKH, "Bitcoin Native SegWit (P2WPKH)"),
        (AddressType::P2TR, "Bitcoin Taproot (P2TR)"),
        (AddressType::Liquid, "Liquid Sidechain"),
        (AddressType::Lightning, "Lightning Network"),
        (AddressType::Nostr, "Nostr Public Keys (npub)"),
    ];

    for (addr_type, type_name) in types {
        let count = addresses
            .get_addresses(&addr_type)
            .map(|addrs| addrs.len())
            .unwrap_or(0);
        println!("   {}: {} addresses", type_name, count);
    }
}

/// Show sample addresses from each type
fn show_sample_addresses(addresses: &uba::BitcoinAddresses) {
    let types = [
        (AddressType::P2PKH, "Legacy", "1..."),
        (AddressType::P2WPKH, "SegWit", "bc1..."),
        (AddressType::P2TR, "Taproot", "bc1p..."),
        (AddressType::Liquid, "Liquid", "lq1..."),
        (AddressType::Lightning, "Lightning", "hex..."),
        (AddressType::Nostr, "Nostr", "npub1..."),
    ];

    for (addr_type, type_name, prefix) in types {
        if let Some(addrs) = addresses.get_addresses(&addr_type) {
            if !addrs.is_empty() {
                println!("   {} ({}): {}", type_name, prefix, addrs[0]);
            }
        }
    }
}
