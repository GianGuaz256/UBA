//! UBA (Unified Bitcoin Addresses) - A comprehensive library for generating and retrieving 
//! Bitcoin addresses across all layers (L1, Liquid, Lightning) using Nostr relays.
//!
//! UBA provides a single string format that unifies Bitcoin addresses from different layers
//! and protocols, storing them securely on decentralized Nostr relays with optional encryption.
//!
//! # Quick Start
//!
//! ```rust
//! use uba::{generate, retrieve, UbaConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Generate UBA with default configuration
//!     let seed = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
//!     let relays = vec!["wss://relay.damus.io".to_string()];
//!     
//!     let uba = generate(seed, Some("my-wallet"), &relays).await?;
//!     println!("Generated UBA: {}", uba);
//!     
//!     // Retrieve addresses
//!     let addresses = retrieve(&uba, &relays).await?;
//!     println!("Retrieved {} addresses", addresses.len());
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - **Multi-layer support**: Bitcoin L1, Liquid sidechain, Lightning Network
//! - **Deterministic generation**: Same seed always produces same addresses
//! - **Nostr integration**: Decentralized storage on Nostr relays
//! - **Optional encryption**: Secure data with ChaCha20Poly1305 encryption
//! - **Configurable address counts**: Flexible control over address generation
//! - **Public relay list**: Curated list of reliable Nostr relays

pub mod error;
pub mod types;
pub mod address;
pub mod nostr_client;
pub mod uba;
pub mod encryption;

// Re-export main types and functions for convenience
pub use error::{UbaError, Result};
pub use types::*;
pub use address::AddressGenerator;
pub use nostr_client::NostrClient;
pub use uba::{generate, generate_with_config, retrieve, retrieve_with_config, retrieve_full, retrieve_full_with_config, parse_uba};
pub use encryption::{UbaEncryption, derive_encryption_key, generate_random_key};

// Re-export commonly used external types
pub use bitcoin::Network;
pub use nostr::Url;
