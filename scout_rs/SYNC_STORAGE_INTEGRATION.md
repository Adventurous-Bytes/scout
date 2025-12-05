## Goal

We are uploading our files to the Supabase storage artifacts bucket. 

## File Format
Files are stored in the `artifacts` bucket with object paths: `{herd_id}/{device_id}/file_name.extension`.

**Important**: The bucket name (`artifacts`) is specified in the API URL and StorageConfig, but the actual object path stored in the database is just `{herd_id}/{device_id}/file_name.extension` without the bucket prefix.

## Optimizations
- uses the dedicated storage subdomain to expedite uploads
- uses TUS for resumable uploads 

## Notes

This routine will likely be used by programs with multiple concurrent processes - many of which may need to update the scout engine. Please ensure that we do not lock the sync engine for en extended period of time.

## Example Usage

1. Create sync engine with storage

```rust
// Create storage config with device API key for access control
let storage_config = StorageConfig {
    supabase_url: "https://your-project.supabase.co".to_string(),
    supabase_anon_key: "your-anon-key".to_string(),
    scout_api_key: "your-device-api-key".to_string(), // Required for RLS policies
    bucket_name: "artifacts".to_string(),
    allowed_extensions: vec![".mp4".to_string()],
};

// Alternative: Use convenience method with device API key
let storage_config = StorageClient::with_device_api_key(
    "https://your-project.supabase.co".to_string(),
    "your-anon-key".to_string(),
    "your-device-api-key".to_string(),
    "artifacts".to_string(),
    vec![".mp4".to_string()],
)?;

let mut sync_engine = SyncEngine::new(scout_client, "db.path".to_string(), None, None, false)?
    .with_storage(storage_config)?;
```

 2. Query and manage artifacts
 
```rust
let ready_artifacts = sync_engine.get_artifacts_ready_for_upload()?;
let mut need_urls = sync_engine.get_artifacts_needing_upload_urls()?;
sync_engine.generate_upload_urls(&mut need_urls).await?;
```


3. Upload with progress monitoring

```rust
for artifact in ready_artifacts {
    let (upload_handle, mut progress_rx) = sync_engine
        .spawn_upload_artifact(artifact.clone(), Some(512 * 1024))?; // 512KB chunks

    // Monitor progress
    tokio::spawn(async move {
        while let Ok(progress) = progress_rx.recv().await {
            let percent = (progress.bytes_uploaded as f64 / progress.total_bytes as f64) * 100.0;
            println!("Progress: {:.1}% for {}", percent, progress.file_name);
        }
    });

    // Handle completion/cancellation
    match upload_handle.await {
        Ok(Ok((updated_artifact, storage_path))) => {
            sync_engine.upsert_items(vec![updated_artifact])?; // Update database
        }
        Ok(Err(e)) => println!("Upload failed: {}", e),
        Err(_) => println!("Upload cancelled"), // upload_handle.abort() was called
    }
}
```

### RLS Policy Enforcement
The Row Level Security policies work with the object path format shown above (`{herd_id}/{device_id}/filename`) and enforce:
- Device API keys can only access paths where `device_id` matches their device and where device belongs to `herd_id`
- Users with view role can access all paths within their permitted `herd_id`
- **Only artifacts with** `has_uploaded_file_to_storage = true` are synced to the database
