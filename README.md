# UBA - Unified Bitcoin Address Library

[![Crates.io](https://img.shields.io/crates/v/uba.svg)](https://crates.io/crates/uba) [![Documentation](https://docs.rs/uba/badge.svg)](https://docs.rs/uba) [![License](https://img.shields.io/crates/l/uba.svg)](LICENSE)

<div align="center">
  <img src="docs/images/uba.png" alt="UBA - Unified Bitcoin Address" width="100%"/>
</div>

A Rust library that enables the creation and retrieval of **Unified Bitcoin Addresses (UBA)** - single, concise strings that unify a user's Bitcoin addresses across **all Bitcoin layers**: Layer 1 (L1), Liquid sidechain, Lightning Network, and Nostr public keys using Nostr relays for decentralized storage.

## 🎯 What is a UBA?

A **Unified Bitcoin Address (UBA)** is a short, shareable string that acts as a pointer to a comprehensive collection of Bitcoin addresses stored on Nostr relays. Instead of sharing multiple addresses for different layers and protocols, users can share a single UBA that recipients can use to retrieve all relevant addresses across the entire Bitcoin ecosystem.

### UBA Format

```
UBA:<NostrEventID>&label=<optional-label>
```

**Examples:**
- `UBA:a1b2c3d4e5f6...` (without label)
- `UBA:a1b2c3d4e5f6...&label=my-wallet` (with label)

## 🌟 Key Features

- **🔗 Truly Unified**: Single string for **ALL** Bitcoin layers (L1, Liquid, Lightning, Nostr)
- **📱 QR-Friendly**: Short strings perfect for QR codes
- **🔒 Privacy-Preserving**: Leverages Nostr's decentralized architecture
- **🛡️ Secure**: No centralized servers, data stored across Nostr relays
- **⚡ Multi-Layer**: Supports Bitcoin L1, Liquid sidechain, Lightning Network, and Nostr
- **🔐 Optional Encryption**: ChaCha20Poly1305 encryption for sensitive data
- **📡 Public Relay Network**: Curated list of reliable Nostr relays
- **⚙️ Configurable**: Flexible address counts and custom relay support
- **🔧 Extensible**: Ready for future Bitcoin protocols and improvements

## 🚀 Quick Start

### Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
uba = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use uba::{generate, retrieve_full, AddressType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Your seed phrase (use a secure one in production!)
    let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    
    // Nostr relay URLs
    let relays = vec![
        "wss://relay.damus.io".to_string(),
        "wss://nos.lol".to_string(),
    ];
    
    // Generate a UBA with multi-layer addresses
    let uba = generate(seed, Some("my-wallet"), &relays).await?;
    println!("UBA: {}", uba);
    
    // Retrieve all addresses across all layers
    let addresses = retrieve_full(&uba, &relays).await?;
    
    // Access specific layer addresses
    if let Some(btc_addresses) = addresses.get_addresses(&AddressType::P2WPKH) {
        println!("Bitcoin SegWit: {:?}", btc_addresses);
    }
    if let Some(liquid_addresses) = addresses.get_addresses(&AddressType::Liquid) {
        println!("Liquid: {:?}", liquid_addresses);
    }
    if let Some(lightning_ids) = addresses.get_addresses(&AddressType::Lightning) {
        println!("Lightning Node IDs: {:?}", lightning_ids);
    }
    if let Some(nostr_pubkeys) = addresses.get_addresses(&AddressType::Nostr) {
        println!("Nostr Public Keys: {:?}", nostr_pubkeys);
    }
    
    Ok(())
}
```

## 📖 Detailed Usage

### 🔐 Encryption Support

UBA supports optional encryption using ChaCha20Poly1305 for secure data storage on Nostr relays:

```rust
use uba::{generate_with_config, retrieve_with_config, UbaConfig, derive_encryption_key, generate_random_key};

// Generate UBA with passphrase-derived encryption
let mut config = UbaConfig::default();
let encryption_key = derive_encryption_key("my-secret-passphrase", None);
config.set_encryption_key(encryption_key);

let uba = generate_with_config(seed, Some("encrypted-wallet"), &[], config.clone()).await?;

// Retrieve with the same encryption key
let addresses = retrieve_with_config(&uba, &[], config).await?;

// Or use a random encryption key
let mut config2 = UbaConfig::default();
let random_key = generate_random_key();
config2.set_encryption_key(random_key);
println!("Save this key: {}", hex::encode(random_key));
```

### 📡 Relay Configuration

UBA includes a curated list of reliable public Nostr relays and supports custom relay configuration:

```rust
use uba::{default_public_relays, UbaConfig};

// Use default public relays (automatic)
let config = UbaConfig::default();
let uba = generate_with_config(seed, None, &[], config).await?;

// View available public relays
println!("Public relays: {:?}", default_public_relays());

// Use custom relays
let mut config = UbaConfig::default();
config.set_custom_relays(vec![
    "wss://my-relay.com".to_string(),
    "wss://relay.damus.io".to_string(),
]);

// Add to default relays
let mut config = UbaConfig::default();
config.add_custom_relay("wss://my-personal-relay.com".to_string());
```

### ⚙️ Configurable Address Counts

Control how many addresses are generated for each Bitcoin layer:

```rust
use uba::{UbaConfig, AddressType};

let mut config = UbaConfig::default();

// Set specific counts per address type
config.set_address_count(AddressType::P2WPKH, 10);
config.set_address_count(AddressType::Liquid, 5);
config.set_address_count(AddressType::Lightning, 3);

// Set all Bitcoin L1 types at once
config.set_bitcoin_l1_counts(8);

// Set all types to the same count
config.set_all_counts(7);

let uba = generate_with_config(seed, None, &[], config).await?;
```

### Generating a UBA

```rust
use uba::{generate, generate_with_config, UbaConfig, Network};

// Simple generation (includes all layers)
let uba = generate(seed, Some("wallet-label"), &relay_urls).await?;

// With custom configuration
let mut config = UbaConfig::default();
config.network = Network::Testnet;
config.max_addresses_per_type = 10;
config.encrypt_data = true;

let uba = generate_with_config(seed, Some("wallet-label"), &relay_urls, config).await?;
```

### Retrieving Addresses

```rust
use uba::{retrieve, retrieve_full, AddressType};

// Get all addresses as a flat vector
let addresses = retrieve(&uba, &relay_urls).await?;

// Get full structured data with layer separation
let bitcoin_addresses = retrieve_full(&uba, &relay_urls).await?;

// Access specific layers
if let Some(btc_legacy) = bitcoin_addresses.get_addresses(&AddressType::P2PKH) {
    println!("Bitcoin Legacy: {:?}", btc_legacy);
}
if let Some(liquid_addrs) = bitcoin_addresses.get_addresses(&AddressType::Liquid) {
    println!("Liquid Sidechain: {:?}", liquid_addrs);
}
if let Some(ln_nodes) = bitcoin_addresses.get_addresses(&AddressType::Lightning) {
    println!("Lightning Network: {:?}", ln_nodes);
}
```

### Offline Address Generation

```rust
use uba::{AddressGenerator, UbaConfig, AddressType};

let config = UbaConfig::default();
let generator = AddressGenerator::new(config);

// Generate addresses without publishing to relays
let addresses = generator.generate_addresses(seed, Some("offline".to_string()))?;

println!("Total addresses: {}", addresses.len());
for addr_type in [AddressType::P2WPKH, AddressType::Liquid, AddressType::Lightning, AddressType::Nostr] {
    if let Some(addrs) = addresses.get_addresses(&addr_type) {
        println!("{:?}: {} addresses", addr_type, addrs.len());
    }
}
```

## 🏗️ Architecture

### How UBA Works

1. **Multi-Layer Address Generation**: Generate addresses from seed using standard derivation paths for Bitcoin L1, Liquid, Lightning, and Nostr
2. **Nostr Publishing**: Serialize all addresses as JSON and publish to Nostr relays as an event
3. **UBA Creation**: Create a UBA string using the Nostr event ID
4. **Layer-Aware Retrieval**: Parse UBA, fetch the event from relays, and deserialize addresses with layer separation

### Supported Address Types & Layers

#### 🟠 Bitcoin Layer 1
- **P2PKH**: Legacy Bitcoin addresses (starts with `1`)
- **P2SH**: SegWit-wrapped addresses (starts with `3`) 
- **P2WPKH**: Native SegWit addresses (starts with `bc1`)
- **P2TR**: Taproot addresses (starts with `bc1p`)

#### 💧 Liquid Sidechain
- **Liquid SegWit**: Liquid Network addresses for faster, private transactions

#### ⚡ Lightning Network
- **Node IDs**: Lightning Network node public keys for channel establishment and payments

#### 🔑 Nostr Protocol
- **Public Keys (npub)**: Nostr public keys in standard npub format for decentralized social networking

### Derivation Paths

- **Bitcoin Legacy (P2PKH)**: `m/44'/0'/0'/0`
- **Bitcoin P2SH-wrapped SegWit**: `m/49'/0'/0'/0`
- **Bitcoin Native SegWit (P2WPKH)**: `m/84'/0'/0'/0`
- **Bitcoin Taproot (P2TR)**: `m/86'/0'/0'/0`
- **Liquid Sidechain**: `m/84'/1776'/0'/0` (1776 = Liquid coin type)
- **Lightning Network**: `m/1017'/0'/0'` (1017 = Lightning node identity)
- **Nostr Protocol**: `m/44'/1237'/0'/0` (1237 = Proposed Nostr coin type)

## ⚙️ Configuration

```rust
use uba::{UbaConfig, Network, derive_encryption_key};

let mut config = UbaConfig::default();

// Basic configuration
config.network = Network::Bitcoin;           // Bitcoin network
config.relay_timeout = 10;                   // Relay timeout in seconds
config.max_addresses_per_type = 1;           // Default addresses per type (changed from 5 to 1)

// Encryption configuration (optional)
let encryption_key = derive_encryption_key("my-passphrase", None);
config.set_encryption_key(encryption_key);   // Enable encryption

// Relay configuration
config.set_custom_relays(vec![               // Use custom relays
    "wss://relay.damus.io".to_string(),
    "wss://nos.lol".to_string(),
]);

// Address count configuration
config.set_all_counts(10);                   // 10 addresses per type
config.set_address_count(AddressType::Lightning, 3); // Override Lightning to 3
```

## 🔒 Security Considerations

- **Seed Security**: Seeds are processed in memory and never stored
- **Deterministic Keys**: Same seed produces same addresses across all layers
- **No Key Exposure**: Private keys and extended public keys are not exposed
- **Layer Isolation**: Each layer uses appropriate derivation paths
- **Relay Privacy**: Use multiple relays for redundancy and privacy
- **ChaCha20Poly1305 Encryption**: Industry-standard authenticated encryption
- **Key Derivation**: HKDF-based key derivation from passphrases
- **Backward Compatibility**: Unencrypted data remains accessible
- **No Key Storage**: Encryption keys must be provided by the user

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Run the multi-layer example:

```bash
cargo run --example basic_usage
```

## 📝 Examples

The library includes comprehensive examples:

- **Basic Usage**: Multi-layer address generation and retrieval
- **Encryption & Relays**: Demonstrates encryption and custom relay configuration
- **CLI with Encryption**: Command-line interface with encryption support
- **Encryption Demo**: Offline encryption demonstration
- **Show Addresses**: Display addresses across all Bitcoin layers
- **Retrieve from Nostr ID**: Retrieve UBA data from a known Nostr event ID

```bash
# Multi-layer UBA example
cargo run --example basic_usage

# Encryption and relay configuration
cargo run --example encryption_and_relays

# CLI with encryption support
cargo run --example cli_with_encryption -- generate --passphrase "my-secret"
cargo run --example cli_with_encryption -- retrieve --uba "UBA:..." --passphrase "my-secret"

# Offline encryption demonstration
cargo run --example encryption_demo

# Show addresses across all layers
cargo run --example show_addresses

# Retrieve from Nostr ID
cargo run --example retrieve_from_nostr_id
```

## 🎯 Use Cases

- **Universal Bitcoin Wallets**: Single identifier for all Bitcoin layers
- **Payment Processors**: Accept payments across L1, Liquid, Lightning, and Nostr
- **Bitcoin Services**: Unified address sharing for exchanges, merchants
- **Privacy Tools**: Decentralized address storage without central servers
- **Developer Tools**: SDK for Bitcoin applications across all layers
- **Nostr Applications**: Unified identity across Bitcoin and Nostr ecosystems

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Clone the repository
2. Install Rust and Cargo
3. Install Bitcoin ecosystem dependencies
4. Run `cargo build` to build the project
5. Run `cargo test` to run tests

## 📋 Roadmap

### ✅ Completed Features
- [x] Bitcoin Layer 1 (Legacy, SegWit, Taproot)
- [x] Liquid sidechain support
- [x] Lightning Network node IDs
- [x] Nostr public keys (npub format)
- [x] ChaCha20Poly1305 encryption support
- [x] Public relay network with custom relay support
- [x] Configurable address counts per layer
- [x] Comprehensive examples and documentation
- [x] Default address count optimization (1 per type)

### 🚨 Production Readiness (Critical)
- [ ] Remove all `.unwrap()` calls and implement proper error handling
- [ ] Complete NIP-04/NIP-17 encryption implementation
- [ ] Implement structured logging and monitoring
- [ ] Add comprehensive input validation and rate limiting
- [ ] Implement connection timeouts and retry logic for relays

### 🚀 Feature Enhancements
- [ ] BOLT12 Lightning offers
- [ ] Lightning addresses (email-like format)
- [ ] Address rotation and versioning
- [ ] QR code generation utilities
- [ ] CLI tool for UBA management
- [ ] Python/JavaScript bindings
- [ ] Mobile SDK (React Native/Flutter)
- [ ] Hardware wallet integration
- [ ] Multi-signature address support
- [ ] Nostr event signing and verification
- [ ] NIP-05 identifier integration

## 🔗 Related Projects

- [Nostr Protocol](https://github.com/nostr-protocol/nostr) - Decentralized social network protocol
- [Bitcoin Core](https://github.com/bitcoin/bitcoin) - Bitcoin reference implementation
- [Elements Project](https://github.com/ElementsProject/elements) - Liquid sidechain
- [Lightning Network](https://github.com/lightningnetwork/lnd) - Bitcoin Layer 2 protocol

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license. See the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This library is experimental and should be used with caution in production environments. Always verify addresses and test thoroughly before using with real funds.

---

**Made with ❤️ for the complete Bitcoin ecosystem**