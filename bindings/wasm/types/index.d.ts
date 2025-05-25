// TypeScript definitions for UBA WebAssembly bindings

export interface ParsedUba {
  nostrId: string;
  label?: string;
}

export declare class JsUbaConfig {
  constructor();
  
  // Network configuration
  network: number; // 0=Bitcoin, 1=Testnet, 2=Signet, 3=Regtest
  
  // Encryption settings
  encrypt_data: boolean;
  
  // Relay settings
  relay_timeout: number;
  max_addresses_per_type: number;
  
  // Methods
  setEncryptionKeyHex(keyHex: string): void;
  generateRandomEncryptionKey(): string;
  getEncryptionKeyHex(): string | undefined;
  
  setCustomRelays(relays: string[]): void;
  addCustomRelay(relayUrl: string): void;
  getRelayUrls(): string[];
  useDefaultRelays(): void;
  
  setAddressCount(addressType: number, count: number): void;
  setBitcoinL1Counts(count: number): void;
  setAllCounts(count: number): void;
}

export declare class JsBitcoinAddresses {
  // Methods
  getAllAddresses(): string[];
  getAddressesByType(addressType: number): string[] | undefined;
  len(): number;
  isEmpty(): boolean;
  getCreatedAt(): number;
  getVersion(): number;
  toJson(): string;
}

// Address type constants
export declare const AddressTypes: {
  readonly P2PKH: 0;
  readonly P2SH: 1;
  readonly P2WPKH: 2;
  readonly P2TR: 3;
  readonly Lightning: 4;
  readonly Liquid: 5;
};

// Network type constants
export declare const Networks: {
  readonly Bitcoin: 0;
  readonly Testnet: 1;
  readonly Signet: 2;
  readonly Regtest: 3;
};

// Main WASM exported functions (matching actual exports)
export declare function generate_addresses(
  seed: string,
  label?: string,
  config?: JsUbaConfig
): JsBitcoinAddresses;

export declare function create_addresses_from_data(
  addressesJson: string
): JsBitcoinAddresses;

export declare function create_addresses_from_arrays(
  p2pkhAddresses?: string[],
  p2shAddresses?: string[],
  p2wpkhAddresses?: string[],
  p2trAddresses?: string[],
  liquidAddresses?: string[],
  lightningAddresses?: string[],
  label?: string
): JsBitcoinAddresses;

export declare function parse_uba_string(uba: string): ParsedUba;

// Utility functions (matching actual exports)
export declare function derive_encryption_key_from_passphrase(
  passphrase: string,
  salt?: string
): string;

export declare function generate_random_encryption_key(): string;

export declare function get_default_public_relays(): string[];

export declare function get_extended_public_relays(): string[];

export declare function is_crypto_available(): boolean;

export declare function get_build_info(): {
  cryptoAvailable: boolean;
  version: string;
  target: string;
  availableFeatures: string[];
  limitations: string[];
};

// Error types
export interface UbaError extends Error {
  name: 'UbaError';
  message: string;
}

// Type aliases for convenience
export type AddressType = 0 | 1 | 2 | 3 | 4 | 5;
export type Network = 0 | 1 | 2 | 3;

// High-level convenience interface
export interface UbaOptions {
  seed: string;
  label?: string;
  network?: Network;
  encryptData?: boolean;
  encryptionKey?: string;
  addressCounts?: Partial<Record<AddressType, number>>;
  relayTimeout?: number;
}

// High-level wrapper functions (these would be implemented in a separate wrapper)
export declare function generateUba(options: UbaOptions): JsBitcoinAddresses;
export declare function parseUba(uba: string): ParsedUba;

// Compatibility aliases for the old function names (deprecated)
/** @deprecated Use generate_addresses instead */
export declare const generate: typeof generate_addresses;
/** @deprecated Use parse_uba_string instead */
export declare const parseUbaString: typeof parse_uba_string;
/** @deprecated Use derive_encryption_key_from_passphrase instead */
export declare const deriveEncryptionKeyFromPassphrase: typeof derive_encryption_key_from_passphrase;
/** @deprecated Use generate_random_encryption_key instead */
export declare const generateRandomEncryptionKey: typeof generate_random_encryption_key;
/** @deprecated Use get_default_public_relays instead */
export declare const getDefaultPublicRelays: typeof get_default_public_relays;
/** @deprecated Use get_extended_public_relays instead */
export declare const getExtendedPublicRelays: typeof get_extended_public_relays; 