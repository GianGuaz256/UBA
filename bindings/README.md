# UBA Language Bindings

This directory contains language bindings for the UBA (Unified Bitcoin Addresses) library, organized by target language for better maintainability and scalability.

## Directory Structure

```
bindings/
├── wasm/               # WebAssembly bindings (JavaScript/TypeScript)
│   ├── src/           # Source code
│   ├── types/         # TypeScript definitions
│   ├── examples/      # Test and example files
│   ├── Cargo.toml     # WASM-specific dependencies
│   ├── package.json   # Node.js configuration
│   └── build.sh       # Build script
├── python/            # Python bindings (future)
├── c/                 # C/C++ bindings (future)
├── java/              # Java bindings (future)
└── README.md          # This file
```

## Available Bindings

### ✅ WebAssembly (JavaScript/TypeScript)

**Status**: Available (with limitations)
**Location**: `wasm/`
**Languages**: JavaScript, TypeScript
**Platforms**: Web browsers, Node.js

**Features**:
- ✅ Local address generation from seeds
- ✅ UBA string parsing
- ✅ Encryption utilities
- ✅ TypeScript definitions
- ⚠️ Limited by WASM compilation issues (secp256k1-sys)
- ❌ No Nostr networking (WASM limitation)

**Build**: 
```bash
cd wasm/
./build.sh
```

**Usage**:
```javascript
import { generateAddresses, parseUbaString } from './wasm/dist/typescript/index';

const addresses = await generateAddresses({
    seed: "your-seed-phrase",
    label: "my-wallet"
});
```

## Planned Bindings

### 🔄 Python

**Status**: Planned
**Target Location**: `python/`
**Technologies**: PyO3, maturin

**Planned Features**:
- Full UBA functionality including Nostr networking
- Pythonic API design
- Type hints and documentation
- PyPI package distribution

### 🔄 C/C++

**Status**: Planned  
**Target Location**: `c/`
**Technologies**: cbindgen, FFI

**Planned Features**:
- C-compatible API
- Header file generation
- Memory management utilities
- Cross-platform support

### 🔄 Java

**Status**: Planned
**Target Location**: `java/`
**Technologies**: JNI, uniffi

**Planned Features**:
- JVM compatibility
- Maven/Gradle integration
- Android support
- Java-style error handling

## Design Principles

### Self-Contained Bindings
Each language binding is self-contained with:
- Own build scripts and configuration
- Language-specific dependencies
- Appropriate examples and tests
- Documentation in the target language

### Main Library Dependency
All bindings depend on the main UBA Rust library located at `../` (repository root), ensuring:
- Single source of truth for core functionality
- Consistent behavior across languages
- Easy maintenance and updates

### Language-Appropriate APIs
Each binding provides:
- Idiomatic APIs for the target language
- Error handling in language conventions
- Documentation in expected formats
- Integration with language ecosystems

## Building All Bindings

From the repository root:
```bash
# Build specific binding
cd bindings/wasm && ./build.sh

# Or build all available bindings
for binding in bindings/*/; do
    if [ -f "$binding/build.sh" ]; then
        echo "Building $binding..."
        cd "$binding" && ./build.sh && cd ../..
    fi
done
```

## Contributing New Bindings

To add a new language binding:

1. **Create the directory**: `bindings/{language}/`
2. **Add core files**:
   - `Cargo.toml` or equivalent dependency file
   - `build.sh` build script
   - `src/` directory for binding code
   - `examples/` directory for usage examples
   - `README.md` with language-specific documentation

3. **Follow the pattern**:
   - Depend on the main UBA library: `uba = { path = "../.." }`
   - Provide language-appropriate APIs
   - Include comprehensive examples
   - Add tests for the binding layer

4. **Update this README** with the new binding information

## Testing

Each binding should include:
- Unit tests for the binding layer
- Integration tests with the main library
- Example usage verification
- Documentation testing

## Documentation

Each binding maintains:
- API documentation in language conventions
- Usage examples
- Installation/build instructions
- Platform-specific notes

## Support Matrix

| Binding    | Status | Local Generation | Nostr Networking | Encryption | Examples |
|------------|--------|------------------|------------------|------------|----------|
| WASM       | ✅ Limited | ✅ | ❌ | ✅ | ✅ |
| Python     | 🔄 Planned | 🔄 | 🔄 | 🔄 | 🔄 |
| C/C++      | 🔄 Planned | 🔄 | 🔄 | 🔄 | 🔄 |
| Java       | 🔄 Planned | 🔄 | 🔄 | 🔄 | 🔄 |

Legend:
- ✅ Available
- ❌ Not available
- 🔄 Planned/In development

## License

All bindings inherit the license from the main UBA library (MIT OR Apache-2.0).