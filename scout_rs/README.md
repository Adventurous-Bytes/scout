# Scout Rust Client

A Rust client for the Scout API that allows uploading events and images to the Scout database.

## Features

- Upload individual events with images and tags
- Upload entire directories of images with metadata extracted from filenames
- **NEW**: Batch upload multiple events and files efficiently
- Automatic filename parsing for location and timestamp data
- Comprehensive error handling and upload reporting

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
scout_rs = { path = "./scout_rs" }
```

## Usage

### Basic Client Setup

```rust
use scout_rs::client::ScoutClient;

let mut client = ScoutClient::new(
    "http://localhost:3000/api/scout".to_string(),  // For local testing
    "your_api_key_here".to_string()
)?;

// Identify and load device/herd information into state
client.identify().await?;
```

### Upload Individual Event

```rust
use scout_rs::client::{Event, Tag};

let event = Event::new(
    Some("Test message".to_string()),
    Some("https://example.com/media".to_string()),
    None,
    Some("https://example.com/earthranger".to_string()),  // Optional EarthRanger URL
    40.7128,  // latitude
    -74.006,  // longitude
    100.0,    // altitude
    45.0,     // heading
    "image".to_string(),
    1,        // device_id
    timestamp_observation,
    false,    // is_public
    None      // session_id (optional)
);

let tags = vec![
    Tag::new(1, 100.0, 200.0, 50.0, 30.0, 0.95, "auto".to_string(), "person".to_string())
];

let result = client.post_event_with_tags(&event, &tags, "path/to/image.jpg").await?;
```

### Batch Upload Multiple Events

For uploading multiple events efficiently, use the batch upload method:

```rust
use scout_rs::client::{Event, Tag};

// Prepare multiple events and files
let events_and_files = vec![
    (event1, tags1, "path/to/image1.jpg".to_string()),
    (event2, tags2, "path/to/image2.jpg".to_string()),
    (event3, tags3, "path/to/image3.jpg".to_string()),
];

// Upload in batches of 10 (max 50 per batch)
let result = client.post_events_batch(&events_and_files, 10).await?;
result.print_summary();
```

### Upload Directory of Images

The directory upload feature automatically parses filenames to extract metadata. Filenames should follow this format:

```
device_id|timestamp|lat_underscore|lon_underscore|altitude|heading.ext
```

Example: `29|1733351509|19_754824|-155_15393|10|0.jpg`

**Fallback Behavior**: If filename parsing fails, the system will use default values instead of skipping the file. This ensures all images are uploaded even if they don't follow the expected naming convention.

**Device ID**: The device ID is automatically retrieved from stored state or fetched from the API using your API key.

#### Standard Directory Upload

The `upload_directory` method now uses batch uploads internally for better performance while maintaining the same simple interface:

```rust
// Create client and identify device/herd
let mut client = ScoutClient::new(scout_url, api_key)?;
client.identify().await?;  // Load device and herd into state

let result = client.upload_directory(
    "/path/to/photo/folder",
    Some("https://example.com/earthranger"),  // earthranger_url (optional)
    false,  // is_public
    Some("Uploaded via Rust client"),  // optional message
    Some(19.754824),  // default_latitude (optional)
    Some(-155.15393), // default_longitude (optional)
    Some(10.0),       // default_altitude (optional)
    Some(0.0),        // default_heading (optional)
    Some(20)          // batch_size (optional, default: 20)
).await?;

// Print upload summary
result.print_summary();
```

**Note**: This method now uses batch uploads internally with a default batch size of 10, providing significant performance improvements over individual file uploads.

#### Batch Directory Upload (Recommended)

For better performance with large directories, use the batch upload version:

```rust
// Create client and identify device/herd
let mut client = ScoutClient::new(scout_url, api_key)?;
client.identify().await?;  // Load device and herd into state

let result = client.upload_directory_batch(
    "/path/to/photo/folder",
    Some("https://example.com/earthranger"),  // earthranger_url (optional)
    false,  // is_public
    Some("Uploaded via Rust client"),  // optional message
    Some(19.754824),  // default_latitude (optional)
    Some(-155.15393), // default_longitude (optional)
    Some(10.0),       // default_altitude (optional)
    Some(0.0),        // default_heading (optional)
    10                // batch_size (1-50)
).await?;

// Print batch upload summary
result.print_summary();
```

### Using the Example Binaries

#### Standard Directory Upload

The `upload_directory` binary accepts command-line arguments for easy use:

```bash
# Basic usage
cargo run --bin upload_directory -- --directory /path/to/photos --api-key your_api_key

# With default location values for files that can't be parsed
cargo run --bin upload_directory -- \
  --directory /path/to/photos \
  --default-latitude 19.754824 \
  --default-longitude -155.15393 \
  --default-altitude 10.0 \
  --default-heading 0.0

# With custom message and public events
cargo run --bin upload_directory -- \
  --directory /path/to/photos \
  --message "Uploaded via Rust client" \
  --public

# With EarthRanger URL
cargo run --bin upload_directory -- \
  --directory /path/to/photos \
  --earthranger-url "https://your-earthranger-instance.com"

# With custom batch size
cargo run --bin upload_directory -- \
  --directory /path/to/photos \
  --batch-size 25
```

#### Batch Directory Upload (Recommended)

The `upload_batch` binary provides efficient batch uploading:

```bash
# Basic batch upload with default batch size of 10
cargo run --bin upload_batch -- --directory /path/to/photos

# Custom batch size (max 50)
cargo run --bin upload_batch -- \
  --directory /path/to/photos \
  --batch-size 25

# With all options
cargo run --bin upload_batch -- \
  --directory /path/to/photos \
  --batch-size 20 \
  --earthranger-url "https://your-earthranger-instance.com" \
  --message "Batch uploaded via Rust client" \
  --public \
  --default-latitude 19.754824 \
  --default-longitude -155.15393 \
  --default-altitude 10.0 \
  --default-heading 0.0 \
  --log-level debug
```

#### Command-line Options for Directory Upload

- `-d, --directory`: Directory path containing images to upload (required)
- `--scout-url`: Scout API URL (default: http://localhost:3000/api/scout)
- `--api-key`: API key for authentication (or set SCOUT_DEVICE_API_KEY env var)
- `--earthranger-url`: EarthRanger URL (optional)
- `--public`: Make events public (default: false)
- `-m, --message`: Message to include with events (optional)
- `--default-latitude`: Default latitude for files without metadata (optional)
- `--default-longitude`: Default longitude for files without metadata (optional)
- `--default-altitude`: Default altitude for files without metadata (optional)
- `--default-heading`: Default heading for files without metadata (optional)
- `--batch-size`: Batch size for uploads (1-50, default: 20)
- `--log-level`: Log level (trace, debug, info, warn, error) (default: info)

**Note**: The device ID is automatically retrieved from your API key, so you don't need to specify it manually.

## Testing with Localhost

For development and testing, you can use localhost instead of the production server:

### Environment Setup

```bash
# Set your API key
export SCOUT_API_KEY=your_api_key_here

# Optional: Set custom Scout URL (defaults to localhost:3000)
export SCOUT_URL=http://localhost:3000/api/scout
```

### Running the Frontend Server

Make sure your frontend server is running on localhost:3000:

```bash
cd frontend
npm run dev
```

### Testing Uploads

```bash
# Test directory upload with localhost
cargo run --bin upload_directory -- \
  --directory /path/to/photos \
  --scout-url http://localhost:3000/api/scout

# Test batch upload with localhost
cargo run --bin upload_batch -- \
  --directory /path/to/photos \
  --batch-size 5
```

## Performance Comparison

| Method          | Files | Time  | Network Requests |
| --------------- | ----- | ----- | ---------------- |
| Individual      | 100   | ~100s | 100              |
| Batch (size 10) | 100   | ~20s  | 10               |
| Batch (size 25) | 100   | ~12s  | 4                |

_Times are approximate and depend on network conditions and file sizes._

## Logging

The client uses the `tracing` crate for structured logging. You can control the log level using the `--log-level` option:

```bash
# Verbose debugging
cargo run --bin upload_batch -- --directory /path/to/photos --log-level debug

# Only warnings and errors
cargo run --bin upload_batch -- --directory /path/to/photos --log-level warn

# All logs including trace
cargo run --bin upload_batch -- --directory /path/to/photos --log-level trace
```

#### Environment Variables

You can also set the API key via environment variable:

```bash
export SCOUT_API_KEY=your_api_key_here
cargo run --bin upload_batch -- --directory /path/to/photos
```

## Filename Format

Images should be named with the following format:

- `device_id`: The device identifier (integer)
- `timestamp`: Unix timestamp when the image was taken
- `lat_underscore`: Latitude with underscores instead of decimal points (e.g., `19_754824` for 19.754824)
- `lon_underscore`: Longitude with underscores instead of decimal points (e.g., `-155_15393` for -155.15393)
- `altitude`: Altitude in meters
- `heading`: Heading in degrees
- `ext`: Image file extension (jpg, jpeg, png, webp)

Example: `29|1733351509|19_754824|-155_15393|10|0.jpg`

## Supported Image Formats

- JPEG (.jpg, .jpeg)
- PNG (.png)
- WebP (.webp)

## Error Handling

The client provides comprehensive error handling:

- **Filename Parsing**: Invalid filenames are logged and processed with default values instead of being skipped
- **Failed Uploads**: Network or server errors are tracked and reported
- **File System Errors**: Missing directories or files are caught and reported
- **Graceful Degradation**: The system continues processing other files even if some fail
- **Batch Failures**: If a batch fails, individual files are marked as failed and processing continues

### Fallback Values

When filename parsing fails, the following defaults are used:

- **Timestamp**: Current system time
- **Location**: Default latitude/longitude (if provided) or 0.0/0.0
- **Altitude**: Default altitude (if provided) or 0.0
- **Heading**: Default heading (if provided) or 0.0

### Upload Results

Both upload methods return detailed results:

```rust
// Standard upload result
let result: UploadResult = client.upload_directory(...).await?;
println!("Uploaded {} of {} files", result.successful_uploads, result.total_files);

// Batch upload result
let batch_result: BatchUploadResult = client.upload_directory_batch(...).await?;
println!("Completed {} batches, uploaded {} of {} files",
         batch_result.successful_batches,
         batch_result.successful_uploads,
         batch_result.total_files);
```

## API Endpoints

The client supports both individual and batch upload endpoints:

- **Individual**: `POST /events` - Upload single event with file
- **Batch**: `POST /events/batch` - Upload multiple events with files (max 50 per batch)

The batch endpoint is more efficient for uploading multiple files as it reduces network overhead and server load.

## Testing

### Setting Up Environment Variables

To run the tests, you need to set up your Scout API key as an environment variable. Create a `.env` file in the project root:

```bash
# Create .env file
echo "SCOUT_API_KEY=your_actual_api_key_here" > .env
```

**Important**: The `.env` file is already added to `.gitignore` to prevent accidentally committing your API key.

### Running Tests

```bash
# Run all tests
cargo test

# Run only unit tests (no API calls)
cargo test --lib

# Run integration tests with API calls
cargo test --test scout_client

# Run specific test
cargo test test_session_api_integration

# Run all session-related tests
cargo test session

# Run session creation tests specifically
cargo test test_session_creation_api

# Run tests with verbose output
cargo test -- --nocapture
```

### Test Categories

- **Unit Tests**: Test individual functions without API calls
- **Integration Tests**: Test API interactions (require valid API key)
- **Error Handling Tests**: Test error scenarios with invalid API keys
- **Session Tests**: Test session creation, retrieval, and management functionality

### Test Environment Variables

The tests automatically load environment variables from the `.env` file. If no `SCOUT_API_KEY` is found, integration tests will be skipped with a message.

### Session Testing

The client includes comprehensive tests for session management functionality:

#### Session Creation Tests

- **`test_session_creation_api`**: Tests creating sessions with realistic data and validates the returned session ID
- **`test_session_creation_error_handling`**: Tests error handling with invalid data (negative device IDs, invalid timestamps, etc.)
- **`test_session_api_integration`**: Tests the full session lifecycle including creation, retrieval, and cleanup

#### Session Management Tests

- **`test_sessions_by_herd`**: Tests retrieving sessions by herd ID
- **`test_session_update_methods`**: Tests session update functionality
- **`test_session_with_id`**: Tests session creation with pre-assigned IDs

#### Running Session Tests

```bash
# Run all session-related tests
cargo test session

# Run specific session creation test
cargo test test_session_creation_api

# Run session error handling tests
cargo test test_session_creation_error_handling

# Run with verbose output to see detailed logs
cargo test test_session_creation_api -- --nocapture
```

#### Session Test Features

- **Realistic Data**: Tests use realistic GPS coordinates, altitude, and velocity data
- **Validation**: Tests validate that created sessions have correct data and positive IDs
- **Cleanup**: Tests automatically clean up created sessions to avoid test pollution
- **Error Handling**: Tests verify proper error handling for invalid inputs
- **Environment Variables**: Tests use environment variables for API configuration

### Example .env File

```env
# Scout API Key for testing
SCOUT_API_KEY=your_actual_api_key_here

# Optional: Custom Scout URL for testing
SCOUT_URL=https://www.adventurelabs.earth/api/scout
```

## License

GPL-3.0
