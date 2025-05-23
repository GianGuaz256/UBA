# UBA WebAssembly API Documentation

## Overview

The UBA WebAssembly bindings provide JavaScript/TypeScript access to the UBA (Unified Bitcoin Addresses) library functionality.

## Installation

```bash
# For web
cp dist/web/* your-project/
```

## Basic Usage

### JavaScript (Web)
```javascript
import init, { generate_addresses, parse_uba_string, JsUbaConfig } from './uba.js';

await init();

const config = new JsUbaConfig();
const addresses = generate_addresses("your-seed-phrase", "label", config);
```

### TypeScript (High-level API)
```typescript
import { generateUba, retrieveAddresses } from './typescript/index';

const result = await generateUba({
    seed: "your-seed-phrase",
    label: "my-wallet",
    network: 1, // Testnet
});
```

## API Reference

See the TypeScript definitions in `types/index.d.ts` for complete type information.

### Core Functions

- `generate_addresses(seed, label, config)` - Generate addresses locally (WASM-only)
- `parse_uba_string(uba)` - Parse UBA string components

### Configuration

- `JsUbaConfig` - Configuration class for UBA operations
- `AddressTypes` - Address type constants
- `Networks` - Network constants

### Utilities

- `generate_random_encryption_key()` - Generate random encryption key
- `derive_encryption_key_from_passphrase(passphrase, salt)` - Derive key from passphrase
- `get_default_public_relays()` - Get default relay URLs
