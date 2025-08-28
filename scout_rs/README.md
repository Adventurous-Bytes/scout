# Scout RS

A Rust client for the Scout API that allows uploading events and images to the Scout database.

## 🚀 Quick Start

```rust
use scout_rs::client::ScoutClient;

// Create the client
let mut client = ScoutClient::new(
    "https://api.example.com/api/scout".to_string(), // Now optional
    "your_api_key_here".to_string()
)?;

// Identify device/herd and establish database connection directly from database
client.identify().await?;

// Now perform operations directly on the database via PostgREST
let event = Event::new(/* ... */);
let created_event = client.create_event(&event).await?;

let tags = vec![Tag::new(/* ... */)];
let created_tags = client.create_tags(created_event.id.unwrap(), &tags).await?;
```

**Environment Variables Required:**

```bash
SCOUT_DATABASE_REST_URL=https://your-db.supabase.co/rest/v1
SCOUT_DEVICE_API_KEY=your_device_api_key_here
```

**Quick Setup:**

1. Copy `env.example` to `.env`
2. Fill in your actual values
3. Run your application

## 🎯 **What's New**

The Scout client has been **completely refactored** to use **direct database access** via PostgREST instead of HTTP API endpoints. This eliminates the need for "careful lockstep development" and provides:

- **🚀 Better Performance**: Direct database operations instead of HTTP round-trips
- **🔒 Secure Authentication**: API key-based authentication with PostgREST
- **📊 Full Database Access**: Complete CRUD operations on all Scout entities
- **🔄 Simplified Architecture**: Single client with comprehensive functionality
- **✅ Backward Compatibility**: Existing code continues to work without changes

## 📚 Documentation & Migration

- **[Migration Guide](MIGRATION_GUIDE.md)**: Complete guide for migrating from the old HTTP API client
- **[Environment Setup](env.example)**: Example configuration file
- **[Examples](examples/)**: Working examples of all client operations

## 🔧 Features

- **Direct Database Access**: All operations go directly to PostgreSQL via PostgREST
- **Comprehensive CRUD**: Full support for events, sessions, connectivity, tags, and more
- **Batch Operations**: Efficient batch creation and updates
- **Geographic Queries**: Location-based event filtering
- **Session Management**: Complete session lifecycle management
- **Device & Herd Management**: Direct access to device and herd information

## 📦 Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
scout_rs = "0.4.1"
```

## 🌟 Key Benefits

1. **No More API Lockstep**: Direct database access eliminates API version dependencies
2. **Better Performance**: Reduced latency and improved throughput
3. **Full Control**: Access to all database features and capabilities
4. **Simplified Testing**: Easier to set up test environments
5. **Future-Proof**: Database schema changes don't require client updates

## ✅ **Backward Compatibility**

**Existing code continues to work without any changes!** The client maintains the same public API:

- **Same return types**: All methods still return `ResponseScout<T>` with `status` and `data` fields
- **Same status handling**: Use `ResponseScoutStatus::Success`, `ResponseScoutStatus::Failure`, etc.
- **Same data access**: Access data via the `.data` field as before
- **No breaking changes**: Drop-in replacement for the old HTTP API client

### Migration Example

```rust
// Your existing code works exactly the same
let response = client.get_device().await?;
if response.status == ResponseScoutStatus::Success {
    if let Some(device) = response.data {
        println!("Device: {}", device.name);
    }
}
```

**The only difference**: Operations now go directly to the database instead of HTTP API endpoints, providing better performance and eliminating API dependencies.

## 🔐 Security

- **API Key Authentication**: Secure device identification via stored API keys
- **PostgREST Gateway**: Database access through secure REST gateway
- **Environment Variables**: Sensitive configuration kept out of code

## 📖 Examples

See the [examples directory](examples/) for comprehensive usage examples including:

- [Basic Usage](examples/basic_usage.rs) - Complete client setup and operations
- Event and tag creation
- Session management
- Connectivity tracking
- Batch operations
