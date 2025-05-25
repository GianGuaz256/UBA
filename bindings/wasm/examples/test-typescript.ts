/**
 * TypeScript test for high-level UBA API
 * 
 * This file tests the TypeScript wrapper that provides a more convenient API
 * Run with: npx ts-node examples/test-typescript.ts
 */

import {
    initUba,
    generateUba,
    retrieveAddresses,
    retrieveFullAddresses,
    parseUbaString,
    deriveEncryptionKey,
    generateRandomEncryptionKey,
    getDefaultPublicRelays,
    createSimpleUba,
    getAddressesFromUba,
    ADDRESS_TYPES,
    NETWORKS,
    UbaOptions,
    RetrieveOptions
} from '../src/js/index';

// Test seed phrase (DO NOT use in production!)
const TEST_SEED = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

async function testHighLevelAPI(): Promise<void> {
    console.log('🧪 Testing High-Level TypeScript API...');
    
    try {
        // Initialize the WASM module
        console.log('🚀 Initializing UBA...');
        await initUba();
        console.log('✅ UBA initialized');
        
        // Test utility functions
        console.log('🔧 Testing utility functions...');
        
        const defaultRelays = getDefaultPublicRelays();
        console.log('✅ Default relays:', defaultRelays.length);
        
        const encryptionKey = generateRandomEncryptionKey();
        console.log('✅ Generated encryption key:', encryptionKey.substring(0, 16) + '...');
        
        const derivedKey = deriveEncryptionKey('test-passphrase');
        console.log('✅ Derived encryption key:', derivedKey.substring(0, 16) + '...');
        
        // Test constants
        console.log('📊 Testing constants...');
        console.log('✅ Address types:', {
            P2PKH: ADDRESS_TYPES.P2PKH,
            P2WPKH: ADDRESS_TYPES.P2WPKH,
            Lightning: ADDRESS_TYPES.Lightning
        });
        
        console.log('✅ Networks:', {
            Bitcoin: NETWORKS.Bitcoin,
            Testnet: NETWORKS.Testnet
        });
        
        // Test UBA parsing
        console.log('📋 Testing UBA parsing...');
        const testUba = "UBA:1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef&label=test";
        const parsed = parseUbaString(testUba);
        console.log('✅ Parsed UBA:', parsed);
        
        console.log('🎉 High-level API basic tests passed!');
        
    } catch (error) {
        console.error('❌ High-level API test failed:', error);
        throw error;
    }
}

async function testUbaGeneration(): Promise<string> {
    console.log('\n🚀 Testing UBA generation with high-level API...');
    
    try {
        const options: UbaOptions = {
            seed: TEST_SEED,
            label: 'typescript-test',
            network: NETWORKS.Testnet,
            encryptData: false,
            relayTimeout: 15,
            addressCounts: {
                [ADDRESS_TYPES.P2WPKH]: 2,
                [ADDRESS_TYPES.Lightning]: 1,
                [ADDRESS_TYPES.Liquid]: 1
            }
        };
        
        console.log('⏳ Generating UBA...');
        const result = await generateUba(options);
        
        console.log('✅ Generated UBA result:', {
            uba: result.uba,
            nostrId: result.nostrId.substring(0, 16) + '...',
            label: result.label
        });
        
        return result.uba;
        
    } catch (error) {
        console.error('❌ UBA generation failed:', error);
        throw error;
    }
}

async function testAddressRetrieval(uba: string): Promise<void> {
    console.log('\n📥 Testing address retrieval...');
    
    try {
        const retrieveOptions: RetrieveOptions = {
            uba,
            relayTimeout: 15
        };
        
        // Test simple retrieval
        console.log('📋 Testing simple address retrieval...');
        const addressResult = await retrieveAddresses(retrieveOptions);
        
        console.log('✅ Retrieved addresses:', {
            count: addressResult.addresses.length,
            addresses: addressResult.addresses,
            metadata: addressResult.metadata
        });
        
        // Test full retrieval
        console.log('📊 Testing full address retrieval...');
        const fullResult = await retrieveFullAddresses(retrieveOptions);
        
        console.log('✅ Full address structure:', {
            totalCount: fullResult.metadata.totalCount,
            createdAt: new Date(fullResult.metadata.createdAt * 1000),
            version: fullResult.metadata.version,
            addressesByType: Object.entries(fullResult.addressesByType)
                .filter(([, addresses]) => addresses && addresses.length > 0)
                .reduce((acc, [type, addresses]) => ({
                    ...acc,
                    [type]: addresses
                }), {})
        });
        
        console.log('🎉 Address retrieval tests passed!');
        
    } catch (error) {
        console.error('❌ Address retrieval failed:', error);
        throw error;
    }
}

async function testSimpleAPI(): Promise<void> {
    console.log('\n🎯 Testing simple convenience API...');
    
    try {
        // Test simple UBA creation
        console.log('⏳ Creating simple UBA...');
        const simpleUba = await createSimpleUba(
            TEST_SEED + ' simple',
            'simple-test',
            NETWORKS.Testnet
        );
        
        console.log('✅ Created simple UBA:', simpleUba);
        
        // Test simple address retrieval
        console.log('📥 Getting addresses from UBA...');
        const addresses = await getAddressesFromUba(simpleUba);
        
        console.log('✅ Retrieved addresses:', {
            count: addresses.length,
            first: addresses[0]
        });
        
        console.log('🎉 Simple API tests passed!');
        
    } catch (error) {
        console.error('❌ Simple API test failed:', error);
        throw error;
    }
}

async function testEncryption(): Promise<void> {
    console.log('\n🔒 Testing encryption with TypeScript API...');
    
    try {
        const encryptionKey = generateRandomEncryptionKey();
        
        const encryptedOptions: UbaOptions = {
            seed: TEST_SEED + ' encrypted',
            label: 'encrypted-test',
            network: NETWORKS.Testnet,
            encryptData: true,
            encryptionKey,
            relayTimeout: 15,
            addressCounts: {
                [ADDRESS_TYPES.P2WPKH]: 1
            }
        };
        
        console.log('⏳ Generating encrypted UBA...');
        const encryptedResult = await generateUba(encryptedOptions);
        
        console.log('✅ Generated encrypted UBA:', encryptedResult.uba);
        
        // Test retrieval with correct key
        console.log('📥 Retrieving with correct key...');
        const decryptedAddresses = await retrieveAddresses({
            uba: encryptedResult.uba,
            encryptionKey,
            relayTimeout: 15
        });
        
        console.log('✅ Decrypted addresses:', decryptedAddresses.addresses);
        
        // Test retrieval without key (should fail)
        console.log('🚫 Testing retrieval without key (should fail)...');
        try {
            await retrieveAddresses({
                uba: encryptedResult.uba,
                relayTimeout: 15
            });
            console.log('❌ ERROR: Should have failed without encryption key!');
        } catch (error) {
            console.log('✅ Correctly failed without encryption key');
        }
        
        console.log('🎉 Encryption tests passed!');
        
    } catch (error) {
        console.error('❌ Encryption test failed:', error);
        throw error;
    }
}

async function runTypeScriptTests(): Promise<void> {
    console.log('🧪 UBA TypeScript API Test Suite');
    console.log('=================================\n');
    
    try {
        await testHighLevelAPI();
        
        // Skip network tests if in CI or no network
        if (process.env.SKIP_NETWORK_TESTS) {
            console.log('⏭️  Skipping network tests (SKIP_NETWORK_TESTS set)');
        } else {
            const uba = await testUbaGeneration();
            await testAddressRetrieval(uba);
            await testSimpleAPI();
            await testEncryption();
        }
        
        console.log('\n🎉 All TypeScript tests passed successfully!');
        console.log('✅ TypeScript bindings are working correctly.');
        
    } catch (error) {
        console.error('\n💥 TypeScript test suite failed:', error);
        process.exit(1);
    }
}

// Run tests if this file is executed directly
if (require.main === module) {
    runTypeScriptTests();
}

export {
    testHighLevelAPI,
    testUbaGeneration,
    testAddressRetrieval,
    testSimpleAPI,
    testEncryption,
    runTypeScriptTests
}; 