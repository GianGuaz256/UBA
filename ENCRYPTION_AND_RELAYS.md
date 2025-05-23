# UBA Encryption and Relay Configuration

This document describes the encryption and relay configuration features added to the UBA library.

## 🔐 Encryption Support

UBA now supports optional ChaCha20Poly1305 encryption for securing address data stored on Nostr relays. This provides an additional layer of privacy and security for sensitive wallet information.

### Key Features

- **ChaCha20Poly1305 Encryption**: Industry-standard authenticated encryption
- **HKDF Key Derivation**: Secure key derivation from passphrases using HKDF-SHA256
- **Random Key Generation**: Cryptographically secure random key generation
- **Backward Compatibility**: Unencrypted data remains accessible
- **Optional**: Encryption is completely optional and disabled by default

### Usage Examples

#### Passphrase-Derived Encryption

```rust
use uba::{generate_with_config, retrieve_with_config, UbaConfig, derive_encryption_key};

// Generate encryption key from passphrase
let passphrase = "my-secret-passphrase-2024";
let encryption_key = derive_encryption_key(passphrase, None);

// Configure UBA with encryption
let mut config = UbaConfig::default();
config.set_encryption_key(encryption_key);

// Generate encrypted UBA
let uba = generate_with_config(seed, Some("encrypted-wallet"), &[], config.clone()).await?;

// Retrieve with same encryption key
let addresses = retrieve_with_config(&uba, &[], config).await?;
```

#### Random Key Encryption

```rust
use uba::{generate_random_key, UbaConfig};

// Generate random encryption key
let random_key = generate_random_key();
println!("Save this key: {}", hex::encode(random_key));

// Use random key for encryption
let mut config = UbaConfig::default();
config.set_encryption_key(random_key);
```

#### Direct Encryption/Decryption

```rust
use uba::{UbaEncryption, derive_encryption_key};

let key = derive_encryption_key("my-passphrase", None);
let encryption = UbaEncryption::new(key);

// Encrypt data
let encrypted = encryption.encrypt("sensitive data")?;
println!("Encrypted: {}", encrypted);

// Decrypt data
let decrypted = encryption.decrypt(&encrypted)?;
println!("Decrypted: {}", decrypted);
```

### Security Considerations

1. **Key Management**: Encryption keys are never stored by the library - users must manage them
2. **Passphrase Security**: Use strong, unique passphrases for key derivation
3. **Key Storage**: Store random keys securely (hardware wallets, secure storage)
4. **Backward Compatibility**: Old unencrypted UBAs remain accessible
5. **Salt Support**: Optional salt parameter for key derivation (recommended for production)

## 📡 Relay Configuration

UBA includes a curated list of reliable public Nostr relays and supports flexible relay configuration.

### Public Relay Network

The library includes 12 carefully selected public Nostr relays:

```rust
use uba::default_public_relays;

let relays = default_public_relays();
// Returns: ["wss://relay.damus.io", "wss://nos.lol", ...]
```

**Default Public Relays:**
1. `wss://relay.damus.io` - Damus (Cloudflare)
2. `wss://nos.lol` - NOS (Hetzner)
3. `wss://relay.snort.social` - Snort (Cloudflare)
4. `wss://nostr.wine` - Nostr Wine (Cloudflare)
5. `wss://relay.nostr.band` - Nostr Band (Hetzner)
6. `wss://nostr.mutinywallet.com` - Mutiny Wallet (Amazon)
7. `wss://relay.primal.net` - Primal (Cloudflare)
8. `wss://relay.nostrati.com` - Nostrati (Digital Ocean)
9. `wss://nostr.sethforprivacy.com` - Seth for Privacy
10. `wss://offchain.pub` - Offchain Pub (MULTACOM)
11. `wss://relay.nostrplebs.com` - Nostr Plebs (Hetzner)
12. `wss://purplepag.es` - Purple Pages

### Custom Relay Configuration

#### Use Custom Relays Only

```rust
use uba::UbaConfig;

let mut config = UbaConfig::default();
config.set_custom_relays(vec![
    "wss://my-relay.com".to_string(),
    "wss://relay.damus.io".to_string(),
]);

// Generate UBA using only these relays
let uba = generate_with_config(seed, None, &[], config).await?;
```

#### Add to Default Relays

```rust
let mut config = UbaConfig::default();
config.add_custom_relay("wss://my-personal-relay.com".to_string());

// Now uses default relays + your custom relay
```

#### Reset to Defaults

```rust
config.use_default_relays();
// Removes all custom relays, back to default public relays
```

### Relay Selection Criteria

Public relays were selected based on:

- **Reliability**: High uptime and stable connections
- **Performance**: Fast response times and good bandwidth
- **Geographic Distribution**: Servers across different regions
- **Infrastructure**: Hosted on reliable cloud providers
- **Reputation**: Well-known and trusted in the Nostr community

## ⚙️ Configurable Address Counts

Control exactly how many addresses are generated for each Bitcoin layer.

### Per-Type Configuration

```rust
use uba::{UbaConfig, AddressType};

let mut config = UbaConfig::default();

// Set specific counts for each address type
config.set_address_count(AddressType::P2WPKH, 10);    // 10 SegWit addresses
config.set_address_count(AddressType::Liquid, 5);     // 5 Liquid addresses
config.set_address_count(AddressType::Lightning, 3);  // 3 Lightning node IDs
```

### Bulk Configuration

```rust
// Set all Bitcoin L1 types to same count
config.set_bitcoin_l1_counts(8);  // 8 each: P2PKH, P2SH, P2WPKH, P2TR

// Set all types to same count
config.set_all_counts(7);  // 7 addresses per type (42 total)
```

### Query Configuration

```rust
// Check current configuration
let p2wpkh_count = config.get_address_count(&AddressType::P2WPKH);
println!("P2WPKH addresses: {}", p2wpkh_count);
```

## 🔧 Configuration API

### UbaConfig Structure

```rust
pub struct UbaConfig {
    pub network: Network,                           // Bitcoin network
    pub encrypt_data: bool,                         // Legacy encryption flag
    pub encryption_key: Option<[u8; 32]>,          // Encryption key
    pub relay_timeout: u64,                        // Relay timeout (seconds)
    pub max_addresses_per_type: usize,             // Default address count
    pub address_counts: HashMap<AddressType, usize>, // Per-type counts
    pub custom_relays: Option<Vec<String>>,        // Custom relay URLs
}
```

### Key Methods

#### Encryption Methods
- `set_encryption_key(key: [u8; 32])` - Set encryption key
- `set_encryption_key_from_hex(hex: &str)` - Set key from hex string
- `generate_random_encryption_key()` - Generate and set random key
- `is_encryption_enabled()` - Check if encryption is enabled
- `get_encryption_key_hex()` - Get key as hex string

#### Relay Methods
- `set_custom_relays(relays: Vec<String>)` - Set custom relay list
- `add_custom_relay(url: String)` - Add relay to current list
- `get_relay_urls()` - Get current relay URLs
- `use_default_relays()` - Reset to default public relays

#### Address Count Methods
- `set_address_count(type, count)` - Set count for specific type
- `get_address_count(type)` - Get count for specific type
- `set_bitcoin_l1_counts(count)` - Set all L1 types to same count
- `set_all_counts(count)` - Set all types to same count

## 🚀 Migration Guide

### From Previous Versions

The new features are backward compatible:

```rust
// Old code still works
let uba = generate(seed, Some("label"), &relay_urls).await?;

// New code with encryption
let mut config = UbaConfig::default();
config.set_encryption_key(derive_encryption_key("passphrase", None));
let uba = generate_with_config(seed, Some("label"), &[], config).await?;
```

### Best Practices

1. **Use Default Relays**: Start with default public relays for reliability
2. **Enable Encryption**: Use encryption for sensitive wallet data
3. **Secure Key Management**: Store encryption keys securely
4. **Reasonable Address Counts**: Balance between utility and performance
5. **Test Configuration**: Verify settings before production use

## 📊 Performance Impact

### Encryption Overhead
- **Encryption**: ~1ms per operation (negligible)
- **Key Derivation**: ~10ms (one-time cost)
- **Storage**: ~33% increase in data size (base64 encoding)

### Relay Configuration
- **Default Relays**: Automatic load balancing across 12 relays
- **Custom Relays**: User-controlled for specific requirements
- **Timeout Configuration**: Adjustable for different network conditions

## 🔍 Examples

See the following examples for complete implementations:

- `examples/encryption_demo.rs` - Offline encryption demonstration
- `examples/encryption_and_relays.rs` - Full network example
- `examples/cli_with_encryption.rs` - Command-line interface
- `examples/configurable_counts.rs` - Address count configuration

## 🛡️ Security Audit

The encryption implementation uses:

- **ChaCha20Poly1305**: AEAD cipher with authentication
- **HKDF-SHA256**: Key derivation function
- **Random Number Generation**: OS-provided entropy
- **Constant-Time Operations**: Timing attack resistant
- **Memory Safety**: Rust's memory safety guarantees

No cryptographic keys are stored or logged by the library. 