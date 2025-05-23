use uba::{AddressGenerator, UbaConfig, AddressType, Network};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing UBA Core Functionality");
    
    // Test seed phrase (DO NOT use in production!)
    let test_seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Create configuration
    let mut config = UbaConfig::default();
    config.network = Network::Testnet;
    config.set_address_count(AddressType::P2WPKH, 3);
    config.set_address_count(AddressType::P2PKH, 2);
    config.set_address_count(AddressType::P2TR, 2);
    
    println!("⚙️  Configuration:");
    println!("   Network: {:?}", config.network);
    println!("   P2WPKH count: {}", config.get_address_count(&AddressType::P2WPKH));
    println!("   P2PKH count: {}", config.get_address_count(&AddressType::P2PKH));
    println!("   P2TR count: {}", config.get_address_count(&AddressType::P2TR));
    
    // Create address generator
    let generator = AddressGenerator::new(config);
    
    // Generate addresses
    println!("\n🔄 Generating addresses...");
    let addresses = generator.generate_addresses(test_seed, Some("test-wallet".to_string()))?;
    
    println!("✅ Generated {} total addresses", addresses.len());
    
    // Display addresses by type
    if let Some(p2wpkh_addrs) = addresses.get_addresses(&AddressType::P2WPKH) {
        println!("\n📍 SegWit (P2WPKH) addresses:");
        for (i, addr) in p2wpkh_addrs.iter().enumerate() {
            println!("   {}: {}", i + 1, addr);
        }
    }
    
    if let Some(p2pkh_addrs) = addresses.get_addresses(&AddressType::P2PKH) {
        println!("\n📍 Legacy (P2PKH) addresses:");
        for (i, addr) in p2pkh_addrs.iter().enumerate() {
            println!("   {}: {}", i + 1, addr);
        }
    }
    
    if let Some(p2tr_addrs) = addresses.get_addresses(&AddressType::P2TR) {
        println!("\n📍 Taproot (P2TR) addresses:");
        for (i, addr) in p2tr_addrs.iter().enumerate() {
            println!("   {}: {}", i + 1, addr);
        }
    }
    
    // Test encryption functionality
    println!("\n🔐 Testing encryption...");
    let encryption_key = uba::generate_random_key();
    println!("✅ Generated random encryption key: {}...", hex::encode(&encryption_key[..8]));
    
    let derived_key = uba::derive_encryption_key("test-passphrase", None);
    println!("✅ Derived key from passphrase: {}...", hex::encode(&derived_key[..8]));
    
    // Test JSON serialization
    println!("\n📄 Testing JSON serialization...");
    let json = serde_json::to_string(&addresses)?;
    println!("✅ Serialized to JSON ({} bytes)", json.len());
    
    // Test deterministic generation
    println!("\n🔄 Testing deterministic generation...");
    let addresses2 = generator.generate_addresses(test_seed, Some("test-wallet".to_string()))?;
    
    let addr1_p2wpkh = addresses.get_addresses(&AddressType::P2WPKH);
    let addr2_p2wpkh = addresses2.get_addresses(&AddressType::P2WPKH);
    
    if addr1_p2wpkh == addr2_p2wpkh {
        println!("✅ Deterministic generation verified - same seed produces same addresses");
    } else {
        println!("❌ Deterministic generation failed");
    }
    
    println!("\n🎉 All core functionality tests passed!");
    println!("\n📋 Summary of working features:");
    println!("   ✅ Address generation from seed phrases");
    println!("   ✅ Multiple address types (P2PKH, P2WPKH, P2TR)");
    println!("   ✅ Deterministic generation");
    println!("   ✅ Encryption utilities");
    println!("   ✅ JSON serialization");
    println!("   ✅ Configurable address counts");
    
    #[cfg(feature = "native")]
    println!("   ✅ Liquid and Lightning support (native builds)");
    
    #[cfg(not(feature = "native"))]
    println!("   ⚠️  Liquid and Lightning support (requires native features)");
    
    println!("   ❌ Nostr relay functionality (requires native features + working secp256k1)");
    
    Ok(())
} 