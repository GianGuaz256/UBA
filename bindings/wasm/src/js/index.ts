/**
 * High-level TypeScript wrapper for UBA (Unified Bitcoin Addresses) WebAssembly bindings
 * 
 * This module provides a more convenient and type-safe API for JavaScript/TypeScript
 * developers using the UBA library.
 */

import init, {
  generate_addresses as wasmGenerateAddresses,
  parse_uba_string as wasmParseUba,
  derive_encryption_key_from_passphrase as wasmDeriveKey,
  generate_random_encryption_key as wasmGenerateRandomKey,
  get_default_public_relays as wasmGetDefaultRelays,
  get_extended_public_relays as wasmGetExtendedRelays,
  JsUbaConfig,
  JsBitcoinAddresses,
  AddressTypes,
  Networks
} from '../pkg/uba_wasm';

// Re-export types
export {
  JsUbaConfig,
  JsBitcoinAddresses,
  AddressTypes,
  Networks
};

export type AddressType = 0 | 1 | 2 | 3 | 4 | 5;
export type Network = 0 | 1 | 2 | 3;

export interface ParsedUba {
  nostrId: string;
  label?: string;
}

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

export interface UbaResult {
  uba: string;
  nostrId: string;
  label?: string;
}

export interface AddressResult {
  addresses: string[];
  metadata: {
    createdAt: number;
    version: number;
    totalCount: number;
  };
}

export interface FullAddressResult extends AddressResult {
  addressesByType: {
    p2pkh?: string[];
    p2sh?: string[];
    p2wpkh?: string[];
    p2tr?: string[];
    lightning?: string[];
    liquid?: string[];
  };
}

let wasmInitialized = false;

/**
 * Initialize the WebAssembly module
 * Must be called before using any other functions
 */
export async function initUba(): Promise<void> {
  if (!wasmInitialized) {
    await init();
    wasmInitialized = true;
  }
}

/**
 * Ensure WASM is initialized before calling functions
 */
async function ensureInitialized(): Promise<void> {
  if (!wasmInitialized) {
    await initUba();
  }
}

/**
 * Create a configuration object from options
 */
function createConfig(options: Partial<UbaOptions & RetrieveOptions>): JsUbaConfig {
  const config = new JsUbaConfig();
  
  if (options.network !== undefined) {
    config.network = options.network;
  }
  
  if (options.encryptData !== undefined) {
    config.encrypt_data = options.encryptData;
  }
  
  if (options.encryptionKey) {
    config.setEncryptionKeyHex(options.encryptionKey);
  }
  
  if (options.relayTimeout !== undefined) {
    config.relay_timeout = options.relayTimeout;
  }
  
  if (options.addressCounts) {
    for (const [typeStr, count] of Object.entries(options.addressCounts)) {
      const type = parseInt(typeStr) as AddressType;
      if (count !== undefined) {
        config.setAddressCount(type, count);
      }
    }
  }
  
  return config;
}

/**
 * Generate addresses locally (WASM-only, no Nostr networking)
 * 
 * @param options Configuration options for address generation
 * @returns Promise resolving to address result
 */
export async function generateAddresses(options: UbaOptions): Promise<JsBitcoinAddresses> {
  await ensureInitialized();
  
  const config = createConfig(options);
  
  const addresses = wasmGenerateAddresses(
    options.seed,
    options.label || null,
    config
  );
  
  return addresses;
}

/**
 * Note: Full UBA generation and retrieval are not available in WASM builds
 * due to networking limitations. Use generateAddresses for local address generation.
 */
export async function generateUba(options: UbaOptions): Promise<UbaResult> {
  throw new Error("UBA generation with Nostr networking is not available in WASM builds. Use generateAddresses() for local address generation only.");
}

/**
 * Note: Address retrieval from Nostr relays is not available in WASM builds
 */
export async function retrieveAddresses(options: RetrieveOptions): Promise<AddressResult> {
  throw new Error("Address retrieval from Nostr relays is not available in WASM builds. Use this library on the server side for full UBA functionality.");
}

/**
 * Note: Full address retrieval from Nostr relays is not available in WASM builds
 */
export async function retrieveFullAddresses(options: RetrieveOptions): Promise<FullAddressResult> {
  throw new Error("Full address retrieval from Nostr relays is not available in WASM builds. Use this library on the server side for full UBA functionality.");
}

/**
 * Parse a UBA string to extract its components
 * 
 * @param uba UBA string to parse
 * @returns Parsed UBA components
 */
export function parseUbaString(uba: string): ParsedUba {
  // This function doesn't need WASM initialization as it's synchronous
  return wasmParseUba(uba);
}

/**
 * Derive an encryption key from a passphrase
 * 
 * @param passphrase The passphrase to derive from
 * @param salt Optional salt (default: none)
 * @returns Hex-encoded encryption key
 */
export function deriveEncryptionKey(passphrase: string, salt?: string): string {
  return wasmDeriveKey(passphrase, salt || null);
}

/**
 * Generate a random encryption key
 * 
 * @returns Hex-encoded random encryption key
 */
export function generateRandomEncryptionKey(): string {
  return wasmGenerateRandomKey();
}

/**
 * Get default public relay URLs
 * 
 * @returns Array of default public relay URLs
 */
export function getDefaultPublicRelays(): string[] {
  return wasmGetDefaultRelays();
}

/**
 * Get extended public relay URLs (includes more relays)
 * 
 * @returns Array of extended public relay URLs
 */
export function getExtendedPublicRelays(): string[] {
  return wasmGetExtendedRelays();
}

/**
 * Create addresses locally (simplified WASM-only version)
 * Note: This does not create a UBA with Nostr storage
 */
export async function createSimpleAddresses(
  seed: string,
  label?: string,
  network: Network = Networks.Bitcoin
): Promise<JsBitcoinAddresses> {
  await ensureInitialized();
  
  const config = new JsUbaConfig();
  config.network = network;
  
  return wasmGenerateAddresses(seed, label || null, config);
}

/**
 * Note: Getting addresses from UBA strings requires Nostr networking
 * which is not available in WASM builds
 */
export async function getAddressesFromUba(uba: string): Promise<string[]> {
  throw new Error("Getting addresses from UBA strings requires Nostr networking, which is not available in WASM builds. Use createSimpleAddresses() for local address generation.");
}

// Export constants for convenience
export const ADDRESS_TYPES = AddressTypes;
export const NETWORKS = Networks;

// Default export with the main API
export default {
  initUba,
  generateUba,
  retrieveAddresses,
  retrieveFullAddresses,
  parseUbaString,
  deriveEncryptionKey,
  generateRandomEncryptionKey,
  getDefaultPublicRelays,
  getExtendedPublicRelays,
  createSimpleAddresses,
  getAddressesFromUba,
  ADDRESS_TYPES,
  NETWORKS
}; 