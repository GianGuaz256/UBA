//! Example showing how to retrieve Bitcoin addresses from a Nostr ID
//!
//! This example demonstrates how to:
//! - Retrieve address data directly from a Nostr event ID
//! - Parse and display the retrieved information
//! - Handle different relay scenarios

use uba::{AddressType, NostrClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 UBA Nostr ID Retrieval Example\n");

    // Example Nostr event ID (64-character hex string)
    // In a real scenario, this would be extracted from a UBA string
    let nostr_id = "73fbd97ad62923d14d119cb2af8bb855a2dc5b5dd0adbafdff217af115922c4b";

    // Example relay URLs - these should be working Nostr relays
    let relay_urls = vec![
        "wss://relay.lifpay.me".to_string(),
        "wss://a.nos.lol".to_string(),
        "wss://ditto.pub/relay".to_string(),
    ];

    println!("🆔 Nostr Event ID: {}", nostr_id);
    println!("🔗 Relay URLs: {:?}\n", relay_urls);

    // Create Nostr client for retrieval
    println!("🔄 Creating Nostr client and connecting to relays...");

    let nostr_client = uba::nostr_client::NostrClient::new(10)?; // 10 second timeout

    match nostr_client.connect_to_relays(&relay_urls).await {
        Ok(_) => {
            println!("✅ Connected to relays successfully\n");

            // Attempt to retrieve the address data
            println!("📥 Retrieving address data from Nostr event...");

            match nostr_client.retrieve_addresses(nostr_id).await {
                Ok(addresses) => {
                    println!("✅ Successfully retrieved address data!\n");

                    // Display the retrieved information
                    display_address_info(&addresses);
                }
                Err(e) => {
                    println!("❌ Failed to retrieve addresses: {}", e);
                    println!("   This could mean:");
                    println!("   - The Nostr event doesn't exist");
                    println!("   - The event is not UBA data");
                    println!("   - The relays don't have this event");
                    println!("   - Network connectivity issues\n");

                    // Show how to construct a UBA for testing
                    show_uba_construction_example(nostr_id);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to connect to relays: {}", e);
            println!("   This usually means network connectivity issues or relay downtime\n");

            // Show how this would work with real data
            show_retrieval_workflow_example();
        }
    }

    // Disconnect from relays
    nostr_client.disconnect().await;

    println!("🎯 Retrieval example completed!");
    Ok(())
}

/// Display detailed information about retrieved addresses
fn display_address_info(addresses: &uba::BitcoinAddresses) {
    println!("📊 RETRIEVED ADDRESS INFORMATION:");
    println!("   Total addresses: {}", addresses.len());
    println!("   Created: {}", addresses.created_at);
    println!("   Version: {}", addresses.version);

    // Show metadata if available
    if let Some(metadata) = &addresses.metadata {
        println!("\n🏷️  METADATA:");
        if let Some(label) = &metadata.label {
            println!("   Label: {}", label);
        }
        if let Some(description) = &metadata.description {
            println!("   Description: {}", description);
        }
        if let Some(paths) = &metadata.derivation_paths {
            println!("   Derivation Paths: {}", paths.join(", "));
        }
    }

    // Display addresses by type
    println!("\n🟠 BITCOIN L1 ADDRESSES:");
    display_addresses_by_type(addresses, &AddressType::P2PKH, "Legacy (P2PKH)");
    display_addresses_by_type(addresses, &AddressType::P2SH, "SegWit-wrapped (P2SH)");
    display_addresses_by_type(addresses, &AddressType::P2WPKH, "Native SegWit (P2WPKH)");
    display_addresses_by_type(addresses, &AddressType::P2TR, "Taproot (P2TR)");

    println!("\n💧 LIQUID ADDRESSES:");
    display_addresses_by_type(addresses, &AddressType::Liquid, "Liquid SegWit");

    println!("\n⚡ LIGHTNING ADDRESSES:");
    display_addresses_by_type(addresses, &AddressType::Lightning, "Lightning Node IDs");
}

/// Helper function to display addresses of a specific type
fn display_addresses_by_type(
    addresses: &uba::BitcoinAddresses,
    addr_type: &AddressType,
    type_name: &str,
) {
    if let Some(addrs) = addresses.get_addresses(addr_type) {
        println!("   {} ({} addresses):", type_name, addrs.len());
        for (i, addr) in addrs.iter().enumerate() {
            println!("     [{}] {}", i + 1, addr);
        }
    } else {
        println!("   {}: No addresses found", type_name);
    }
}

/// Show how to construct a UBA from a Nostr ID
fn show_uba_construction_example(nostr_id: &str) {
    println!("💡 HOW TO CONSTRUCT A UBA:");
    println!("   If you had a valid Nostr event with UBA data, you could:");
    println!("   1. Create UBA without label: UBA:{}", nostr_id);
    println!(
        "   2. Create UBA with label:    UBA:{}&label=my-wallet",
        nostr_id
    );
    println!("   3. Use uba::retrieve() or uba::retrieve_full() to get addresses\n");

    println!("📝 EXAMPLE CODE:");
    println!(
        r#"   let uba = "UBA:{}";
   let addresses = uba::retrieve(uba, &relay_urls).await?;"#,
        nostr_id
    );
}

/// Show the typical workflow for UBA retrieval
fn show_retrieval_workflow_example() {
    println!("🔄 TYPICAL UBA RETRIEVAL WORKFLOW:");
    println!("   1. Parse UBA string:        uba::parse_uba(uba_string)");
    println!("   2. Extract Nostr ID:        parsed.nostr_id");
    println!("   3. Connect to relays:       NostrClient::new().connect_to_relays()");
    println!("   4. Retrieve addresses:      client.retrieve_addresses(nostr_id)");
    println!("   5. Use the addresses:       Display, send payments, etc.\n");

    println!("📝 COMPLETE EXAMPLE:");
    println!(
        r#"   let parsed = uba::parse_uba("UBA:abc123...&label=wallet")?;
   let client = NostrClient::new(10)?;
   client.connect_to_relays(&relay_urls).await?;
   let addresses = client.retrieve_addresses(&parsed.nostr_id).await?;
   
   // Use addresses for payments across all Bitcoin layers
   let btc_addr = addresses.get_addresses(&AddressType::P2WPKH);
   let liquid_addr = addresses.get_addresses(&AddressType::Liquid);
   let lightning_node = addresses.get_addresses(&AddressType::Lightning);"#
    );
}
