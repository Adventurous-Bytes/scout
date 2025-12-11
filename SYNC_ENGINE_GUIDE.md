# Scout Sync Engine Guide

The Scout Sync Engine is a robust synchronization system that manages bidirectional data flow between local SQLite storage and the remote Scout server. It handles hierarchical data relationships, batch operations, and provides resilient error handling.

## Overview

The sync engine maintains a strict hierarchical sync order to preserve data integrity:

1. **Sessions** (parent entities)
2. **Connectivity** entries (children of sessions)
3. **Events** (children of sessions)
4. **Tags** (children of events)

This order ensures that parent entities are synced before their children, preventing foreign key constraint violations.

## Key Features

- **Batch Operations**: Efficiently syncs multiple items in single requests
- **Automatic ID Management**: Handles local-to-remote ID mapping and relationship updates
- **Partial Failure Recovery**: Continues syncing other entities even if one type fails
- **Auto-cleaning**: Removes completed sessions from local storage after successful sync
- **Configurable Batching**: Control sync frequency and batch sizes
- **Background Operation**: Can run continuously in background tasks

## Basic Usage

### Creating a Sync Engine

```rust
use scout_rs::{SyncEngine, ScoutClient};

// With default settings (recommended for most use cases)
let client = ScoutClient::new("your-api-key", "https://api.scout.com")?;
let sync_engine = SyncEngine::with_defaults(client, "/path/to/local.db")?;

// With failed record removal enabled (removes records with critical errors)
let sync_engine = SyncEngine::with_failed_record_removal(client, "/path/to/local.db")?;

// With custom configuration
let sync_engine = SyncEngine::new(
    client,
    "/path/to/local.db".to_string(),
    Some(5000), // Sync every 5 seconds
    Some(50),   // Max 50 items per batch
    true,       // Enable auto-clean
    false       // Disable failed record removal (safer default)
)?;
```

### Manual Sync Operations

```rust
// One-time sync of all pending data
sync_engine.flush().await?;

// Sync and clean in one operation
sync_engine.flush_and_clean().await?;

// Just run the cleaning operation
sync_engine.clean().await?;
```

### Background Sync

```rust
// Start sync engine in current task (blocks until stopped)
sync_engine.start().await?;

// Spawn in background task
let sync_handle = sync_engine.spawn_background_sync();

// Later, cancel the background sync
sync_handle.abort();
```

## Configuration Options

### Sync Intervals

- `None`: Manual sync only - call `flush()` when needed
- `Some(milliseconds)`: Automatic sync at specified intervals

```rust
// Manual sync only
let sync_engine = SyncEngine::new(client, db_path, None, None, false)?;

// Sync every 30 seconds
let sync_engine = SyncEngine::new(client, db_path, Some(30_000), None, true)?;
```

### Batch Sizes

Control how many items are synced in each batch to balance performance and memory usage:

```rust
// No limit (sync all pending items)
let sync_engine = SyncEngine::new(client, db_path, Some(3000), None, true)?;

// Limit to 25 items per entity type per sync
let sync_engine = SyncEngine::new(client, db_path, Some(3000), Some(25), true)?;
```

### Auto-cleaning

Auto-cleaning removes completed sessions and their descendants from local storage after successful sync:

- **Enabled**: Automatically cleans after each successful flush
- **Disabled**: Manual cleaning via `clean()` method

Safety features:
- Only cleans sessions that ended more than 30 seconds ago
- Verifies all descendants have remote IDs before cleaning
- Logs detailed information about cleaned items

### Failed Record Removal

Failed record removal automatically removes records from local storage when they fail with critical errors that indicate the operation will never succeed:

- **Critical Errors**:
  - `"parse error - invalid geometry"` - Invalid geographic data
  - `"new row violates row-level security policy"` - Permission denied
  - `"All object keys must match"` - Data structure mismatch

- **Safety**: Disabled by default to prevent data loss
- **Logging**: Records detailed information about removed items
- **Fallback**: Individual processing continues for other records

## Data Flow

### Local Data Creation

```rust
// Create local entities with unique local IDs
let session = SessionLocal {
    id: None,                    // No remote ID yet
    id_local: Some("session_123".to_string()), // Unique local ID
    timestamp_start: now(),
    timestamp_end: None,
    // ... other fields
};

// Store in local database
sync_engine.upsert_items(vec![session])?;
```

### Sync Process

The sync engine uses different strategies for different entity types:

**Sessions**: Always upserted because they can be updated (e.g., when `timestamp_end` is set)
**Other Entities** (Connectivity, Events, Tags): Only new items (without remote IDs) are synced

1. **Preparation**: Engine collects items by type with different strategies:
   - Sessions: All items (with and without remote IDs) for upserting
   - Others: Only items without remote IDs for insertion
2. **Batching**: Items are grouped and limited by `max_num_items_per_sync`
3. **Remote Sync**: Batch upsert operations send data to server
4. **Local Update**: Returned data updates local storage with remote IDs
5. **Relationship Updates**: Child entities get updated with parent remote IDs
6. **Cleaning**: Completed sessions are removed (if auto-clean enabled)

### Error Handling

The sync engine uses a "continue on error" approach:

- If one entity type fails, others continue syncing
- Detailed error logging for troubleshooting
- Partial failures are reported but don't stop the entire sync
- Background sync continues running despite individual failures
- Critical errors can optionally remove problematic records (when `remove_failed_records` is enabled)

### Sync Behavior by Entity Type

**Sessions**:
- Always synced (upserted) regardless of whether they have remote IDs
- Can be updated multiple times (e.g., setting `timestamp_end`)
- Forms the foundation for all child entity relationships

**Connectivity, Events, Tags**:
- Only synced if they don't have remote IDs yet (insert-only)
- Items with remote IDs are skipped (considered already synced)
- More efficient since these entities typically don't change after creation

## Advanced Usage

### Utility Methods

```rust
// Get local database path
let db_path = sync_engine.get_db_path();

// Generate unique local IDs
let unique_id = sync_engine.generate_unique_id::<SessionLocal>()?;

// Get table counts
let session_count = sync_engine.get_table_count::<SessionLocal>()?;

// Manual item operations
sync_engine.upsert_items(items)?;
sync_engine.remove_items(items)?;
```

### Failed Record Removal Example

```rust
// Create sync engine with failed record removal enabled
let sync_engine = SyncEngine::with_failed_record_removal(client, db_path)?;

// Records with critical errors will be automatically removed
sync_engine.flush().await?;

// Check logs for removed records:
// WARN: Critical error detected for session "session_123", removing from local storage: parse error - invalid geometry
// INFO: Removed 1 sessions with critical errors from local storage
```

### Monitoring and Debugging

The sync engine provides extensive logging:

```rust
// Enable debug logging to see detailed sync information
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

Log levels:
- **INFO**: Sync start/stop, batch sizes, cleaning operations
- **DEBUG**: Individual entity processing, descendant updates
- **ERROR**: Failures with detailed context
- **WARN**: Non-critical issues and configuration warnings

## Best Practices

### 1. Use Default Configuration

Start with `with_defaults()` unless you have specific requirements:

```rust
let sync_engine = SyncEngine::with_defaults(client, db_path)?;
```

For environments with data quality issues, consider enabling failed record removal:

```rust
let sync_engine = SyncEngine::with_failed_record_removal(client, db_path)?;
```

### 2. Handle Background Tasks Properly

```rust
use tokio::select;

let sync_handle = sync_engine.spawn_background_sync();

// Graceful shutdown
select! {
    _ = tokio::signal::ctrl_c() => {
        println!("Shutting down...");
        sync_handle.abort();
    }
    result = sync_handle => {
        match result {
            Ok(Ok(())) => println!("Sync completed successfully"),
            Ok(Err(e)) => eprintln!("Sync error: {}", e),
            Err(e) => eprintln!("Task error: {}", e),
        }
    }
}
```

### 3. Monitor Sync Performance

```rust
// Check for pending items before sync
let session_count = sync_engine.get_table_count::<SessionLocal>()?;
println!("Syncing {} sessions", session_count);

// Time sync operations
let start = std::time::Instant::now();
sync_engine.flush().await?;
println!("Sync took {:?}", start.elapsed());
```

### 4. Handle Network Issues

```rust
// Implement retry logic for critical operations
for attempt in 1..=3 {
    match sync_engine.flush().await {
        Ok(()) => {
            println!("Sync successful on attempt {}", attempt);
            break;
        }
        Err(e) if attempt < 3 => {
            eprintln!("Sync attempt {} failed: {}. Retrying...", attempt, e);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Err(e) => {
            eprintln!("Sync failed after {} attempts: {}", attempt, e);
            return Err(e);
        }
    }
}
```

### 5. Local ID Management

```rust
// Always use unique local IDs for trackability
let local_id = format!("session_{}", sync_engine.generate_unique_id::<SessionLocal>()?);

// Store relationships using local IDs initially
let event = EventLocal {
    id: None,
    id_local: Some(format!("event_{}", unique_id)),
    ancestor_id_local: Some(session_local_id.clone()), // Reference parent's local ID
    session_id: None, // Will be filled when parent gets remote ID
    // ...
};
```

## Troubleshooting

### Common Issues

1. **Foreign Key Constraint Errors**
   - Ensure parent entities are synced before children
   - Check that local ID relationships are properly set

2. **Memory Usage**
   - Reduce `max_num_items_per_sync` for large datasets
   - Enable auto-cleaning to prevent local storage growth

3. **Sync Delays**
   - Increase sync interval for less frequent but larger batches
   - Check network connectivity and server response times

4. **Incomplete Cleaning**
   - Verify all descendants have remote IDs before sessions are cleaned
   - Check that sessions have `timestamp_end` set

5. **Critical Sync Errors**
   - Enable `remove_failed_records` for automatic cleanup of problematic records
   - Monitor logs for removed records to identify data quality issues
   - Consider data validation before sync to prevent critical errors

### Debug Commands

```rust
// Check sync engine status
println!("DB Path: {}", sync_engine.get_db_path());
println!("Sessions: {}", sync_engine.get_table_count::<SessionLocal>()?);
println!("Events: {}", sync_engine.get_table_count::<EventLocal>()?);

// Force a single sync operation
sync_engine.flush_once().await?;
```

## Examples

### Complete Example: Session with Events and Tags

```rust
use scout_rs::*;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize sync engine
    let client = ScoutClient::new("your-api-key", "https://api.scout.com")?;
    let mut sync_engine = SyncEngine::with_defaults(client, "/tmp/scout.db")?;

    // Create a session
    let session = SessionLocal {
        id: None,
        id_local: Some("session_001".to_string()),
        timestamp_start: chrono::Utc::now().timestamp(),
        timestamp_end: None,
        // ... other fields
    };
    sync_engine.upsert_items(vec![session])?;

    // Create an event for this session
    let event = EventLocal {
        id: None,
        id_local: Some("event_001".to_string()),
        ancestor_id_local: Some("session_001".to_string()),
        session_id: None, // Will be updated after session sync
        // ... other fields
    };
    sync_engine.upsert_items(vec![event])?;

    // Create tags for this event
    let tag = TagLocal {
        id: None,
        id_local: Some("tag_001".to_string()),
        ancestor_id_local: Some("event_001".to_string()),
        event_id: 0, // Will be updated after event sync
        // ... other fields
    };
    sync_engine.upsert_items(vec![tag])?;

    // Sync everything to server
    sync_engine.flush_and_clean().await?;

    println!("Sync completed successfully!");
    Ok(())
}
```

## Troubleshooting

If sync tests are failing, check:

1. **Environment Variables**: Ensure `SCOUT_DEVICE_API_KEY` and `SCOUT_DATABASE_REST_URL` are set
2. **API Key Validity**: Verify the API key is active and has proper permissions
3. **Database Permissions**: Check row-level security (RLS) policies allow insertions
4. **Network Connectivity**: Ensure access to the Scout database URL
5. **Device Registration**: Confirm the API key corresponds to a valid registered device

This guide covers the essential aspects of using the Scout Sync Engine. For additional questions or advanced use cases, refer to the source code documentation or reach out to the Scout development team.
