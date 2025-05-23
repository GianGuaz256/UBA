//! Test relay connectivity and diagnose Nostr relay issues
//! 
//! This utility helps troubleshoot UBA connectivity by testing individual relays
//! and providing detailed connection information.

use uba::{NostrClient, default_public_relays, extended_public_relays};
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 UBA Relay Connectivity Test\n");

    // Test individual relays with different timeouts
    let timeouts = vec![5, 10, 20]; // seconds
    
    println!("📡 Testing Default Public Relays:");
    println!("=================================");
    
    let default_relays = default_public_relays();
    for (timeout_secs, desc) in timeouts.iter().zip(&["Quick", "Normal", "Patient"]) {
        println!("\n⏱️  {} Test ({}s timeout):", desc, timeout_secs);
        test_relay_batch(&default_relays, *timeout_secs).await;
    }

    // Test a subset of relays that are known to be reliable
    println!("\n\n🎯 Testing High-Reliability Relay Subset:");
    println!("=========================================");
    
    let reliable_relays = vec![
        "wss://relay.damus.io".to_string(),
        "wss://nos.lol".to_string(),
        "wss://relay.snort.social".to_string(),
        "wss://nostr.wine".to_string(),
        "wss://relay.primal.net".to_string(),
    ];
    
    test_relay_batch(&reliable_relays, 15).await;

    // Test extended relay list
    println!("\n\n📈 Testing Extended Relay List:");
    println!("===============================");
    
    let extended_relays = extended_public_relays();
    println!("Extended list has {} relays (vs {} default)", 
             extended_relays.len(), default_relays.len());
    
    // Test first 10 from extended list
    let extended_subset: Vec<String> = extended_relays.into_iter().take(10).collect();
    test_relay_batch(&extended_subset, 10).await;

    // Provide recommendations
    println!("\n\n💡 RECOMMENDATIONS:");
    println!("===================");
    println!("1. Use working relays from the test above");
    println!("2. Increase timeout if you have slow internet (config.relay_timeout = 20)");
    println!("3. Use UbaConfig::set_custom_relays() with only working relays");
    println!("4. Consider running examples offline with AddressGenerator for testing");
    
    println!("\n📝 Example custom configuration:");
    println!(r#"
    let mut config = UbaConfig::default();
    config.relay_timeout = 20; // Increase timeout
    config.set_custom_relays(vec![
        "wss://relay.damus.io".to_string(),  // Usually reliable
        "wss://nos.lol".to_string(),         // Good uptime
        "wss://relay.snort.social".to_string(), // Fast
    ]);
    "#);

    Ok(())
}

async fn test_relay_batch(relay_urls: &[String], timeout_secs: u64) {
    let mut successful = 0;
    let mut failed = 0;
    
    for (i, relay_url) in relay_urls.iter().enumerate() {
        let start_time = Instant::now();
        
        match test_single_relay(relay_url, timeout_secs).await {
            Ok(()) => {
                let duration = start_time.elapsed();
                println!("   ✅ [{}] {} ({}ms)", i + 1, relay_url, duration.as_millis());
                successful += 1;
            }
            Err(e) => {
                let duration = start_time.elapsed();
                println!("   ❌ [{}] {} - {} ({}ms)", i + 1, relay_url, e, duration.as_millis());
                failed += 1;
            }
        }
    }
    
    println!("\n📊 Results: {} successful, {} failed ({:.1}% success rate)", 
             successful, failed, 
             (successful as f64 / (successful + failed) as f64) * 100.0);
}

async fn test_single_relay(relay_url: &str, timeout_secs: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Create a timeout wrapper around the connection test
    let test_future = async {
        let client = NostrClient::new(timeout_secs)?;
        client.connect_to_relays(&[relay_url.to_string()]).await?;
        client.disconnect().await;
        Ok::<(), Box<dyn std::error::Error>>(())
    };
    
    // Apply timeout to the entire test
    timeout(Duration::from_secs(timeout_secs + 2), test_future).await??;
    
    Ok(())
} 