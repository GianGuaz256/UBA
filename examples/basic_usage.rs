//! Basic usage example for the UBA library
//!
//! This example demonstrates:
//! - Generating a UBA from a seed phrase
//! - Parsing the UBA string
//! - Retrieving addresses from the UBA (requires working relays)
//! - Showcasing Bitcoin L1, Liquid, and Lightning addresses

use uba::{generate, parse_uba, retrieve_full, AddressType, Network, UbaConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 UBA Library - Multi-Layer Bitcoin Address Example\n");

    // Example seed phrase (DO NOT use this in production!)
    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    // Example relay URLs (these are just examples - you would use real Nostr relays)
    let relay_urls = vec![
        "wss://relay.lifpay.me".to_string(),
        "wss://a.nos.lol".to_string(),
        "wss://ditto.pub/relay".to_string(),
    ];

    println!("📝 Seed phrase: {}", seed);
    println!("🔗 Relay URLs: {:?}\n", relay_urls);

    // Step 1: Generate a UBA with multi-layer support
    println!("🔄 Generating UBA with L1, Liquid, and Lightning addresses...");
    match generate(seed, Some("multi-layer-wallet"), &relay_urls).await {
        Ok(uba) => {
            println!("✅ Generated UBA: {}\n", uba);

            // Step 2: Parse the UBA string
            println!("🔍 Parsing UBA string...");
            match parse_uba(&uba) {
                Ok(parsed) => {
                    println!("✅ Parsed UBA:");
                    println!("   Nostr ID: {}", parsed.nostr_id);
                    println!("   Label: {:?}\n", parsed.label);

                    // Step 3: Retrieve full address structure to see all layers
                    println!("📥 Attempting to retrieve full address structure...");
                    match retrieve_full(&uba, &relay_urls).await {
                        Ok(bitcoin_addresses) => {
                            println!("✅ Retrieved addresses across all layers:\n");

                            // Display Bitcoin L1 addresses
                            println!("🟠 Bitcoin Layer 1 Addresses:");
                            display_addresses(
                                &bitcoin_addresses,
                                &AddressType::P2PKH,
                                "Legacy (P2PKH)",
                            );
                            display_addresses(
                                &bitcoin_addresses,
                                &AddressType::P2SH,
                                "SegWit-wrapped (P2SH)",
                            );
                            display_addresses(
                                &bitcoin_addresses,
                                &AddressType::P2WPKH,
                                "Native SegWit (P2WPKH)",
                            );
                            display_addresses(
                                &bitcoin_addresses,
                                &AddressType::P2TR,
                                "Taproot (P2TR)",
                            );

                            // Display Liquid addresses
                            println!("\n💧 Liquid Sidechain Addresses:");
                            display_addresses(
                                &bitcoin_addresses,
                                &AddressType::Liquid,
                                "Liquid SegWit",
                            );

                            // Display Lightning addresses
                            println!("\n⚡ Lightning Network Addresses:");
                            display_addresses(
                                &bitcoin_addresses,
                                &AddressType::Lightning,
                                "Lightning Node IDs",
                            );

                            // Display Nostr addresses
                            println!("\n🔑 Nostr Addresses:");
                            display_addresses(
                                &bitcoin_addresses,
                                &AddressType::Nostr,
                                "Nostr Public Keys (npub)",
                            );

                            // Show metadata
                            if let Some(metadata) = &bitcoin_addresses.metadata {
                                println!("\n📊 Address Collection Metadata:");
                                if let Some(label) = &metadata.label {
                                    println!("   Label: {}", label);
                                }
                                if let Some(description) = &metadata.description {
                                    println!("   Description: {}", description);
                                }
                                if let Some(paths) = &metadata.derivation_paths {
                                    println!("   Derivation Paths: {}", paths.join(", "));
                                }
                                println!("   Created: {}", bitcoin_addresses.created_at);
                                println!("   Version: {}", bitcoin_addresses.version);
                            }
                        }
                        Err(e) => {
                            println!("⚠️  Could not retrieve addresses: {}", e);
                            println!("   This is expected if the relays are not accessible or the note hasn't propagated yet.");
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to parse UBA: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to generate UBA: {}", e);
            println!("   This might be due to relay connectivity issues.");
        }
    }

    println!("\n🔧 Advanced Configuration Example:");

    // Demonstrate custom configuration with different networks
    let mut config = UbaConfig::default();
    config.network = Network::Testnet;
    config.max_addresses_per_type = 3;
    config.relay_timeout = 5;

    println!("   Network: {:?}", config.network);
    println!(
        "   Max addresses per type: {}",
        config.max_addresses_per_type
    );
    println!("   Relay timeout: {} seconds", config.relay_timeout);

    println!("\n🎯 Address Types Summary:");
    println!("   📍 Bitcoin L1: Legacy, SegWit-wrapped, Native SegWit, Taproot");
    println!("   💧 Liquid: Sidechain addresses for faster, private transactions");
    println!("   ⚡ Lightning: Node IDs for Lightning Network payments");
    println!("   �� Nostr: Public keys in npub format for decentralized social networking");

    println!("\n✨ Multi-layer UBA example completed!");
    Ok(())
}

/// Helper function to display addresses of a specific type
fn display_addresses(
    bitcoin_addresses: &uba::BitcoinAddresses,
    address_type: &AddressType,
    type_name: &str,
) {
    if let Some(addresses) = bitcoin_addresses.get_addresses(address_type) {
        println!("   {} ({} addresses):", type_name, addresses.len());
        for (i, addr) in addresses.iter().enumerate().take(2) {
            // Show first 2 for brevity
            println!("     {}: {}", i + 1, addr);
        }
        if addresses.len() > 2 {
            println!("     ... and {} more", addresses.len() - 2);
        }
    }
}

/// Example of offline address generation (doesn't require network access)
#[allow(dead_code)]
fn offline_address_generation_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Offline Address Generation Example\n");

    // Generate addresses locally without publishing to relays
    use uba::address::AddressGenerator;

    let config = UbaConfig::default();
    let generator = AddressGenerator::new(config);

    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    match generator.generate_addresses(seed, Some("offline-test".to_string())) {
        Ok(addresses) => {
            println!("✅ Generated addresses offline:");
            println!("   Total addresses: {}", addresses.len());

            for address_type in [
                AddressType::P2PKH,
                AddressType::P2WPKH,
                AddressType::P2TR,
                AddressType::Liquid,
                AddressType::Lightning,
                AddressType::Nostr,
            ] {
                if let Some(addrs) = addresses.get_addresses(&address_type) {
                    println!("   {:?}: {} addresses", address_type, addrs.len());
                }
            }
        }
        Err(e) => println!("❌ Error: {}", e),
    }

    Ok(())
}
