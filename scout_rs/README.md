# Scout Rust Client

A Rust client library for the Scout database system with direct database access via PostgREST.

## Quick Start

```rust
use scout_rs::client::ScoutClient;
use scout_rs::db_client::DatabaseConfig;
use scout_rs::models::data::Connectivity;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize client with database configuration from environment
    let config = DatabaseConfig::from_env()?;
    let mut client = ScoutClient::new(config);
    
    // Identify device and establish connection
    client.identify().await?;
    
    // Get peer devices in the same herd
    let peers = client.get_peer_devices().await?;
    println!("Found {} peer devices", peers.data.unwrap().len());
    
    // Create and push a connectivity event
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    
    let connectivity = Connectivity::new(
        None,                                   // session_id (optional)
        Some(client.device.as_ref().unwrap().id.unwrap()), // device_id
        timestamp,                              // timestamp_start (unix epoch)
        -65.0,                                  // signal strength
        -95.0,                                  // noise floor
        100.0,                                  // altitude
        45.0,                                   // heading
        "POINT(-122.4194 37.7749)".to_string(), // location (SF)
        "8928308280fffff".to_string(),          // h14_index
        "8828308280fffff".to_string(),          // h13_index
        "8728308280fffff".to_string(),          // h12_index
        "8628308280fffff".to_string(),          // h11_index
        Some(85.5),                             // battery_percentage
    );
    
    let result = client.create_connectivity(&connectivity).await?;
    println!("Connectivity event created with ID: {}", result.data.unwrap().id.unwrap());
    
    Ok(())
}
```

## Environment Setup

Create a `.env` file with your database configuration:

```bash
SCOUT_DATABASE_REST_URL=https://your-database.supabase.co/rest/v1
SCOUT_DEVICE_API_KEY=your_device_api_key_here
SUPABASE_PUBLIC_API_KEY=your_supabase_public_key_here
```

## Features

- **Direct Database Access**: Operations via PostgREST for better performance
- **Device Management**: Automatic device identification and peer discovery
- **Real-time Data**: Push connectivity, events, and sensor data
- **Row Level Security**: Automatic data isolation by herd membership
- **Batch Operations**: Efficient bulk data operations
- **Local Sync**: Optional local database synchronization

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
scout_rs = "0.6"
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
```
