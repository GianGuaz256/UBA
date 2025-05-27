//! UBA Update and Address Filtering Demo
//!
//! This example demonstrates:
//! 1. Creating a UBA with specific address types enabled/disabled
//! 2. Updating an existing UBA with new address configuration
//! 3. Validating update operations
//! 4. Working with address filtering

use uba::{
    generate_with_config, update_uba, update_uba_with_addresses, retrieve_full_with_config,
    AddressGenerator, AddressType, BitcoinAddresses, UbaConfig, UbaError,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 UBA Update and Address Filtering Demo");
    println!("=========================================\n");

    // Demo seed phrase
    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Use a test relay (in production, use reliable relays)
    let relays = vec![
        "wss://relay.damus.io".to_string(),
        "wss://nos.lol".to_string(),
    ];

    // Step 1: Create initial UBA with all address types
    println!("📝 Step 1: Creating initial UBA with all address types");
    let mut initial_config = UbaConfig::default();
    initial_config.set_all_counts(2); // Generate 2 addresses per type
    
    println!("   Enabled address types: {:?}", initial_config.get_enabled_address_types());
    
    match generate_with_config(seed, Some("demo-wallet"), &relays, initial_config.clone()).await {
        Ok(initial_uba) => {
            println!("   ✅ Initial UBA created: {}", initial_uba);
            
            // Extract the Nostr event ID from the UBA
            let event_id = initial_uba.strip_prefix("UBA:").unwrap().split('&').next().unwrap();
            println!("   📋 Event ID: {}", event_id);

            // Step 2: Retrieve and display initial addresses
            println!("\n📖 Step 2: Retrieving initial addresses");
            match retrieve_full_with_config(&initial_uba, &relays, initial_config.clone()).await {
                Ok(addresses) => {
                    display_addresses(&addresses);
                }
                Err(e) => println!("   ❌ Failed to retrieve initial addresses: {}", e),
            }

            // Step 3: Create updated configuration with filtering
            println!("\n🔧 Step 3: Creating update configuration with address filtering");
            let mut update_config = UbaConfig::default();
            
            // Disable Lightning and Liquid for the update
            update_config.set_address_type_enabled(AddressType::Lightning, false);
            update_config.set_address_type_enabled(AddressType::Liquid, false);
            
            // Increase Bitcoin L1 address counts
            update_config.set_bitcoin_l1_counts(3);
            
            // Keep Nostr enabled but with custom count
            update_config.set_address_count(AddressType::Nostr, 1);
            
            println!("   Enabled address types for update: {:?}", update_config.get_enabled_address_types());
            println!("   Disabled: Lightning, Liquid");
            println!("   Bitcoin L1 count: 3 addresses each");

            // Step 4: Update the UBA
            println!("\n🔄 Step 4: Updating UBA with new configuration");
            match update_uba(event_id, seed, &relays, update_config.clone()).await {
                Ok(updated_uba) => {
                    println!("   ✅ UBA updated successfully!");
                    println!("   🆕 New UBA: {}", updated_uba);
                    
                    let new_event_id = updated_uba.strip_prefix("UBA:").unwrap();
                    println!("   📋 New Event ID: {}", new_event_id);

                    // Step 5: Retrieve and display updated addresses
                    println!("\n📖 Step 5: Retrieving updated addresses");
                    match retrieve_full_with_config(&updated_uba, &relays, update_config).await {
                        Ok(updated_addresses) => {
                            display_addresses(&updated_addresses);
                            
                            // Verify filtering worked
                            verify_filtering(&updated_addresses);
                        }
                        Err(e) => println!("   ❌ Failed to retrieve updated addresses: {}", e),
                    }
                }
                Err(e) => {
                    println!("   ❌ Failed to update UBA: {}", e);
                    demonstrate_error_handling().await;
                }
            }

            // Step 6: Demonstrate custom address update
            println!("\n🎯 Step 6: Demonstrating custom address update");
            demonstrate_custom_address_update(event_id, &relays).await;
        }
        Err(e) => {
            println!("   ❌ Failed to create initial UBA: {}", e);
            println!("   💡 This might be due to relay connectivity issues.");
            
            // Demonstrate offline functionality
            demonstrate_offline_functionality();
        }
    }

    println!("\n✨ Demo completed!");
    Ok(())
}

fn display_addresses(addresses: &BitcoinAddresses) {
    println!("   📊 Address Summary:");
    println!("   ├─ Total address types: {}", addresses.addresses.len());
    println!("   ├─ Total addresses: {}", addresses.len());
    println!("   └─ Created at: {}", addresses.created_at);
    
    for (addr_type, addr_list) in &addresses.addresses {
        println!("   ");
        println!("   🏷️  {:?} ({} addresses):", addr_type, addr_list.len());
        for (i, addr) in addr_list.iter().enumerate() {
            let truncated = if addr.len() > 50 {
                format!("{}...{}", &addr[..25], &addr[addr.len()-20..])
            } else {
                addr.clone()
            };
            println!("      {}. {}", i + 1, truncated);
        }
    }
}

fn verify_filtering(addresses: &BitcoinAddresses) {
    println!("\n   🔍 Verifying address filtering:");
    
    // Check that Lightning and Liquid are not present
    if !addresses.addresses.contains_key(&AddressType::Lightning) {
        println!("   ✅ Lightning addresses correctly filtered out");
    } else {
        println!("   ❌ Lightning addresses should have been filtered out");
    }
    
    if !addresses.addresses.contains_key(&AddressType::Liquid) {
        println!("   ✅ Liquid addresses correctly filtered out");
    } else {
        println!("   ❌ Liquid addresses should have been filtered out");
    }
    
    // Check that Bitcoin L1 addresses are present with correct counts
    let bitcoin_l1_types = [AddressType::P2PKH, AddressType::P2SH, AddressType::P2WPKH, AddressType::P2TR];
    for addr_type in &bitcoin_l1_types {
        if let Some(addr_list) = addresses.addresses.get(addr_type) {
            if addr_list.len() == 3 {
                println!("   ✅ {:?}: {} addresses (correct)", addr_type, addr_list.len());
            } else {
                println!("   ⚠️  {:?}: {} addresses (expected 3)", addr_type, addr_list.len());
            }
        } else {
            println!("   ❌ {:?}: missing (should be present)", addr_type);
        }
    }
}

async fn demonstrate_custom_address_update(original_event_id: &str, relays: &[String]) {
    println!("   Creating custom address collection...");
    
    // Create a custom address collection
    let mut custom_addresses = BitcoinAddresses::new();
    custom_addresses.add_address(AddressType::P2WPKH, "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4".to_string());
    custom_addresses.add_address(AddressType::P2WPKH, "bc1qrp33g0q5c5txsp9arysrx4k6zdkfs4nce4xj0gdcccefvpysxf3qccfmv3".to_string());
    custom_addresses.add_address(AddressType::Nostr, "npub1sn0wdenkukak0d9dfczzeacvhkrgz92ak56egt7vdgzn8pv2wfqqhrjdv9".to_string());
    
    // Set metadata
    custom_addresses.metadata = Some(uba::AddressMetadata {
        label: Some("custom-update".to_string()),
        description: Some("Custom address update demo".to_string()),
        xpub: None,
        derivation_paths: None,
    });
    
    let config = UbaConfig::default();
    
    match update_uba_with_addresses(original_event_id, custom_addresses, relays, config).await {
        Ok(updated_uba) => {
            println!("   ✅ Custom address update successful!");
            println!("   🆕 Updated UBA: {}", updated_uba);
        }
        Err(e) => {
            println!("   ❌ Custom address update failed: {}", e);
        }
    }
}

async fn demonstrate_error_handling() {
    println!("\n🚨 Demonstrating error handling:");
    
    let config = UbaConfig::default();
    let relays = vec!["wss://relay.example.com".to_string()];
    
    // Test 1: Invalid event ID
    println!("   Testing invalid event ID...");
    match update_uba("invalid_event_id", "test_seed", &relays, config.clone()).await {
        Err(UbaError::InvalidUbaFormat(_)) => println!("   ✅ Correctly caught invalid event ID"),
        Err(e) => println!("   ⚠️  Unexpected error: {}", e),
        Ok(_) => println!("   ❌ Should have failed with invalid event ID"),
    }
    
    // Test 2: Empty addresses
    println!("   Testing empty address collection...");
    let empty_addresses = BitcoinAddresses::new();
    match update_uba_with_addresses("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef", empty_addresses, &relays, config).await {
        Err(UbaError::UpdateValidation(_)) => println!("   ✅ Correctly caught empty address collection"),
        Err(e) => println!("   ⚠️  Unexpected error: {}", e),
        Ok(_) => println!("   ❌ Should have failed with empty addresses"),
    }
}

fn demonstrate_offline_functionality() {
    println!("\n💻 Demonstrating offline address filtering functionality:");
    
    // Show how address filtering works without network operations
    let mut config = UbaConfig::default();
    
    println!("   🔧 Testing different filtering configurations:");
    
    // Configuration 1: Only Bitcoin L1
    config.disable_all_address_types();
    config.enable_bitcoin_l1();
    println!("   📋 Bitcoin L1 only: {:?}", config.get_enabled_address_types());
    
    // Configuration 2: Only Layer 2
    config.disable_all_address_types();
    config.set_address_type_enabled(AddressType::Lightning, true);
    config.set_address_type_enabled(AddressType::Liquid, true);
    println!("   📋 Layer 2 only: {:?}", config.get_enabled_address_types());
    
    // Configuration 3: Custom selection
    config.disable_all_address_types();
    config.set_address_type_enabled(AddressType::P2WPKH, true);
    config.set_address_type_enabled(AddressType::P2TR, true);
    config.set_address_type_enabled(AddressType::Nostr, true);
    println!("   📋 Custom selection: {:?}", config.get_enabled_address_types());
    
    // Test address generation with filtering
    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    let address_generator = AddressGenerator::new(config);
    
    match address_generator.generate_addresses(seed, Some("offline-test".to_string())) {
        Ok(addresses) => {
            println!("   ✅ Offline address generation successful!");
            println!("   📊 Generated {} address types", addresses.addresses.len());
            for (addr_type, addr_list) in &addresses.addresses {
                println!("      - {:?}: {} addresses", addr_type, addr_list.len());
            }
        }
        Err(e) => {
            println!("   ❌ Offline address generation failed: {}", e);
        }
    }
} 