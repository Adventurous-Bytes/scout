# Storage Access Control Implementation

This document describes the implementation of Row Level Security (RLS) policies for Supabase Storage in the Scout project, ensuring that artifacts are properly protected based on herd membership and device ownership.

## Overview

The storage system implements a hierarchical access control model where:
- Files are stored in the `artifacts` bucket with object paths: `{herd_id}/{device_id}/filename.ext`
- **Important**: The bucket name (`artifacts`) is separate from the object path stored in the database
- Users with view role for a herd can view all artifacts in that herd
- Device API keys can read/write only to their own device folder within their herd
- Users with edit role can delete artifacts in their herd

## File Structure

### Database Policies
- `scout/database/storage_policies.sql` - Main RLS policies for storage.objects table
- `scout/database/test_storage_policies.sql` - Test queries to verify policies work

### Code Changes
- `scout/scout_rs/src/storage.rs` - Updated to inject API key headers
- `scout/scout_rs/SYNC_STORAGE_INTEGRATION.md` - Updated documentation

## SQL Policies Implemented

### 1. Helper Functions

```sql
-- Extract herd_id from storage object path (format: {herd_id}/{device_id}/filename)
private.get_herd_id_from_path(object_name text) RETURNS bigint

-- Extract device_id from storage object path (format: {herd_id}/{device_id}/filename)
private.get_device_id_from_path(object_name text) RETURNS bigint
```

### 2. Access Policies

#### View Access
- **Policy**: "Artifact view access: Users with view role for herd"
- **Allows**: 
  - Users with view role for the herd to view all objects in that herd
  - Device API keys to view their own artifacts if device belongs to herd

#### Upload Access
- **Policy**: "Artifact upload access: Device API keys to own folder"
- **Allows**: Device API keys to upload only to their own device folder (object path: `{herd_id}/{device_id}/`)

#### Update Access
- **Policy**: "Artifact update access: Device API keys to own files"
- **Allows**: Device API keys to overwrite their own artifacts

#### Delete Access
- **Policy**: "Artifact delete access: Device API keys or users with edit role"
- **Allows**:
  - Device API keys to delete their own artifacts
  - Users with edit role for the herd to delete any artifact in that herd

## Code Implementation

### StorageConfig Changes

```rust
pub struct StorageConfig {
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub scout_api_key: String, // Required: Device API key for RLS
    pub bucket_name: String,
    pub allowed_extensions: Vec<String>,
}
```

### Header Injection

The storage client now injects the same headers as the database client:

```rust
// Authorization header with Bearer token
headers.insert("Authorization", format!("Bearer {}", supabase_anon_key));

// apikey header (required by Supabase)
headers.insert("apikey", supabase_anon_key);

// api_key header (device API key for RLS)
headers.insert("api_key", config.scout_api_key);
```

## Usage Examples

### Basic Setup with Device API Key

```rust
use scout_rs::storage::{StorageConfig, StorageClient};

// Create config with device API key
let storage_config = StorageConfig {
    supabase_url: "https://your-project.supabase.co".to_string(),
    supabase_anon_key: "your-supabase-anon-key".to_string(),
    scout_api_key: "your-device-api-key".to_string(),
    bucket_name: "artifacts".to_string(),
    allowed_extensions: vec![".mp4".to_string(), ".jpg".to_string()],
};

let storage_client = StorageClient::new(storage_config)?;
```

### Alternative Constructor

```rust
let storage_client = StorageClient::with_device_api_key(
    "https://your-project.supabase.co".to_string(),
    "your-supabase-anon-key".to_string(),
    "your-device-api-key".to_string(),
    "artifacts".to_string(),
    vec![".mp4".to_string()],
)?;
```

## Security Model

### Device API Keys
- Generated using HMAC with device-specific secrets
- Validated through `private.key_uid()` function
- Only allow access to artifacts belonging to the same device
- Must verify device belongs to the herd being accessed

### User Authentication
- Uses Supabase Auth JWT tokens
- Roles managed through `users_roles_per_herd` table
- Role hierarchy: viewer < editor < admin
- Permissions checked through helper functions like `private.has_good_view_role()`

### Path Validation
- **Bucket**: `artifacts` (specified in API calls and StorageConfig)
- **Object paths**: Must follow exact format: `{herd_id}/{device_id}/filename`
- **Database storage**: Object paths in `storage.objects.name` do NOT include bucket name
- Invalid paths return NULL from helper functions, denying access
- Path components are validated as bigint for herd_id and device_id

## Deployment Instructions

### 1. Apply Storage Policies

```sql
-- Run this in your Supabase SQL editor
\i scout/database/storage_policies.sql
```

### 2. Verify Policies

```sql
-- Run test queries to verify everything works
\i scout/database/test_storage_policies.sql
```

### 3. Update Application Code

Ensure your application uses the updated `StorageConfig` with device API keys:

```rust
// With device API key (required for RLS policies)
let config = StorageConfig {
    supabase_url: url,
    supabase_anon_key: key,
    scout_api_key: device_api_key, // Required for RLS
    bucket_name: "artifacts".to_string(),
    allowed_extensions: vec![".mp4".to_string()],
};
```

## Testing

### Manual Testing Steps

**Important**: API calls use bucket name in URL, but database stores object path without bucket name.

1. **Upload Test**:
   ```bash
   # Should succeed: Upload to own device folder
   # API URL: /storage/v1/object/artifacts/1/2/test.mp4
   # Database stores: object.name = "1/2/test.mp4", bucket_id = "artifacts"
   curl -X POST "https://your-project.supabase.co/storage/v1/object/artifacts/1/2/test.mp4" \
     -H "Authorization: Bearer YOUR_ANON_KEY" \
     -H "apikey: YOUR_ANON_KEY" \
     -H "api_key: DEVICE_2_API_KEY" \
     --data-binary @test.mp4
   
   # Should fail: Upload to different device folder
   curl -X POST "https://your-project.supabase.co/storage/v1/object/artifacts/1/3/test.mp4" \
     -H "Authorization: Bearer YOUR_ANON_KEY" \
     -H "apikey: YOUR_ANON_KEY" \
     -H "api_key: DEVICE_2_API_KEY" \
     --data-binary @test.mp4
   ```

2. **View Test**:
   ```bash
   # Should succeed: View own artifact
   curl -X GET "https://your-project.supabase.co/storage/v1/object/artifacts/1/2/test.mp4" \
     -H "Authorization: Bearer YOUR_ANON_KEY" \
     -H "apikey: YOUR_ANON_KEY" \
     -H "api_key: DEVICE_2_API_KEY"
   ```

3. **Cross-Herd Test**:
   ```bash
   # Should fail: Device from herd 1 trying to access herd 2
   curl -X GET "https://your-project.supabase.co/storage/v1/object/artifacts/2/2/test.mp4" \
     -H "Authorization: Bearer YOUR_ANON_KEY" \
     -H "apikey: YOUR_ANON_KEY" \
     -H "api_key: DEVICE_FROM_HERD_1_API_KEY"
   ```

### Automated Testing

Run the SQL test file:
```sql
\i scout/database/test_storage_policies.sql
```

This will test:
- Helper function path parsing
- Policy existence and structure
- RLS enablement status
- Function permissions

## Troubleshooting

### Common Issues

1. **"Access denied" errors**:
   - Verify device API key is being sent in `api_key` header
   - Check that device belongs to the herd being accessed
   - Ensure file path follows exact format: `artifacts/{herd_id}/{device_id}/filename`

2. **Policies not applying**:
   - Verify RLS is enabled: `SELECT * FROM pg_tables WHERE schemaname='storage' AND tablename='objects';`
   - Check policy exists: `SELECT * FROM pg_policies WHERE schemaname='storage' AND tablename='objects';`

3. **Path parsing errors**:
   - Test helper functions manually: `SELECT storage.get_herd_id_from_path('artifacts/1/2/test.mp4');`
   - Ensure path components are valid integers

### Debug Queries

```sql
-- Check current authentication context
SELECT 
    auth.uid() as current_user,
    private.key_uid() as current_device;

-- Test policy conditions for specific path (object path without bucket name)
WITH test_path AS (SELECT '1/2/test.mp4' as path)
SELECT
    path,
    private.get_herd_id_from_path(path) as herd_id,
    private.get_device_id_from_path(path) as device_id,
    private.key_uid() as current_device,
    private.key_uid() = private.get_device_id_from_path(path) as device_matches
FROM test_path;
```

## Security Considerations

1. **API Key Protection**: Device API keys should be stored securely and rotated regularly
2. **Path Injection**: Helper functions validate input and return NULL for invalid paths
3. **Cross-Herd Access**: Policies strictly enforce herd boundaries
4. **Role Escalation**: View/edit roles are checked through secure definer functions
5. **File Overwrite**: x-upsert header allows overwriting but only within policy constraints

## Future Enhancements

1. **Audit Logging**: Add triggers to log storage access attempts
2. **Temporary URLs**: Implement time-limited access URLs for sharing
3. **Quota Management**: Add per-device or per-herd storage quotas  
4. **File Versioning**: Support multiple versions of the same artifact
5. **Metadata Storage**: Store artifact metadata in database with foreign key to storage

## Important Notes

### Bucket vs Object Path
- **API Calls**: Include bucket name in URL path (`/storage/v1/object/artifacts/1/2/file.mp4`)
- **Database Storage**: `storage.objects.name` contains only object path (`1/2/file.mp4`)
- **Bucket ID**: `storage.objects.bucket_id` contains bucket name (`artifacts`)
- **RLS Policies**: Work with the object path format, not the full API URL path