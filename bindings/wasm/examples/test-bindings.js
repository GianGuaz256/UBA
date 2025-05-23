/**
 * Test script for UBA WebAssembly bindings
 *
 * This script tests the JavaScript/TypeScript bindings for the UBA library
 * Run with: node examples/test-bindings.js
 */

const uba = require("../pkg-node/uba");

// Test seed phrase (DO NOT use in production!)
const TEST_SEED =
  "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

async function testBasicFunctionality() {
  console.log("🧪 Testing basic UBA functionality...");

  try {
    // Test parsing
    console.log("📋 Testing UBA parsing...");
    const testUbaString =
      "UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef&label=test";
    const parsed = uba.parse_uba_string(testUbaString);
    console.log("✅ Parsed UBA:", parsed);

    // Test configuration
    console.log("⚙️  Testing configuration...");
    const config = new uba.JsUbaConfig();
    config.set_network(1); // Testnet
    config.set_encrypt_data(false); // No encryption for testing
    config.set_relay_timeout(10);
    config.set_max_addresses_per_type(3); // Fewer addresses for faster testing

    console.log("✅ Config created:", {
      network: config.get_network(),
      encrypt_data: config.get_encrypt_data(),
      relay_timeout: config.get_relay_timeout(),
      max_addresses: config.get_max_addresses_per_type(),
    });

    // Test relay URLs
    console.log("📡 Testing relay functions...");
    const defaultRelays = uba.get_default_public_relays();
    console.log("✅ Default relays count:", defaultRelays.length);
    console.log("   First relay:", defaultRelays[0]);

    // Test encryption utilities
    console.log("🔐 Testing encryption utilities...");
    const encryptionKey = uba.generate_random_encryption_key();
    console.log(
      "✅ Generated encryption key:",
      encryptionKey.substring(0, 16) + "..."
    );

    const derivedKey =
      uba.derive_encryption_key_from_passphrase("test-passphrase");
    console.log("✅ Derived key:", derivedKey.substring(0, 16) + "...");

    // Test constants
    console.log("🏷️  Testing constants...");
    console.log("✅ Address types:", {
      P2PKH: uba.AddressTypes.P2PKH,
      P2WPKH: uba.AddressTypes.P2WPKH,
      Lightning: uba.AddressTypes.Lightning,
    });

    console.log("✅ Networks:", {
      Bitcoin: uba.Networks.Bitcoin,
      Testnet: uba.Networks.Testnet,
    });

    console.log("🎉 All basic tests passed!");
  } catch (error) {
    console.error("❌ Basic test failed:", error.message);
    throw error;
  }
}

async function testUbaGeneration() {
  console.log("\n🚀 Testing UBA generation (testnet)...");

  try {
    const config = new uba.JsUbaConfig();
    config.set_network(1); // Testnet
    config.set_encrypt_data(false);
    config.set_relay_timeout(15);
    config.set_max_addresses_per_type(2); // Minimal for testing

    // Use fewer, more reliable relays for testing
    const testRelays = ["wss://relay.damus.io", "wss://nos.lol"];

    console.log("⏳ Generating UBA (this may take a moment)...");

    const generatedUba = await uba.generate(
      TEST_SEED,
      "test-wallet",
      testRelays,
      config
    );

    console.log("✅ Generated UBA:", generatedUba);

    // Parse the generated UBA
    const parsed = uba.parse_uba_string(generatedUba);
    console.log("✅ Parsed generated UBA:", parsed);

    // Test retrieval
    console.log("📥 Testing address retrieval...");

    const addresses = await uba.retrieve(generatedUba, testRelays, config);

    console.log("✅ Retrieved addresses:", addresses);
    console.log("📊 Total addresses retrieved:", addresses.length);

    // Test full retrieval
    console.log("📥 Testing full address retrieval...");

    const fullAddresses = await uba.retrieve_full(
      generatedUba,
      testRelays,
      config
    );

    console.log("✅ Full address structure:");
    console.log("   Total addresses:", fullAddresses.len());
    console.log(
      "   Created at:",
      new Date(fullAddresses.get_created_at() * 1000)
    );
    console.log("   Version:", fullAddresses.get_version());

    // Test specific address types
    const p2wpkhAddresses = fullAddresses.get_addresses_by_type(
      uba.AddressTypes.P2WPKH
    );
    if (p2wpkhAddresses) {
      console.log("   P2WPKH addresses:", p2wpkhAddresses);
    }

    console.log("🎉 UBA generation and retrieval test passed!");

    return generatedUba;
  } catch (error) {
    console.error("❌ UBA generation test failed:", error.message);
    console.error(
      "   This might be due to network issues or relay connectivity"
    );
    throw error;
  }
}

async function testEncryption() {
  console.log("\n🔒 Testing encrypted UBA...");

  try {
    const config = new uba.JsUbaConfig();
    config.set_network(1); // Testnet
    config.set_encrypt_data(true);

    // Generate and set encryption key
    const encryptionKey = config.generate_random_encryption_key();
    console.log(
      "🔑 Generated encryption key:",
      encryptionKey.substring(0, 16) + "..."
    );

    config.set_relay_timeout(15);
    config.set_max_addresses_per_type(2);

    const testRelays = ["wss://relay.damus.io", "wss://nos.lol"];

    console.log("⏳ Generating encrypted UBA...");

    const encryptedUba = await uba.generate(
      TEST_SEED + " extra", // Different seed for different UBA
      "encrypted-wallet",
      testRelays,
      config
    );

    console.log("✅ Generated encrypted UBA:", encryptedUba);

    // Test retrieval with correct key
    console.log("📥 Testing retrieval with correct encryption key...");

    const decryptedAddresses = await uba.retrieve(
      encryptedUba,
      testRelays,
      config
    );

    console.log("✅ Decrypted addresses:", decryptedAddresses);

    // Test retrieval without key (should fail)
    console.log("🚫 Testing retrieval without encryption key (should fail)...");

    const configNoKey = new uba.JsUbaConfig();
    configNoKey.set_network(1);
    configNoKey.set_relay_timeout(15);

    try {
      await uba.retrieve(encryptedUba, testRelays, configNoKey);
      console.log("❌ ERROR: Retrieval without key should have failed!");
    } catch (error) {
      console.log(
        "✅ Correctly failed to retrieve without encryption key:",
        error.message
      );
    }

    console.log("🎉 Encryption test passed!");
  } catch (error) {
    console.error("❌ Encryption test failed:", error.message);
    throw error;
  }
}

async function runAllTests() {
  console.log("🧪 UBA WebAssembly Bindings Test Suite");
  console.log("=====================================\n");

  try {
    await testBasicFunctionality();

    // Skip network tests if in CI or no network
    if (process.env.SKIP_NETWORK_TESTS) {
      console.log("⏭️  Skipping network tests (SKIP_NETWORK_TESTS set)");
    } else {
      await testUbaGeneration();
      await testEncryption();
    }

    console.log("\n🎉 All tests passed successfully!");
    console.log("✅ UBA WebAssembly bindings are working correctly.");
  } catch (error) {
    console.error("\n💥 Test suite failed:", error.message);
    process.exit(1);
  }
}

// Run tests if this file is executed directly
if (require.main === module) {
  runAllTests();
}

module.exports = {
  testBasicFunctionality,
  testUbaGeneration,
  testEncryption,
  runAllTests,
};
