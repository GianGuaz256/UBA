//! Simple CLI example demonstrating UBA with encryption
//! 
//! Usage:
//!   cargo run --example cli_with_encryption -- generate --seed "your seed" --passphrase "secret"
//!   cargo run --example cli_with_encryption -- retrieve --uba "UBA:..." --passphrase "secret"

use std::env;
use uba::{
    generate_with_config, retrieve_with_config, UbaConfig, 
    derive_encryption_key, default_public_relays
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "generate" => {
            let seed = get_arg(&args, "--seed").unwrap_or_else(|| {
                "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string()
            });
            let passphrase = get_arg(&args, "--passphrase");
            let label = get_arg(&args, "--label");
            
            generate_uba(&seed, passphrase.as_deref(), label.as_deref()).await?;
        }
        "retrieve" => {
            let uba = get_arg(&args, "--uba").expect("--uba is required for retrieve command");
            let passphrase = get_arg(&args, "--passphrase");
            
            retrieve_uba(&uba, passphrase.as_deref()).await?;
        }
        "relays" => {
            list_default_relays();
        }
        _ => {
            print_usage();
        }
    }

    Ok(())
}

async fn generate_uba(seed: &str, passphrase: Option<&str>, label: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Generating UBA...");
    
    let mut config = UbaConfig::default();
    config.set_all_counts(5); // Generate 5 addresses per type
    
    // Set up encryption if passphrase provided
    if let Some(passphrase) = passphrase {
        let encryption_key = derive_encryption_key(passphrase, None);
        config.set_encryption_key(encryption_key);
        println!("🔐 Encryption enabled with passphrase");
    } else {
        println!("⚠️  No encryption (use --passphrase for encryption)");
    }
    
    // Generate UBA
    let uba = generate_with_config(seed, label, &[], config).await?;
    
    println!("\n✅ Generated UBA:");
    println!("{}", uba);
    
    if passphrase.is_some() {
        println!("\n🔑 Remember your passphrase to retrieve the addresses!");
    }
    
    println!("\n📊 This UBA contains 30 addresses across all Bitcoin layers:");
    println!("   • 5 P2PKH (Legacy) addresses");
    println!("   • 5 P2SH (SegWit v0) addresses");
    println!("   • 5 P2WPKH (Native SegWit) addresses");
    println!("   • 5 P2TR (Taproot) addresses");
    println!("   • 5 Liquid addresses");
    println!("   • 5 Lightning node public keys");
    
    Ok(())
}

async fn retrieve_uba(uba: &str, passphrase: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Retrieving addresses from UBA...");
    
    let mut config = UbaConfig::default();
    
    // Set up decryption if passphrase provided
    if let Some(passphrase) = passphrase {
        let encryption_key = derive_encryption_key(passphrase, None);
        config.set_encryption_key(encryption_key);
        println!("🔐 Decryption enabled with passphrase");
    }
    
    // Retrieve addresses
    let addresses = retrieve_with_config(uba, &[], config).await?;
    
    println!("\n✅ Retrieved {} addresses:", addresses.len());
    
    // Group and display addresses by type
    let mut bitcoin_l1 = Vec::new();
    let mut liquid = Vec::new();
    let mut lightning = Vec::new();
    
    for addr in addresses {
        if addr.starts_with('1') || addr.starts_with('3') || addr.starts_with("bc1") {
            bitcoin_l1.push(addr);
        } else if addr.starts_with("lq1") || addr.starts_with("ex1") {
            liquid.push(addr);
        } else if addr.len() == 66 && addr.chars().all(|c| c.is_ascii_hexdigit()) {
            lightning.push(addr);
        }
    }
    
    if !bitcoin_l1.is_empty() {
        println!("\n🟠 Bitcoin L1 Addresses ({}):", bitcoin_l1.len());
        for addr in bitcoin_l1 {
            println!("   {}", addr);
        }
    }
    
    if !liquid.is_empty() {
        println!("\n🔵 Liquid Addresses ({}):", liquid.len());
        for addr in liquid {
            println!("   {}", addr);
        }
    }
    
    if !lightning.is_empty() {
        println!("\n⚡ Lightning Node Public Keys ({}):", lightning.len());
        for addr in lightning {
            println!("   {}", addr);
        }
    }
    
    Ok(())
}

fn list_default_relays() {
    println!("📡 Default Public Nostr Relays:");
    for (i, relay) in default_public_relays().iter().enumerate() {
        println!("   {}. {}", i + 1, relay);
    }
    println!("\nThese relays are automatically used when no custom relays are specified.");
}

fn get_arg(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|pos| args.get(pos + 1))
        .cloned()
}

fn print_usage() {
    println!("🔐 UBA CLI with Encryption Support");
    println!();
    println!("USAGE:");
    println!("   cargo run --example cli_with_encryption -- <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("   generate    Generate a new UBA");
    println!("   retrieve    Retrieve addresses from a UBA");
    println!("   relays      List default public relays");
    println!();
    println!("GENERATE OPTIONS:");
    println!("   --seed <SEED>           BIP39 mnemonic seed (default: test seed)");
    println!("   --passphrase <PASS>     Encryption passphrase (optional)");
    println!("   --label <LABEL>         Optional label for the UBA");
    println!();
    println!("RETRIEVE OPTIONS:");
    println!("   --uba <UBA_STRING>      The UBA string to retrieve");
    println!("   --passphrase <PASS>     Decryption passphrase (if encrypted)");
    println!();
    println!("EXAMPLES:");
    println!("   # Generate encrypted UBA");
    println!("   cargo run --example cli_with_encryption -- generate --passphrase \"my-secret\"");
    println!();
    println!("   # Generate with custom seed and label");
    println!("   cargo run --example cli_with_encryption -- generate --seed \"your seed words\" --label \"my-wallet\"");
    println!();
    println!("   # Retrieve encrypted UBA");
    println!("   cargo run --example cli_with_encryption -- retrieve --uba \"UBA:abc123...\" --passphrase \"my-secret\"");
} 