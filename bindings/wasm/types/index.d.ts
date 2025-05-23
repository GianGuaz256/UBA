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

// Main functions
export declare function generate(
  seed: string,
  label?: string,
  relayUrls?: string[],
  config?: JsUbaConfig
): Promise<string>;

export declare function retrieve(
  uba: string,
  relayUrls?: string[],
  config?: JsUbaConfig
): Promise<string[]>;

export declare function retrieveFull(
  uba: string,
  relayUrls?: string[],
  config?: JsUbaConfig
): Promise<JsBitcoinAddresses>;

export declare function parseUbaString(uba: string): ParsedUba;

// Utility functions
export declare function deriveEncryptionKeyFromPassphrase(
  passphrase: string,
  salt?: string
): string;

export declare function generateRandomEncryptionKey(): string;

export declare function getDefaultPublicRelays(): string[];

export declare function getExtendedPublicRelays(): string[];

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
  relayUrls?: string[];
  network?: Network;
  encryptData?: boolean;
  encryptionKey?: string;
  addressCounts?: Partial<Record<AddressType, number>>;
  relayTimeout?: number;
}

export interface RetrieveOptions {
  uba: string;
  relayUrls?: string[];
  encryptionKey?: string;
  relayTimeout?: number;
}

// High-level API functions
export declare function generateUba(options: UbaOptions): Promise<string>;
export declare function retrieveAddresses(options: RetrieveOptions): Promise<string[]>;
export declare function retrieveFullAddresses(options: RetrieveOptions): Promise<JsBitcoinAddresses>; 