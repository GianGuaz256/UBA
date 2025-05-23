# UBA WebAssembly Bindings

WebAssembly bindings for the UBA (Unified Bitcoin Addresses) library, providing JavaScript and TypeScript support for web browsers and Node.js applications.

## ⚠️ Current Limitations

The WASM bindings currently have limitations due to WebAssembly compilation issues:

- **✅ Available**: Local address generation, UBA parsing, encryption utilities
- **❌ Not Available**: Nostr relay networking (WASM limitation with secp256k1-sys)

For full UBA functionality including Nostr relay operations, use the native Rust library or wait for pure-Rust crypto alternatives.

## 🚀 Quick Start

### Installation

```bash
# Build the bindings
./build.sh

# For Node.js projects
cp -r dist/nodejs/* your-project/

# For web projects  
cp -r dist/web/* your-project/

# For TypeScript projects
cp -r dist/typescript/* your-project/
```

### Basic Usage

#### JavaScript (Node.js)
```javascript
const uba = require('./uba_wasm');

// Generate addresses locally
const config = new uba.JsUbaConfig();
config.set_network(uba.Networks.Testnet);

const addresses = uba.generate_addresses(
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
    "my-wallet",
    config
);

console.log('Generated addresses:', addresses.get_all_addresses());
```

#### TypeScript (High-level API)
```typescript
import { generateAddresses, parseUbaString } from './typescript/index';

// Generate addresses locally (no Nostr)
const addresses = await generateAddresses({
    seed: "your-seed-phrase",
    label: "my-wallet",
    network: 1 // Testnet
});

// Parse UBA strings
const parsed = parseUbaString("UBA:1234...&label=test");
console.log(parsed);
```

#### Web Browser
```html
<script type="module">
import init, { generate_addresses, JsUbaConfig, Networks } from './uba_wasm.js';

async function main() {
    await init();
    
    const config = new JsUbaConfig();
    config.set_network(Networks.Testnet);
    
    const addresses = generate_addresses("seed-phrase", "label", config);
    console.log(addresses.get_all_addresses());
}

main();
</script>
```

## 📁 Directory Structure

```
bindings/wasm/
├── src/
│   ├── lib.rs           # WASM bindings implementation
│   └── js/
│       └── index.ts     # High-level TypeScript wrapper
├── types/
│   └── index.d.ts       # TypeScript type definitions
├── examples/
│   ├── test-bindings.js # JavaScript test examples
│   ├── test-typescript.ts # TypeScript test examples
│   └── dist/            # Built example files
├── dist/                # Distribution packages
│   ├── web/             # Web browser target
│   ├── nodejs/          # Node.js target
│   ├── bundler/         # Bundler target (webpack, rollup)
│   └── typescript/      # TypeScript wrapper
├── pkg*/                # Generated wasm-pack outputs
├── Cargo.toml           # WASM-specific dependencies
├── package.json         # Node.js configuration
├── build.sh             # Build script
├── API.md               # API documentation
└── README.md            # This file
```

## 🔧 Building

### Prerequisites

- Rust and Cargo
- `wasm-pack` (auto-installed by build script)
- Node.js and npm (for testing)

### Build Commands

```bash
# Full build with tests
./build.sh

# Skip tests
./build.sh --skip-tests

# Skip cleanup
./build.sh --skip-clean

# Help
./build.sh --help
```

### Build Targets

The build script generates packages for multiple targets:

- **Web**: ES modules for browsers
- **Node.js**: CommonJS for Node.js applications
- **Bundler**: For webpack, rollup, etc.
- **TypeScript**: High-level wrapper with type definitions

## 🧪 Testing

```bash
# JavaScript tests
node examples/test-bindings.js

# TypeScript tests (requires ts-node)
npm install -g typescript ts-node
npx ts-node examples/test-typescript.ts

# Browser tests (manual)
open examples/dist/web-example.html
```

## 📚 API Reference

### Core Functions

#### `generate_addresses(seed, label, config)`
Generate Bitcoin addresses locally from a seed phrase.

**Parameters:**
- `seed: string` - BIP39 mnemonic or hex private key
- `label: string | null` - Optional label for the addresses
- `config: JsUbaConfig` - Configuration object

**Returns:** `JsBitcoinAddresses`

#### `parse_uba_string(uba)`
Parse a UBA string and extract its components.

**Parameters:**
- `uba: string` - UBA string to parse

**Returns:** `Object` with `nostrId` and optional `label`

### Configuration

#### `JsUbaConfig`
Configuration class for address generation.

**Methods:**
- `set_network(network: number)` - Set Bitcoin network (0=Bitcoin, 1=Testnet)
- `set_encrypt_data(encrypt: boolean)` - Enable/disable encryption
- `set_encryption_key_hex(key: string)` - Set encryption key
- `set_address_count(type: number, count: number)` - Set address count per type

### Constants

#### `AddressTypes`
- `P2PKH: 0` - Legacy addresses
- `P2SH: 1` - P2SH-wrapped SegWit
- `P2WPKH: 2` - Native SegWit
- `P2TR: 3` - Taproot
- `Lightning: 4` - Lightning Network
- `Liquid: 5` - Liquid sidechain

#### `Networks`
- `Bitcoin: 0` - Bitcoin mainnet
- `Testnet: 1` - Bitcoin testnet
- `Signet: 2` - Bitcoin signet
- `Regtest: 3` - Bitcoin regtest

### Utilities

#### Encryption
- `generate_random_encryption_key()` - Generate random 32-byte key
- `derive_encryption_key_from_passphrase(passphrase, salt?)` - Derive key from passphrase

#### Relays
- `get_default_public_relays()` - Get default Nostr relay URLs
- `get_extended_public_relays()` - Get extended relay list

## 🔧 Troubleshooting

### WASM Compilation Issues

If you encounter `secp256k1-sys` compilation errors:

1. **Install LLVM with WASM support (macOS):**
   ```bash
   brew install llvm
   export AR=/opt/homebrew/opt/llvm/bin/llvm-ar
   export CC=/opt/homebrew/opt/llvm/bin/clang
   ```

2. **Alternative**: Wait for pure-Rust crypto implementation

### Import Path Issues

The TypeScript wrapper imports will work after building. The import paths reference the generated WASM package.

### Memory Issues

For large address generations, consider:
- Using smaller address counts
- Calling `wee_alloc` for memory optimization
- Processing in batches

## 🚀 Future Enhancements

### Planned Features
- Pure-Rust crypto backend (no C dependencies)
- BOLT12 Lightning offers support
- Address rotation and versioning
- QR code generation utilities
- Optimized WASM bundle sizes

### Known Limitations
- No async/await support for networking
- Limited to local address generation
- Larger bundle size due to crypto dependencies

## 🤝 Contributing

### Adding Features
1. Update `src/lib.rs` for new WASM bindings
2. Update `src/js/index.ts` for TypeScript wrapper
3. Add TypeScript definitions to `types/index.d.ts`
4. Update tests and examples
5. Update documentation

### Testing Additions
- Add JavaScript tests to `examples/test-bindings.js`
- Add TypeScript tests to `examples/test-typescript.ts`
- Update browser examples

## 📄 License

Inherits license from main UBA library (MIT OR Apache-2.0).

## 🔗 Related

- [Main UBA Library](../../README.md)
- [WASM Book](https://rustwasm.github.io/docs/book/)
- [wasm-bindgen Guide](https://rustwasm.github.io/wasm-bindgen/) 