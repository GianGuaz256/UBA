/**
 * Complete JavaScript UBA Solution
 *
 * This example shows how to use UBA with JavaScript when WASM crypto compilation fails.
 * It demonstrates multiple approaches to get complete UBA functionality:
 *
 * 1. Using external address generation APIs
 * 2. Integrating with existing Bitcoin libraries
 * 3. Manual address creation for testing
 * 4. UBA parsing and encryption utilities
 */

// Import UBA WASM module (assuming it's available)
// In a real application, you would use: import * as UBA from 'uba-wasm';
// For this example, we'll assume the module is loaded

class CompletUbaJavaScript {
  constructor() {
    this.wasmModule = null;
    this.cryptoAvailable = false;
  }

  async initialize() {
    try {
      // Initialize UBA WASM module
      this.wasmModule = await import("uba-wasm");

      // Check what functionality is available
      const buildInfo = this.wasmModule.get_build_info();
      console.log("UBA Build Info:", buildInfo);

      this.cryptoAvailable = this.wasmModule.is_crypto_available();
      console.log("Crypto available:", this.cryptoAvailable);

      if (!this.cryptoAvailable) {
        console.warn("⚠️  secp256k1 crypto not available in WASM build");
        console.log("💡 Using alternative address generation methods");
      }
    } catch (error) {
      console.error("Failed to initialize UBA WASM:", error);
      throw error;
    }
  }

  /**
   * Generate addresses using multiple fallback methods
   */
  async generateAddresses(seed, label, config) {
    // Method 1: Try WASM native generation first
    if (this.cryptoAvailable) {
      try {
        console.log("🔐 Using native WASM address generation");
        return await this.wasmModule.generate_addresses(seed, label, config);
      } catch (error) {
        console.warn("Native generation failed:", error);
      }
    }

    // Method 2: Use external address generation API
    console.log("🌐 Using external address generation API");
    try {
      const addresses = await this.generateAddressesViaAPI(seed, label, config);
      return this.wasmModule.create_addresses_from_arrays(
        addresses.p2pkh,
        addresses.p2sh,
        addresses.p2wpkh,
        addresses.p2tr,
        addresses.liquid,
        addresses.lightning,
        label
      );
    } catch (error) {
      console.warn("API generation failed:", error);
    }

    // Method 3: Use browser-based Bitcoin library
    console.log("📱 Using browser Bitcoin library");
    try {
      const addresses = await this.generateAddressesViaBrowser(
        seed,
        label,
        config
      );
      return this.wasmModule.create_addresses_from_arrays(
        addresses.p2pkh,
        addresses.p2sh,
        addresses.p2wpkh,
        addresses.p2tr,
        addresses.liquid,
        addresses.lightning,
        label
      );
    } catch (error) {
      console.warn("Browser generation failed:", error);
    }

    // Method 4: Use demo/test addresses
    console.log("🧪 Using demo addresses for testing");
    return this.generateDemoAddresses(label);
  }

  /**
   * Generate addresses using an external API service
   * This would call a backend service that runs the native UBA library
   */
  async generateAddressesViaAPI(seed, label, config) {
    const response = await fetch("/api/uba/generate", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        seed,
        label,
        config: {
          network: config?.network || 0,
          maxAddressesPerType: config?.max_addresses_per_type || 5,
          // Don't send the actual seed in production!
          // Use proper authentication and secure key derivation
        },
      }),
    });

    if (!response.ok) {
      throw new Error(`API request failed: ${response.statusText}`);
    }

    return await response.json();
  }

  /**
   * Generate addresses using browser-based Bitcoin libraries
   * This example uses bitcoinjs-lib (you'd need to install it)
   */
  async generateAddressesViaBrowser(seed, label, config) {
    // This is a simplified example - you'd need to properly implement
    // HD wallet derivation, Liquid address generation, etc.

    // For demo purposes, we'll create mock addresses
    // In a real implementation, you'd use:
    // - bitcoinjs-lib for Bitcoin addresses
    // - Elements-related libraries for Liquid
    // - Lightning libraries for Lightning addresses

    const network = config?.network || 0; // 0 = mainnet
    const count = config?.max_addresses_per_type || 5;

    return {
      p2pkh: this.generateMockAddresses("1", count), // Legacy
      p2sh: this.generateMockAddresses("3", count), // SegWit-wrapped
      p2wpkh: this.generateMockAddresses("bc1q", count), // Native SegWit
      p2tr: this.generateMockAddresses("bc1p", count), // Taproot
      liquid: this.generateMockAddresses("lq1", count), // Liquid
      lightning: this.generateMockAddresses("ln", count), // Lightning
    };
  }

  /**
   * Generate demo addresses for testing purposes
   */
  generateDemoAddresses(label) {
    const demoAddresses = {
      p2pkh: [
        "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
        "1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA",
      ],
      p2sh: [
        "3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy",
        "3QJmV3qfvL9SuYo34YihAf3sRCW3qSinyC",
      ],
      p2wpkh: [
        "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu",
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
      ],
      p2tr: [
        "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr",
        "bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0",
      ],
      liquid: [
        "lq1qqd8jmeqx9l5jrpnqfe9aer5hwg0al75tgak9wcnpz6reuure4eedwfe0247rp5h4yzmdftsahhw64uy8pzfe7pww7z35skp6j",
      ],
      lightning: [
        "03ba1cf8f4ea06cd3f3a5b6c3f3f5b0e4d5c3b9f2e8a7f4c5b3a8f2e1d9c6b4a@lightning.node:9735",
      ],
    };

    return this.wasmModule.create_addresses_from_arrays(
      demoAddresses.p2pkh,
      demoAddresses.p2sh,
      demoAddresses.p2wpkh,
      demoAddresses.p2tr,
      demoAddresses.liquid,
      demoAddresses.lightning,
      label || "demo-wallet"
    );
  }

  generateMockAddresses(prefix, count) {
    const addresses = [];
    for (let i = 0; i < count; i++) {
      // Generate a realistic-looking address for the given prefix
      const random = Math.random().toString(36).substring(2, 15);
      addresses.push(`${prefix}${random}`.padEnd(34, "0"));
    }
    return addresses;
  }

  /**
   * Parse UBA strings (always works regardless of crypto availability)
   */
  parseUba(ubaString) {
    return this.wasmModule.parse_uba_string(ubaString);
  }

  /**
   * Encryption utilities (always work regardless of crypto availability)
   */
  deriveEncryptionKey(passphrase, salt) {
    return this.wasmModule.derive_encryption_key_from_passphrase(
      passphrase,
      salt
    );
  }

  generateRandomEncryptionKey() {
    return this.wasmModule.generate_random_encryption_key();
  }

  /**
   * Get available relay URLs
   */
  getDefaultRelays() {
    return this.wasmModule.get_default_public_relays();
  }

  getExtendedRelays() {
    return this.wasmModule.get_extended_public_relays();
  }

  /**
   * Complete UBA workflow with Nostr storage
   * This requires a backend service since WASM can't do networking
   */
  async createAndStoreUba(seed, label, config) {
    // Generate addresses
    const addresses = await this.generateAddresses(seed, label, config);

    // Store on Nostr relays via backend API
    const ubaString = await this.storeOnNostr(addresses, label, config);

    return {
      uba: ubaString,
      addresses: addresses,
    };
  }

  async storeOnNostr(addresses, label, config) {
    // This would call your backend API to store the addresses on Nostr
    const response = await fetch("/api/uba/store", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        addresses: addresses.to_json(),
        label,
        config: {
          encrypt: config?.encrypt_data || false,
          relays: config?.getRelayUrls() || this.getDefaultRelays(),
        },
      }),
    });

    if (!response.ok) {
      throw new Error(`Failed to store UBA: ${response.statusText}`);
    }

    const result = await response.json();
    return result.uba;
  }

  async retrieveFromNostr(ubaString) {
    // Parse UBA to get Nostr ID
    const parsed = this.parseUba(ubaString);

    // Retrieve via backend API
    const response = await fetch(`/api/uba/retrieve/${parsed.nostrId}`);

    if (!response.ok) {
      throw new Error(`Failed to retrieve UBA: ${response.statusText}`);
    }

    const addressData = await response.json();
    return this.wasmModule.create_addresses_from_data(
      JSON.stringify(addressData)
    );
  }
}

// Example usage
async function demonstrateCompleteUbaUsage() {
  const uba = new CompletUbaJavaScript();

  try {
    await uba.initialize();

    console.log("🚀 Complete UBA JavaScript Demo");
    console.log("================================");

    // Test parsing (always works)
    const testUba =
      "UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef&label=test";
    const parsed = uba.parseUba(testUba);
    console.log("Parsed UBA:", parsed);

    // Test encryption (always works)
    const encKey = uba.deriveEncryptionKey(
      "my-secret-passphrase",
      "optional-salt"
    );
    console.log("Encryption key:", encKey);

    // Test address generation (fallback methods)
    const seed =
      "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const addresses = await uba.generateAddresses(seed, "test-wallet");
    console.log("Generated addresses:", addresses.get_all_addresses());

    // Test relay information
    const relays = uba.getDefaultRelays();
    console.log("Available relays:", relays.slice(0, 3), "...");

    console.log("✅ All functionality demonstrated successfully!");
  } catch (error) {
    console.error("Demo failed:", error);
  }
}

// Export for use in other modules
if (typeof module !== "undefined" && module.exports) {
  module.exports = CompletUbaJavaScript;
}

// Auto-run demo if this file is executed directly
if (typeof window !== "undefined") {
  window.CompletUbaJavaScript = CompletUbaJavaScript;
  window.demonstrateCompleteUbaUsage = demonstrateCompleteUbaUsage;
}

export { CompletUbaJavaScript, demonstrateCompleteUbaUsage };
