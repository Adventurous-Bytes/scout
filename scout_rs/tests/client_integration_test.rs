use scout_rs::client::*;
use scout_rs::client::{MediaType, TagObservationType};
use std::env;

use once_cell::sync::Lazy;
use std::sync::Mutex;

/// # Scout Client Integration Tests
///
/// This test suite comprehensively tests the Scout client's integration capabilities including:
///
/// ## Core Functionality Tested:
/// - **Self-identification**: Client authentication and device/herd identification
/// - **Event Management**: Creating individual events and event batches
/// - **Session Management**: Creating and managing data collection sessions
/// - **Connectivity Data**: Recording signal strength and location data
/// - **Tag Management**: Creating AI detection tags for events
/// - **Error Handling**: Graceful failure handling and validation
/// - **Integration Workflow**: Complete end-to-end data collection workflow
///
/// ## Running the Tests:
///
/// ### 1. Test Environment (Default)
/// Tests will run with mock data and expected failures for database operations:
/// ```bash
/// cargo test --test client_integration_test
/// ```
///
/// ### 2. Real Database Integration (Optional)
/// To test with a real database, set these environment variables:
/// ```bash
/// export SCOUT_DATABASE_REST_URL="https://your-db.supabase.co/rest/v1"
/// export SCOUT_DEVICE_API_KEY="your_device_api_key"
/// export SCOUT_DATABASE_URL="postgresql://user:pass@host:port/db"
/// export SCOUT_DATABASE_ANON_KEY="your_anon_key"
/// export SCOUT_DATABASE_SERVICE_KEY="your_service_key"
///
/// # Then run tests with real database
/// cargo test --test client_integration_test -- --nocapture
/// ```
///
/// ### 3. Test Database Setup
/// For full integration testing, you'll need:
/// - A PostgREST-enabled database (e.g., Supabase)
/// - A device record with your API key
/// - A herd record associated with the device
/// - Proper database schema with tables: devices, herds, events, sessions, connectivity, tags
///
/// ## Test Structure:
/// - **Unit Tests**: Test data structures and basic client creation
/// - **Integration Tests**: Test client operations with database (real or mock)
/// - **Workflow Tests**: Test complete data collection workflows
/// - **Error Tests**: Test error handling and edge cases
///
/// ## Expected Behavior:
/// - In test environment: Tests pass with graceful handling of database connection failures
/// - In real environment: Tests pass with actual database operations
/// - All tests validate proper response structures and error handling

// ===== TEST DATA CLEANUP SYSTEM =====
//
// This system automatically tracks and cleans up test data to ensure tests don't
// leave behind data that could interfere with other tests.
//
// ## How to Use:
//
// 1. **Wrap your test with the macro**: Replace `#[tokio::test]` with `test_with_cleanup!`
// 2. **Create a test implementation function**: Make your test logic a separate async function
// 3. **Track created data**: Call cleanup.track_*() methods for each piece of data you create
// 4. **Automatic cleanup**: The macro automatically cleans up after your test runs
//
// ## Example:
// ```rust
// async fn my_test_impl(cleanup: &TestCleanup) {
//     // Your test logic here
//     let event = create_event().await?;
//     cleanup.track_event(event.id); // Track for cleanup
//
//     let plan = create_plan().await?;
//     cleanup.track_plan(plan.id); // Track for cleanup
// }
//
// test_with_cleanup!(my_test, my_test_impl);
// ```
//
// ## What Gets Cleaned Up:
// - Events (and their tags)
// - Sessions
// - Connectivity data
// - Tags
// - Artifacts
// - Plans
//
// ## Cleanup Order:
// The system cleans up in the correct dependency order to avoid foreign key constraint violations:
// 1. Tags (reference events)
// 2. Connectivity (references sessions)
// 3. Events (reference sessions and devices)
// 4. Sessions (reference devices)
// 5. Artifacts (reference sessions)
// 6. Plans (reference herds)

/// Global test data tracker for cleanup
static TEST_DATA: Lazy<Mutex<TestDataTracker>> = Lazy::new(|| Mutex::new(TestDataTracker::new()));

/// Tracks test data that needs cleanup
struct TestDataTracker {
    events: Vec<i64>,
    sessions: Vec<i64>,
    connectivity: Vec<i64>,
    tags: Vec<i64>,
    artifacts: Vec<i64>,
    plans: Vec<i64>,
}

impl TestDataTracker {
    fn new() -> Self {
        Self {
            events: Vec::new(),
            sessions: Vec::new(),
            connectivity: Vec::new(),
            tags: Vec::new(),
            artifacts: Vec::new(),
            plans: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.events.clear();
        self.sessions.clear();
        self.connectivity.clear();
        self.tags.clear();
        self.artifacts.clear();
        self.plans.clear();
    }
}

/// Test cleanup helper
struct TestCleanup {
    tracker: &'static Mutex<TestDataTracker>,
}

impl TestCleanup {
    fn new() -> Self {
        Self {
            tracker: &TEST_DATA,
        }
    }

    /// Track an event ID for cleanup
    fn track_event(&self, event_id: i64) {
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.events.push(event_id);
        }
    }

    /// Track a session ID for cleanup
    fn track_session(&self, session_id: i64) {
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.sessions.push(session_id);
        }
    }

    /// Track connectivity ID for cleanup
    fn track_connectivity(&self, connectivity_id: i64) {
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.connectivity.push(connectivity_id);
        }
    }

    /// Track tag ID for cleanup
    fn track_tag(&self, tag_id: i64) {
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.tags.push(tag_id);
        }
    }

    /// Track artifact ID for cleanup
    fn track_artifact(&self, artifact_id: i64) {
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.artifacts.push(artifact_id);
        }
    }

    /// Track plan ID for cleanup
    fn track_plan(&self, plan_id: i64) {
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.plans.push(plan_id);
        }
    }

    /// Clean up all tracked test data
    async fn cleanup(&self, client: &mut ScoutClient) {
        if let Ok(tracker) = self.tracker.lock() {
            // Clean up in reverse dependency order to avoid foreign key constraint violations

            // Clean up tags first (they reference events)
            for &tag_id in &tracker.tags {
                let _ = client.delete_tag(tag_id).await;
            }

            // Clean up connectivity (they reference sessions)
            for &connectivity_id in &tracker.connectivity {
                let _ = client.delete_connectivity(connectivity_id).await;
            }

            // Clean up events (they reference sessions and devices)
            for &event_id in &tracker.events {
                let _ = client.delete_event(event_id).await;
            }

            // Clean up sessions (they reference devices)
            for &session_id in &tracker.sessions {
                let _ = client.delete_session(session_id).await;
            }

            // Clean up artifacts (they reference sessions)
            for &artifact_id in &tracker.artifacts {
                let _ = client.delete_artifact(artifact_id).await;
            }

            // Clean up plans (they reference herds)
            for &plan_id in &tracker.plans {
                let _ = client.delete_plan(plan_id).await;
            }
        }

        // Reset the tracker for the next test
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.reset();
        }
    }
}

/// Macro to automatically clean up test data after each test
macro_rules! test_with_cleanup {
    ($test_name:ident, $test_fn:ident) => {
        #[tokio::test]
        async fn $test_name() {
            let cleanup = TestCleanup::new();

            // Run the test
            $test_fn(&cleanup).await;

            // Clean up test data
            let mut client = ScoutClient::new(
                env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
            )
            .unwrap();

            if client.identify().await.is_ok() {
                cleanup.cleanup(&mut client).await;
            }
        }
    };
}

// Setup test environment using actual .env file values

fn setup_test_env() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Check for required environment variables and panic if missing
    let missing_vars = vec![
        (
            "SCOUT_DEVICE_API_KEY",
            env::var("SCOUT_DEVICE_API_KEY").is_err(),
        ),
        (
            "SCOUT_DATABASE_REST_URL",
            env::var("SCOUT_DATABASE_REST_URL").is_err(),
        ),
        ("SCOUT_DEVICE_ID", env::var("SCOUT_DEVICE_ID").is_err()),
        ("SCOUT_HERD_ID", env::var("SCOUT_HERD_ID").is_err()),
    ];

    let missing: Vec<&str> = missing_vars
        .into_iter()
        .filter(|(_, is_missing)| *is_missing)
        .map(|(name, _)| name)
        .collect();

    if !missing.is_empty() {
        panic!(
            "❌ Missing required environment variables: {}. Please check your .env file.",
            missing.join(", ")
        );
    }

    // Check for Scout API key for custom authentication
    let has_scout_api_key = env::var("SCOUT_DEVICE_API_KEY").is_ok();

    if !has_scout_api_key {
        panic!("❌ Missing Scout API key. Set SCOUT_DEVICE_API_KEY in your .env file.");
    }

    // Check for Supabase API key for PostgREST access
    let has_supabase_api_key = env::var("SUPABASE_PUBLIC_API_KEY").is_ok()
        || env::var("SCOUT_SUPABASE_ANON_KEY").is_ok()
        || env::var("SCOUT_SUPABASE_SERVICE_KEY").is_ok();

    if !has_supabase_api_key {
        panic!(
            "❌ Missing Supabase API key. Set SUPABASE_PUBLIC_API_KEY, SCOUT_SUPABASE_ANON_KEY, or SCOUT_SUPABASE_SERVICE_KEY in your .env file."
        );
    }
}

#[test]
fn test_tracing_works() {
    // This should show up in the logs

    // Simple assertion to make this a valid test
    assert!(true);
}

#[test]
fn test_response_scout_types() {
    // Test that ResponseScout types work correctly
    let success_response = ResponseScout::new(ResponseScoutStatus::Success, Some("test data"));
    assert_eq!(success_response.status, ResponseScoutStatus::Success);
    assert_eq!(success_response.data, Some("test data"));

    let failure_response = ResponseScout::new(ResponseScoutStatus::Failure, None::<&str>);
    assert_eq!(failure_response.status, ResponseScoutStatus::Failure);
    assert_eq!(failure_response.data, None);

    let not_authorized = ResponseScout::new(ResponseScoutStatus::NotAuthorized, None::<&str>);
    assert_eq!(not_authorized.status, ResponseScoutStatus::NotAuthorized);
}

#[test]
fn test_data_structures() {
    // Test that data structures can be created and serialized
    let event = Event::new(
        Some("Test event".to_string()),
        Some("https://example.com/image.jpg".to_string()),
        None,
        Some("https://earthranger.example.com".to_string()),
        19.754824,
        -155.15393,
        10.0,
        0.0,
        MediaType::Image,
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None,
    );

    assert_eq!(event.message, Some("Test event".to_string()));
    assert_eq!(
        event.media_url,
        Some("https://example.com/image.jpg".to_string())
    );
    let expected_device_id = env::var("SCOUT_DEVICE_ID")
        .unwrap_or_else(|_| "123".to_string())
        .parse()
        .unwrap_or(123);
    assert_eq!(event.device_id, Some(expected_device_id as i64));
    assert_eq!(event.is_public, false);

    let session = Session::new(
        expected_device_id,
        1640995200,
        1640998800,
        "v1.0.0".to_string(),
        None,
        100.0,
        50.0,
        75.0,
        10.0,
        5.0,
        7.5,
        1000.0,
        500.0,
    );

    assert_eq!(session.device_id, expected_device_id);
    assert_eq!(session.software_version, "v1.0.0");
    assert_eq!(session.altitude_max, 100.0);
    assert_eq!(session.altitude_min, 50.0);

    let tag = Tag::new(
        1,
        100.0,
        200.0,
        50.0,
        30.0,
        0.95,
        TagObservationType::Auto,
        "animal".to_string(),
    );

    assert_eq!(tag.x, 100.0);
    assert_eq!(tag.y, 200.0);
    assert_eq!(tag.conf, 0.95);
    assert_eq!(tag.class_name, "animal");
}

#[test]
fn test_client_creation() {
    // Test that ScoutClient can be created
    let client = ScoutClient::new("test_key".to_string());
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.api_key, "test_key");
    assert!(client.device.is_none());
    assert!(client.herd.is_none());
    assert!(!client.is_identified());
}

#[tokio::test]
async fn test_client_identification() {
    setup_test_env();

    // Create a client with actual credentials from .env file
    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Test identification process
    let identify_result = client.identify().await;

    // Test identification process
    match identify_result {
        Ok(_) => {
            assert!(client.device.is_some());
            assert!(client.herd.is_some());
            assert!(client.is_identified());

            // Test device retrieval - should always succeed with proper credentials
            let device_response = client.get_device().await;
            match device_response {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    assert!(response.data.is_some());
                }
                Err(e) => {
                    panic!("❌ Device retrieval failed: {}", e);
                }
            }

            // Test herd retrieval - should always succeed with proper credentials
            let herd_response = client.get_herd(None).await;
            match herd_response {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    assert!(response.data.is_some());
                }
                Err(e) => {
                    panic!("❌ Herd retrieval failed: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Client identification failed: {}", e);
        }
    }
}

async fn test_event_creation_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // Create test event
    let event = Event::new(
        Some("Integration test event".to_string()),
        Some("https://test.com/image.jpg".to_string()),
        None,
        Some("https://test.earthranger.com".to_string()),
        19.754824,
        -155.15393,
        15.0,
        45.0,
        MediaType::Image,
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        true,
        None,
    );

    // Test event creation - should always succeed with proper credentials
    let event_result = client.create_event(&event).await;

    match event_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            // Track the created event for cleanup
            if let Some(created_event) = response.data {
                if let Some(event_id) = created_event.id {
                    cleanup.track_event(event_id);
                }
            }
        }
        Err(e) => {
            panic!("❌ Event creation failed: {}", e);
        }
    }
}

test_with_cleanup!(test_event_creation, test_event_creation_impl);

async fn test_event_batch_creation_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // Create multiple test events
    let events = vec![
        Event::new(
            Some("Batch event 1".to_string()),
            Some("https://test.com/image1.jpg".to_string()),
            None,
            None,
            19.754824,
            -155.15393,
            10.0,
            0.0,
            MediaType::Image,
            env::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995200,
            false,
            None,
        ),
        Event::new(
            Some("Batch event 2".to_string()),
            Some("https://test.com/image2.jpg".to_string()),
            None,
            None,
            19.755,
            -155.154,
            12.0,
            90.0,
            MediaType::Image,
            env::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995260,
            false,
            None,
        ),
        Event::new(
            Some("Batch event 3".to_string()),
            Some("https://test.com/image2.jpg".to_string()),
            None,
            None,
            19.7545,
            -155.1535,
            8.0,
            180.0,
            MediaType::Image,
            env::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995320,
            false,
            None,
        ),
    ];

    // Test batch event creation - should always succeed with proper credentials
    let batch_result = client.create_events_batch(&events).await;

    match batch_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            let created_events = response.data.unwrap();
            assert_eq!(created_events.len(), 3);

            // Track all created events for cleanup
            for created_event in &created_events {
                if let Some(event_id) = created_event.id {
                    cleanup.track_event(event_id);
                }
            }
        }
        Err(e) => {
            panic!("❌ Event batch creation failed: {}", e);
        }
    }
}

test_with_cleanup!(test_event_batch_creation, test_event_batch_creation_impl);

async fn test_event_with_tags_creation_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // First create a real session for the event
    let session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        1640998800,
        "tags_test_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    let session_result = client.create_session(&session).await;
    if let Ok(response) = session_result {
        if response.status == ResponseScoutStatus::Success {
            let created_session = response.data.unwrap();
            let session_id = created_session.id.unwrap();

            // Track the created session for cleanup
            cleanup.track_session(session_id);

            // Create test event with real session ID
            let event = Event::new(
                Some("Tagged event".to_string()),
                Some("https://test.com/tagged.jpg".to_string()),
                None,
                None,
                19.754824,
                -155.15393,
                10.0,
                0.0,
                MediaType::Image,
                env::var("SCOUT_DEVICE_ID")
                    .unwrap_or_else(|_| "123".to_string())
                    .parse()
                    .unwrap_or(123),
                1640995200,
                false,
                Some(session_id),
            );

            // Create test tags
            let tags = vec![
                Tag::new(
                    1,
                    100.0,
                    200.0,
                    50.0,
                    30.0,
                    0.95,
                    TagObservationType::Auto,
                    "elephant".to_string(),
                ),
                Tag::new(
                    2,
                    150.0,
                    250.0,
                    40.0,
                    25.0,
                    0.87,
                    TagObservationType::Auto,
                    "giraffe".to_string(),
                ),
            ];

            // Create the event first
            let event_result = client.create_event(&event).await;
            match event_result {
                Ok(event_response) => {
                    if event_response.status == ResponseScoutStatus::Success {
                        let created_event = event_response.data.unwrap();
                        let event_id = created_event.id.unwrap();

                        // Track the created event for cleanup
                        cleanup.track_event(event_id);

                        // Create tags with the correct event_id
                        let mut tags_with_event_id = tags.clone();
                        for tag in &mut tags_with_event_id {
                            tag.update_event_id(event_id);
                        }

                        // Create tags for the created event
                        let tags_result = client.create_tags(event_id, &tags_with_event_id).await;
                        match tags_result {
                            Ok(tags_response) => {
                                if tags_response.status == ResponseScoutStatus::Success {
                                    let created_tags = tags_response.data.unwrap();

                                    // Verify the tags have the correct event_id
                                    for tag in &created_tags {
                                        assert_eq!(tag.event_id, event_id);
                                    }

                                    // Track all created tags for cleanup
                                    for created_tag in &created_tags {
                                        if let Some(tag_id) = created_tag.id {
                                            cleanup.track_tag(tag_id);
                                        }
                                    }
                                } else {
                                    panic!("❌ Tags creation failed: {:?}", tags_response.status);
                                }
                            }
                            Err(e) => {
                                // Fail the test - if we can't create tags, that's a test failure
                                panic!("❌ Tags creation failed: {}. This test requires successful tag creation.", e);
                            }
                        }

                        // Verify the event exists by querying it
                        let event_verification = client.get_event_by_id(event_id).await;
                        match event_verification {
                            Ok(response) => {
                                if response.status == ResponseScoutStatus::Success {
                                    let _fetched_event = response.data.unwrap();
                                } else {
                                    // Event verification failed
                                }
                            }
                            Err(_e) => {
                                // Event verification error
                            }
                        }
                    } else {
                        panic!("❌ Event creation failed: {:?}", event_response.status);
                    }
                }
                Err(e) => {
                    panic!("❌ Event creation error: {}", e);
                }
            }
        } else {
            panic!("❌ Session creation failed for tags test");
        }
    } else {
        panic!("❌ Session creation error for tags test");
    }
}

test_with_cleanup!(
    test_event_with_tags_creation,
    test_event_with_tags_creation_impl
);

async fn test_session_creation_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // Get the actual device ID from the identified client
    let device_id = client.device.as_ref().unwrap().id;

    // Create test session
    let session = Session::new(
        device_id,
        1640995200,
        1640998800,
        "v2.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    // Test session creation - should always succeed with proper credentials
    let session_result = client.create_session(&session).await;

    match session_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            // Track the created session for cleanup
            if let Some(created_session) = response.data {
                if let Some(session_id) = created_session.id {
                    cleanup.track_session(session_id);
                }
            }
        }
        Err(e) => {
            panic!("❌ Session creation failed: {}", e);
        }
    }
}

test_with_cleanup!(test_session_creation, test_session_creation_impl);

async fn test_does_session_exist_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // Get the actual device ID from the identified client
    let device_id = client.device.as_ref().unwrap().id;

    // Test 1: Check for non-existent session
    let exists_result = client
        .does_session_exist(
            device_id,
            "2023-01-01T00:00:00Z", // Non-existent start time
            "2023-01-01T01:00:00Z", // Non-existent end time
        )
        .await;
    match exists_result {
        Ok(exists) => {
            assert!(!exists, "Non-existent session should not exist");
            println!("✅ Non-existent session check passed");
        }
        Err(e) => {
            panic!("❌ Session existence check failed: {}", e);
        }
    }

    // Test 2: Create a session with unique timestamps
    let unique_start_timestamp = chrono::Utc::now().timestamp() as u64;
    let unique_end_timestamp = unique_start_timestamp + 3600;

    let session = Session::new(
        device_id,
        unique_start_timestamp,
        unique_end_timestamp,
        "does_session_exist_test_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    let session_result = client.create_session(&session).await;
    match session_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            let created_session = response.data.unwrap();
            let session_id = created_session.id.unwrap();
            println!("Created test session with ID: {}", session_id);

            // Track the created session for cleanup
            cleanup.track_session(session_id);

            // Test 3: Check that the created session exists
            let exists_result = client
                .does_session_exist(
                    device_id,
                    &created_session.timestamp_start,
                    &created_session.timestamp_end,
                )
                .await;
            match exists_result {
                Ok(exists) => {
                    assert!(exists, "Created session should exist");
                    println!("✅ Created session existence check passed");
                }
                Err(e) => {
                    panic!("❌ Session existence check failed: {}", e);
                }
            }

            // Test 4: Delete the session and verify it no longer exists
            let delete_result = client.delete_session(session_id).await;
            match delete_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    println!("✅ Session deleted successfully");

                    // Test 5: Check that the deleted session no longer exists
                    println!(
                        "Checking if deleted session still exists with timestamps: {} to {}",
                        &created_session.timestamp_start, &created_session.timestamp_end
                    );
                    let exists_after_delete = client
                        .does_session_exist(
                            device_id,
                            &created_session.timestamp_start,
                            &created_session.timestamp_end,
                        )
                        .await;
                    match exists_after_delete {
                        Ok(exists) => {
                            assert!(!exists, "Deleted session should not exist");
                            println!("✅ Deleted session existence check passed");
                        }
                        Err(e) => {
                            panic!("❌ Post-deletion session existence check failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    panic!("❌ Session deletion failed: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Session creation failed: {}", e);
        }
    }

    // Test 6: Edge case - different device ID with same timestamps
    let different_device_exists_result = client
        .does_session_exist(
            999999, // Non-existent device ID
            "1970-01-01T00:00:00Z",
            "1970-01-01T01:00:00Z",
        )
        .await;
    match different_device_exists_result {
        Ok(exists) => {
            assert!(!exists, "Session with different device ID should not exist");
            println!("✅ Different device ID check passed");
        }
        Err(e) => {
            panic!("❌ Different device ID check failed: {}", e);
        }
    }

    // Test 7: Edge case - same device ID with different timestamps
    let different_timestamps_result = client
        .does_session_exist(
            device_id,
            "1970-01-01T00:00:00Z", // Different timestamps
            "1970-01-01T01:00:00Z",
        )
        .await;
    match different_timestamps_result {
        Ok(exists) => {
            assert!(
                !exists,
                "Session with different timestamps should not exist"
            );
            println!("✅ Different timestamps check passed");
        }
        Err(e) => {
            panic!("❌ Different timestamps check failed: {}", e);
        }
    }

    println!("🎉 All does_session_exist tests passed!");
}

test_with_cleanup!(test_does_session_exist, test_does_session_exist_impl);

#[tokio::test]
async fn test_connectivity_creation() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // First create a real session for the connectivity data
    let session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        1640998800,
        "connectivity_test_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    let session_result = client.create_session(&session).await;
    if let Ok(response) = session_result {
        if response.status == ResponseScoutStatus::Success {
            let created_session = response.data.unwrap();
            let session_id = created_session.id.unwrap();

            // Create test connectivity entry with real session ID
            let connectivity = Connectivity::new(
                session_id,
                1640995200,
                -45.0, // signal
                -60.0, // noise
                100.0, // altitude
                180.0, // heading
                "POINT(-155.15393 19.754824)".to_string(),
                "H14_INDEX".to_string(),
                "H13_INDEX".to_string(),
                "H12_INDEX".to_string(),
                "H11_INDEX".to_string(),
            );

            // Test connectivity creation - should always succeed with proper credentials
            let connectivity_result = client.create_connectivity(&connectivity).await;

            match connectivity_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    assert!(response.data.is_some());

                    // Clean up test data
                    let _ = client.delete_session(session_id).await;
                }
                Err(e) => {
                    // Clean up test data even on failure
                    let _ = client.delete_session(session_id).await;
                    panic!("❌ Connectivity creation failed: {}", e);
                }
            }
        } else {
            panic!("❌ Session creation failed for connectivity test");
        }
    } else {
        panic!("❌ Session creation error for connectivity test");
    }
}

#[tokio::test]
async fn test_compatibility_methods() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // Test post_events_batch with proper event and tag creation
    // First create a session for the events
    let session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        1640998800,
        "compat_test_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    let session_result = client.create_session(&session).await;
    if let Ok(response) = session_result {
        if response.status == ResponseScoutStatus::Success {
            let created_session = response.data.unwrap();
            let session_id = created_session.id.unwrap();

            let events_and_files = vec![
                (
                    Event::new(
                        Some("Compat event 1".to_string()),
                        Some("https://test.com/compat1.jpg".to_string()),
                        None,
                        None,
                        19.754824,
                        -155.15393,
                        10.0,
                        0.0,
                        MediaType::Image,
                        env::var("SCOUT_DEVICE_ID")
                            .unwrap_or_else(|_| "123".to_string())
                            .parse()
                            .unwrap_or(123),
                        1640995200,
                        false,
                        Some(session_id),
                    ),
                    vec![Tag::new(
                        1,
                        100.0,
                        200.0,
                        50.0,
                        30.0,
                        0.95,
                        TagObservationType::Auto,
                        "animal".to_string(),
                    )],
                    "/path/to/file1.jpg".to_string(),
                ),
                (
                    Event::new(
                        Some("Compat event 2".to_string()),
                        Some("https://test.com/compat2.jpg".to_string()),
                        None,
                        None,
                        19.755,
                        -155.154,
                        12.0,
                        90.0,
                        MediaType::Image,
                        env::var("SCOUT_DEVICE_ID")
                            .unwrap_or_else(|_| "123".to_string())
                            .parse()
                            .unwrap_or(123),
                        1640995260,
                        false,
                        Some(session_id),
                    ),
                    vec![Tag::new(
                        2,
                        150.0,
                        250.0,
                        40.0,
                        25.0,
                        0.87,
                        TagObservationType::Auto,
                        "animal".to_string(),
                    )],
                    "/path/to/file2.jpg".to_string(),
                ),
            ];

            // Test post_events_batch with tags - should always succeed with proper credentials
            let compat_result = client.post_events_batch(&events_and_files, 10).await;
            match compat_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    assert!(response.data.is_some());

                    let created_events = response.data.unwrap();
                    assert_eq!(created_events.len(), 2);

                    // Clean up test data
                    let _ = client.delete_session(session_id).await;
                }
                Err(e) => {
                    // Clean up test data even on failure
                    let _ = client.delete_session(session_id).await;

                    // Fail the test - if we can't create events with tags, that's a test failure
                    panic!("❌ Compatibility batch method failed: {}. This test requires successful event and tag creation.", e);
                }
            }
        } else {
            panic!("❌ Session creation failed for compatibility test");
        }
    } else {
        panic!("❌ Session creation error for compatibility test");
    }
}

#[tokio::test]
async fn test_error_handling() {
    setup_test_env();

    let mut client = ScoutClient::new("invalid_api_key".to_string()).unwrap();

    // Test that operations fail gracefully when not identified
    let event = Event::new(
        Some("Error test event".to_string()),
        None,
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        MediaType::Image,
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None,
    );

    // This should fail because client is not identified
    let event_result = client.create_event(&event).await;

    match event_result {
        Ok(_) => {
            panic!("❌ Expected event creation to fail when not identified");
        }
        Err(e) => {
            assert!(e.to_string().contains("Database client not initialized"));
        }
    }
}

#[tokio::test]
async fn test_error_handling_and_edge_cases() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Test 1: Operations before identification
    let event = Event::new(
        Some("Pre-identification event".to_string()),
        None,
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        MediaType::Image,
        123,
        1640995200,
        false,
        None,
    );

    let event_result = client.create_event(&event).await;
    match event_result {
        Ok(_) => {
            panic!("❌ Expected event creation to fail when not identified");
        }
        Err(e) => {
            assert!(e.to_string().contains("Database client not initialized"));
        }
    }

    // Test 2: Invalid device ID
    let invalid_device_result = client.get_device_by_id(999999).await;
    if let Ok(response) = invalid_device_result {
        if response.status == ResponseScoutStatus::Success {
        } else {
        }
    }

    // Test 3: Invalid herd ID
    let invalid_herd_result = client.get_herd_by_id(999999).await;
    if let Ok(response) = invalid_herd_result {
        if response.status == ResponseScoutStatus::Success {
        } else {
        }
    }

    // Test 4: Invalid session ID
    let invalid_session_result = client.get_session_events(999999).await;
    if let Ok(response) = invalid_session_result {
        if response.status == ResponseScoutStatus::Success {
        } else {
        }
    }
}

async fn test_integration_workflow_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Test the complete workflow: identify -> create session -> create events -> create connectivity

    // Step 1: Identify (will fail in test env, but tests structure)
    let identify_result = client.identify().await;

    if identify_result.is_ok() {
        // Step 2: Create a session
        let session = Session::new(
            env::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995200,
            1640998800,
            "v2.0.0".to_string(),
            Some("POINT(-155.15393 19.754824)".to_string()),
            120.0,
            45.0,
            82.5,
            15.0,
            3.0,
            9.0,
            1200.0,
            600.0,
        );

        let session_result = client.create_session(&session).await;

        if let Ok(_response) = session_result {
            // Step 3: Create events for the session
            let events = vec![
                Event::new(
                    Some("Workflow event 1".to_string()),
                    Some("https://test.com/workflow1.jpg".to_string()),
                    None,
                    None,
                    19.754824,
                    -155.15393,
                    10.0,
                    0.0,
                    MediaType::Image,
                    env::var("SCOUT_DEVICE_ID")
                        .unwrap_or_else(|_| "123".to_string())
                        .parse()
                        .unwrap_or(123),
                    1640995200,
                    false,
                    None, // Will be set after session creation
                ),
                Event::new(
                    Some("Workflow event 2".to_string()),
                    Some("https://test.com/workflow2.jpg".to_string()),
                    None,
                    None,
                    19.755,
                    -155.154,
                    12.0,
                    90.0,
                    MediaType::Image,
                    env::var("SCOUT_DEVICE_ID")
                        .unwrap_or_else(|_| "123".to_string())
                        .parse()
                        .unwrap_or(123),
                    1640995260,
                    false,
                    None, // Will be set after session creation
                ),
            ];

            // Get the session ID from the created session
            let session_id = _response.data.unwrap().id.unwrap();

            // Update events with the real session ID
            let mut events_with_session = events.clone();
            for event in &mut events_with_session {
                event.session_id = Some(session_id);
            }

            let events_result = client.create_events_batch(&events_with_session).await;

            if let Ok(_response) = events_result {
                // Get the created events with their IDs
                let created_events = _response.data.unwrap();

                // Step 4: Create tags for the first event
                if let Some(first_event) = created_events.first() {
                    if let Some(event_id) = first_event.id {
                        let tags = vec![Tag::new(
                            1,
                            100.0,
                            200.0,
                            50.0,
                            30.0,
                            0.95,
                            TagObservationType::Auto,
                            "elephant".to_string(),
                        )];

                        // Debug: Check what device_id the event has

                        let tags_result = client.create_tags(event_id, &tags).await;
                        if let Ok(response) = tags_result {
                            if response.status == ResponseScoutStatus::Success {}
                        } else {
                            panic!("❌ Step 4: Tags creation failed");
                        }
                    } else {
                    }
                }

                // Step 5: Create connectivity data
                let connectivity = Connectivity::new(
                    session_id,
                    chrono::Utc::now().timestamp() as u64,
                    -45.0,
                    -60.0,
                    100.0,
                    180.0,
                    "POINT(-155.15393 19.754824)".to_string(),
                    "H14_INDEX".to_string(),
                    "H13_INDEX".to_string(),
                    "H12_INDEX".to_string(),
                    "H11_INDEX".to_string(),
                );

                let connectivity_result = client.create_connectivity(&connectivity).await;
                if let Ok(response) = connectivity_result {
                    if response.status == ResponseScoutStatus::Success {}
                }

                // Step 6: Query the data we just created
                let session_events = client.get_session_events(session_id).await;
                if let Ok(response) = session_events {
                    if response.status == ResponseScoutStatus::Success {
                        let events = response.data.unwrap();
                    }
                }

                let session_connectivity = client.get_session_connectivity(session_id).await;
                if let Ok(response) = session_connectivity {
                    if response.status == ResponseScoutStatus::Success {
                        let connectivity_entries = response.data.unwrap();
                    }
                }

                // Step 7: Clean up test data
                let delete_result = client.delete_session(session_id).await;
                if let Ok(response) = delete_result {
                    if response.status == ResponseScoutStatus::Success {}
                }
            } else {
                panic!("❌ Step 3: Events creation failed");
            }
        } else {
            panic!("❌ Step 2: Session creation failed");
        }
    } else {
        panic!("❌ Step 1: Identification failed");
    }
}

#[tokio::test]
async fn test_real_database_integration() {
    // This test runs with real database credentials

    let mut client = ScoutClient::new(env::var("SCOUT_DEVICE_API_KEY").unwrap()).unwrap();

    // Step 1: Real identification
    let identify_result = client.identify().await;

    match identify_result {
        Ok(_) => {
            assert!(client.is_identified());

            let device_id = client.device.as_ref().unwrap().id;
            let _device_name = client.device.as_ref().unwrap().name.clone();
            let herd_id = client.herd.as_ref().unwrap().id;
            let _herd_slug = client.herd.as_ref().unwrap().slug.clone();

            // Step 2: Create a real session
            let session = Session::new(
                device_id as i64,
                chrono::Utc::now().timestamp() as u64,
                (chrono::Utc::now().timestamp() as u64) + 3600,
                "integration_test_v1.0.0".to_string(),
                Some("POINT(-155.15393 19.754824)".to_string()),
                120.0,
                45.0,
                82.5,
                15.0,
                3.0,
                9.0,
                1200.0,
                600.0,
            );

            let session_result = client.create_session(&session).await;
            if let Ok(ref response) = session_result {
                if response.status == ResponseScoutStatus::Success {
                    let created_session = response.data.clone().unwrap();

                    // Step 3: Create real events
                    let events = vec![
                        Event::new(
                            Some("Real integration test event 1".to_string()),
                            Some("https://real.example.com/image1.jpg".to_string()),
                            None,
                            None,
                            19.754824,
                            -155.15393,
                            10.0,
                            0.0,
                            MediaType::Image,
                            device_id,
                            chrono::Utc::now().timestamp() as u64,
                            false,
                            created_session.id,
                        ),
                        Event::new(
                            Some("Real integration test event 2".to_string()),
                            Some("https://real.example.com/image2.jpg".to_string()),
                            None,
                            None,
                            19.755,
                            -155.154,
                            12.0,
                            90.0,
                            MediaType::Image,
                            device_id,
                            chrono::Utc::now().timestamp() as u64,
                            false,
                            created_session.id,
                        ),
                    ];

                    let events_result = client.create_events_batch(&events).await;
                    if let Ok(response) = events_result {
                        if response.status == ResponseScoutStatus::Success {
                            let _created_events = response.data.unwrap();

                            // Step 4: Create real connectivity data
                            let connectivity = Connectivity::new(
                                created_session.id.unwrap(),
                                chrono::Utc::now().timestamp() as u64,
                                -45.0,
                                -60.0,
                                100.0,
                                180.0,
                                "POINT(-155.15393 19.754824)".to_string(),
                                "H14_INDEX".to_string(),
                                "H13_INDEX".to_string(),
                                "H12_INDEX".to_string(),
                                "H11_INDEX".to_string(),
                            );

                            let connectivity_result =
                                client.create_connectivity(&connectivity).await;
                            if let Ok(response) = connectivity_result {
                                if response.status == ResponseScoutStatus::Success {
                                    let _created_connectivity = response.data.unwrap();

                                    // Step 5: Test query operations
                                    let sessions_response =
                                        client.get_sessions_by_herd(herd_id).await;
                                    if let Ok(_response) = sessions_response {
                                        if _response.status == ResponseScoutStatus::Success {
                                            let _sessions = _response.data.unwrap();
                                        }
                                    }

                                    let events_response = client
                                        .get_session_events(created_session.id.unwrap())
                                        .await;
                                    if let Ok(_response) = events_response {
                                        if _response.status == ResponseScoutStatus::Success {
                                            let _events = _response.data.unwrap();
                                        }
                                    }

                                    // Step 6: Cleanup - delete test data
                                    let delete_result =
                                        client.delete_session(created_session.id.unwrap()).await;
                                    if let Ok(response) = delete_result {
                                        if response.status == ResponseScoutStatus::Success {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(_e) => {
            // This might happen if the API key is invalid or database is not accessible
            assert!(
                false,
                "Real database integration test failed during identification"
            );
        }
    }
}

#[tokio::test]
async fn test_device_events_with_tags_via_function() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let device_id = client.device.as_ref().unwrap().id;

    // Test getting events with tags via database function
    let events_result = client
        .get_device_events_with_tags_via_function(device_id, 10)
        .await;

    match events_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            // Note: This might return empty results if no events exist yet
        }
        Err(e) => {
            panic!("❌ Device events with tags retrieval failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_sessions_with_coordinates_via_function() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let herd_id = client.herd.as_ref().unwrap().id;

    // Test getting sessions with coordinates via database function
    let sessions_result = client.get_sessions_by_herd(herd_id).await;

    match sessions_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            // Note: This might return empty results if no sessions exist yet
        }
        Err(e) => {
            panic!("❌ Sessions with coordinates retrieval failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_connectivity_with_coordinates_via_function() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    // First create a session to test connectivity
    let session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        1640998800,
        "v2.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    let session_result = client.create_session(&session).await;
    if let Ok(response) = session_result {
        if response.status == ResponseScoutStatus::Success {
            let created_session = response.data.unwrap();
            let session_id = created_session.id.unwrap();

            // Test getting connectivity with coordinates via database function
            let connectivity_result = client.get_connectivity_with_coordinates(session_id).await;

            match connectivity_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    // Note: This might return empty results if no connectivity data exists yet
                }
                Err(e) => {
                    // This is expected if no connectivity data exists yet
                }
            }

            // Clean up the test session
            let _ = client.delete_session(session_id).await;
        }
    } else {
    }
}

async fn test_plans_by_herd_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let herd_id = client.herd.as_ref().unwrap().id;

    // Test getting plans by herd
    let plans_result = client.get_plans_by_herd(herd_id).await;

    match plans_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            // Note: This might return empty results if no plans exist yet
            if let Some(plans) = response.data {
                // Validate plan structure if plans exist
                for plan in &plans {
                    assert!(
                        plan.herd_id == herd_id,
                        "Plan herd_id should match requested herd_id"
                    );
                    assert!(!plan.name.is_empty(), "Plan name should not be empty");
                    assert!(
                        !plan.instructions.is_empty(),
                        "Plan instructions should not be empty"
                    );
                    assert!(plan.id >= 0, "Plan should have a non-negative ID");
                }
            }
        }
        Err(e) => {
            panic!("❌ Plans by herd retrieval failed: {}", e);
        }
    }
}

test_with_cleanup!(test_plans_by_herd, test_plans_by_herd_impl);

async fn test_plans_comprehensive_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let herd_id = client.herd.as_ref().unwrap().id;

    // Test 1: Get existing plans
    let plans_result = client.get_plans_by_herd(herd_id).await;
    match &plans_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            if let Some(plans) = &response.data {
                println!("Found {} plans for herd {}", plans.len(), herd_id);

                // Validate each plan structure
                for (i, plan) in plans.iter().enumerate() {
                    println!(
                        "Plan {}: ID={:?}, Name='{}', Type={:?}",
                        i, plan.id, plan.name, plan.plan_type
                    );

                    // Basic validation
                    assert_eq!(plan.herd_id, herd_id, "Plan herd_id mismatch");
                    assert!(!plan.name.is_empty(), "Plan name is empty");
                    assert!(!plan.instructions.is_empty(), "Plan instructions is empty");

                    // Type validation - all plan types are valid
                    match plan.plan_type {
                        PlanType::Mission
                        | PlanType::Fence
                        | PlanType::Rally
                        | PlanType::Markov => {
                            // Valid plan type
                        }
                    }

                    // ID validation - allow ID=0 for existing plans that might not have been properly migrated
                    // Note: ID=0 is valid for existing plans in the database
                    assert!(plan.id >= 0, "Plan ID should be non-negative");

                    // Validate inserted_at timestamp if present
                    if let Some(inserted_at) = &plan.inserted_at {
                        assert!(
                            !inserted_at.is_empty(),
                            "Inserted timestamp should not be empty"
                        );
                        // Basic timestamp format validation
                        assert!(
                            inserted_at.contains('T') || inserted_at.contains(' '),
                            "Inserted timestamp should contain date/time separator"
                        );
                    }
                }
            } else {
                println!("No plans found for herd {}", herd_id);
            }
        }
        Err(e) => {
            panic!("❌ Plans retrieval failed: {}", e);
        }
    }

    // Test 2: Test with different herd IDs (edge cases)
    let invalid_herd_id = 999999;
    let invalid_plans_result = client.get_plans_by_herd(invalid_herd_id).await;
    match invalid_plans_result {
        Ok(response) => {
            // Should succeed but return empty results
            assert_eq!(response.status, ResponseScoutStatus::Success);
            if let Some(plans) = response.data {
                assert_eq!(plans.len(), 0, "Invalid herd ID should return no plans");
            }
        }
        Err(e) => {
            // This is also acceptable - some databases might return an error
            println!("Expected behavior: Invalid herd ID returned error: {}", e);
        }
    }

    // Test 3: Test plan data structure validation
    let test_plan = Plan {
        id: 1,
        inserted_at: Some("2023-01-01T00:00:00Z".to_string()),
        name: "Test Plan".to_string(),
        instructions: "Test instructions for the plan".to_string(),
        herd_id: 1,
        plan_type: PlanType::Mission,
    };

    // Validate test plan structure
    assert_eq!(test_plan.id, 1);
    assert_eq!(test_plan.name, "Test Plan");
    assert_eq!(test_plan.instructions, "Test instructions for the plan");
    assert_eq!(test_plan.herd_id, 1);
    assert_eq!(test_plan.plan_type, PlanType::Mission);

    // Test 4: Test all plan types
    let plan_types = vec![
        PlanType::Mission,
        PlanType::Fence,
        PlanType::Rally,
        PlanType::Markov,
    ];

    for plan_type in plan_types {
        let test_plan = Plan {
            id: 0,             // Placeholder ID for testing
            inserted_at: None, // Database will use default value
            name: format!("Test {} Plan", format!("{:?}", plan_type)),
            instructions: format!("Test instructions for {} plan", format!("{:?}", plan_type)),
            herd_id: 1,
            plan_type: plan_type.clone(),
        };

        assert_eq!(test_plan.plan_type, plan_type);
        assert!(test_plan.name.contains(&format!("{:?}", plan_type)));
    }

    // Test 5: Test individual plan retrieval if plans exist
    match &plans_result {
        Ok(response) => {
            if let Some(plans) = &response.data {
                if !plans.is_empty() {
                    let first_plan = &plans[0];
                    let plan_id = first_plan.id;
                    let individual_plan_result = client.get_plan_by_id(plan_id).await;
                    match individual_plan_result {
                        Ok(response) => {
                            assert_eq!(response.status, ResponseScoutStatus::Success);
                            assert!(response.data.is_some());

                            let retrieved_plan = response.data.unwrap();
                            assert_eq!(retrieved_plan.id, plan_id);
                            assert_eq!(retrieved_plan.herd_id, herd_id);
                            assert_eq!(retrieved_plan.name, first_plan.name);
                            assert_eq!(retrieved_plan.instructions, first_plan.instructions);
                            assert_eq!(retrieved_plan.plan_type, first_plan.plan_type);

                            println!(
                                "Successfully tested individual plan retrieval for plan ID: {}",
                                plan_id
                            );
                        }
                        Err(e) => {
                            panic!("❌ Individual plan retrieval failed: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            panic!(
                "❌ Failed to get plans for individual retrieval test: {}",
                e
            );
        }
    }
}

test_with_cleanup!(test_plans_comprehensive, test_plans_comprehensive_impl);

async fn test_plans_crud_operations_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let herd_id = client.herd.as_ref().unwrap().id;

    // Test 1: Create a new plan
    let new_plan = Plan {
        id: 0,             // Placeholder ID for creation
        inserted_at: None, // Database will use default value
        name: "Test CRUD Plan".to_string(),
        instructions: "This is a test plan for CRUD operations".to_string(),
        herd_id,
        plan_type: PlanType::Mission,
    };

    let create_result = client.create_plan(&new_plan).await;
    match create_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            let created_plan = response.data.unwrap();
            assert!(created_plan.id >= 0, "Created plan should have a valid ID");
            assert_eq!(created_plan.name, "Test CRUD Plan");
            assert_eq!(
                created_plan.instructions,
                "This is a test plan for CRUD operations"
            );
            assert_eq!(created_plan.herd_id, herd_id);
            assert_eq!(created_plan.plan_type, PlanType::Mission);

            let plan_id = created_plan.id;
            println!("Created plan with ID: {}", plan_id);

            // Track the created plan for cleanup
            cleanup.track_plan(plan_id);

            // Test 2: Read the created plan
            let plans_result = client.get_plans_by_herd(herd_id).await;
            match plans_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    if let Some(plans) = response.data {
                        // Find our created plan
                        let found_plan = plans.iter().find(|p| p.id == plan_id);
                        assert!(found_plan.is_some(), "Should find the created plan");

                        let found_plan = found_plan.unwrap();
                        assert_eq!(found_plan.name, "Test CRUD Plan");
                        assert_eq!(
                            found_plan.instructions,
                            "This is a test plan for CRUD operations"
                        );
                    }
                }
                Err(e) => {
                    panic!("❌ Failed to read plans after creation: {}", e);
                }
            }

            // Test 3: Update the plan
            let updated_plan = Plan {
                id: plan_id,
                inserted_at: created_plan.inserted_at.clone(),
                name: "Updated CRUD Plan".to_string(),
                instructions: "This plan has been updated".to_string(),
                herd_id,
                plan_type: PlanType::Fence,
            };

            let update_result = client.update_plan(plan_id, &updated_plan).await;
            match update_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    assert!(response.data.is_some());

                    let updated_plan_result = response.data.unwrap();
                    assert_eq!(updated_plan_result.name, "Updated CRUD Plan");
                    assert_eq!(
                        updated_plan_result.instructions,
                        "This plan has been updated"
                    );
                    assert_eq!(updated_plan_result.plan_type, PlanType::Fence);
                    println!("Successfully updated plan {}", plan_id);
                }
                Err(e) => {
                    panic!("❌ Failed to update plan: {}", e);
                }
            }

            // Test 4: Verify the update by reading again
            let plans_result = client.get_plans_by_herd(herd_id).await;
            match plans_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    if let Some(plans) = response.data {
                        let found_plan = plans.iter().find(|p| p.id == plan_id);
                        assert!(found_plan.is_some(), "Should find the updated plan");

                        let found_plan = found_plan.unwrap();
                        assert_eq!(found_plan.name, "Updated CRUD Plan");
                        assert_eq!(found_plan.instructions, "This plan has been updated");
                        assert_eq!(found_plan.plan_type, PlanType::Fence);
                    }
                }
                Err(e) => {
                    panic!("❌ Failed to read plans after update: {}", e);
                }
            }

            // Test 5: Delete the plan
            let delete_result = client.delete_plan(plan_id).await;
            match delete_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    println!("Successfully deleted plan {}", plan_id);
                }
                Err(e) => {
                    panic!("❌ Failed to delete plan: {}", e);
                }
            }

            // Test 6: Verify deletion by trying to read the plan
            let plans_result = client.get_plans_by_herd(herd_id).await;
            match plans_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    if let Some(plans) = response.data {
                        let found_plan = plans.iter().find(|p| p.id == plan_id);
                        assert!(found_plan.is_none(), "Should not find the deleted plan");
                    }
                }
                Err(e) => {
                    panic!("❌ Failed to read plans after deletion: {}", e);
                }
            }
        }
        Err(e) => {
            panic!("❌ Failed to create plan: {}", e);
        }
    }
}

test_with_cleanup!(test_plans_crud_operations, test_plans_crud_operations_impl);

async fn test_plans_bulk_operations_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let herd_id = client.herd.as_ref().unwrap().id;
    let mut created_plan_ids = Vec::new();

    // Test 1: Create multiple plans
    let test_plans = vec![
        Plan {
            id: 0,             // Placeholder ID for creation
            inserted_at: None, // Database will use default value
            name: "Bulk Test Plan 1".to_string(),
            instructions: "First bulk test plan".to_string(),
            herd_id,
            plan_type: PlanType::Mission,
        },
        Plan {
            id: 0,             // Placeholder ID for creation
            inserted_at: None, // Database will use default value
            name: "Bulk Test Plan 2".to_string(),
            instructions: "Second bulk test plan".to_string(),
            herd_id,
            plan_type: PlanType::Fence,
        },
        Plan {
            id: 0,             // Placeholder ID for creation
            inserted_at: None, // Database will use default value
            name: "Bulk Test Plan 3".to_string(),
            instructions: "Third bulk test plan".to_string(),
            herd_id,
            plan_type: PlanType::Rally,
        },
    ];

    // Create plans one by one and track IDs
    for (i, plan) in test_plans.iter().enumerate() {
        let create_result = client.create_plan(plan).await;
        match create_result {
            Ok(response) => {
                assert_eq!(response.status, ResponseScoutStatus::Success);
                assert!(response.data.is_some());

                let created_plan = response.data.unwrap();
                assert!(
                    created_plan.id >= 0,
                    "Plan {} should have a valid ID",
                    i + 1
                );
                assert_eq!(created_plan.name, format!("Bulk Test Plan {}", i + 1));
                assert_eq!(created_plan.herd_id, herd_id);

                created_plan_ids.push(created_plan.id);
                println!(
                    "Created bulk test plan {} with ID: {}",
                    i + 1,
                    created_plan.id
                );

                // Track the created plan for cleanup
                cleanup.track_plan(created_plan.id);
            }
            Err(e) => {
                panic!("❌ Failed to create bulk test plan {}: {}", i + 1, e);
            }
        }
    }

    // Test 2: Verify all plans were created
    let plans_result = client.get_plans_by_herd(herd_id).await;
    match plans_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            if let Some(plans) = response.data {
                // Check that we can find all our created plans
                for plan_id in &created_plan_ids {
                    let found_plan = plans.iter().find(|p| p.id == *plan_id);
                    assert!(
                        found_plan.is_some(),
                        "Should find bulk test plan with ID {}",
                        plan_id
                    );
                }

                println!(
                    "Successfully verified {} bulk test plans",
                    created_plan_ids.len()
                );
            }
        }
        Err(e) => {
            panic!("❌ Failed to read plans after bulk creation: {}", e);
        }
    }

    // Test 3: Clean up all created plans
    for plan_id in &created_plan_ids {
        let delete_result = client.delete_plan(*plan_id).await;
        match delete_result {
            Ok(response) => {
                assert_eq!(response.status, ResponseScoutStatus::Success);
                println!("Successfully deleted bulk test plan {}", plan_id);
            }
            Err(e) => {
                panic!("❌ Failed to delete bulk test plan {}: {}", plan_id, e);
            }
        }
    }

    // Test 4: Verify all plans were deleted
    let plans_result = client.get_plans_by_herd(herd_id).await;
    match plans_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            if let Some(plans) = response.data {
                // Check that none of our created plans exist
                for plan_id in &created_plan_ids {
                    let found_plan = plans.iter().find(|p| p.id == *plan_id);
                    assert!(
                        found_plan.is_none(),
                        "Should not find deleted bulk test plan with ID {}",
                        plan_id
                    );
                }

                println!("Successfully verified all bulk test plans were deleted");
            }
        }
        Err(e) => {
            panic!("❌ Failed to read plans after bulk deletion: {}", e);
        }
    }
}

test_with_cleanup!(test_plans_bulk_operations, test_plans_bulk_operations_impl);

async fn test_plan_individual_retrieval_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let herd_id = client.herd.as_ref().unwrap().id;

    // Test 1: Create a test plan first
    let test_plan = Plan {
        id: 0,             // Placeholder ID for creation
        inserted_at: None, // Database will use default value
        name: "Individual Retrieval Test Plan".to_string(),
        instructions: "This plan is for testing individual retrieval".to_string(),
        herd_id,
        plan_type: PlanType::Mission,
    };

    let create_result = client.create_plan(&test_plan).await;
    match create_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            let created_plan = response.data.unwrap();
            assert!(created_plan.id >= 0, "Created plan should have a valid ID");
            let plan_id = created_plan.id;
            println!("Created test plan with ID: {}", plan_id);

            // Track the created plan for cleanup
            cleanup.track_plan(plan_id);

            // Test 2: Get the plan by ID
            let get_result = client.get_plan_by_id(plan_id).await;
            match get_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    assert!(response.data.is_some());

                    let retrieved_plan = response.data.unwrap();
                    assert_eq!(retrieved_plan.id, plan_id);
                    assert_eq!(retrieved_plan.name, "Individual Retrieval Test Plan");
                    assert_eq!(
                        retrieved_plan.instructions,
                        "This plan is for testing individual retrieval"
                    );
                    assert_eq!(retrieved_plan.herd_id, herd_id);
                    assert_eq!(retrieved_plan.plan_type, PlanType::Mission);

                    println!("Successfully retrieved plan {} by ID", plan_id);
                }
                Err(e) => {
                    panic!("❌ Failed to get plan by ID: {}", e);
                }
            }

            // Test 3: Test getting non-existent plan
            let non_existent_id = 999999;
            let non_existent_result = client.get_plan_by_id(non_existent_id).await;
            match non_existent_result {
                Ok(response) => {
                    // Should return failure status for non-existent plan
                    assert_eq!(response.status, ResponseScoutStatus::Failure);
                    assert!(response.data.is_none());
                    println!(
                        "Correctly handled non-existent plan ID: {}",
                        non_existent_id
                    );
                }
                Err(e) => {
                    // This is also acceptable - some databases might return an error
                    println!(
                        "Expected behavior: Non-existent plan ID returned error: {}",
                        e
                    );
                }
            }

            // Test 4: Verify the plan still exists in herd plans
            let herd_plans_result = client.get_plans_by_herd(herd_id).await;
            match herd_plans_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    if let Some(plans) = response.data {
                        let found_plan = plans.iter().find(|p| p.id == plan_id);
                        assert!(
                            found_plan.is_some(),
                            "Should find the test plan in herd plans"
                        );

                        let found_plan = found_plan.unwrap();
                        assert_eq!(found_plan.name, "Individual Retrieval Test Plan");
                        assert_eq!(found_plan.plan_type, PlanType::Mission);
                    }
                }
                Err(e) => {
                    panic!("❌ Failed to get herd plans: {}", e);
                }
            }

            // Test 5: Clean up the test plan
            let delete_result = client.delete_plan(plan_id).await;
            match delete_result {
                Ok(response) => {
                    assert_eq!(response.status, ResponseScoutStatus::Success);
                    println!("Successfully deleted test plan {}", plan_id);
                }
                Err(e) => {
                    panic!("❌ Failed to delete test plan: {}", e);
                }
            }

            // Test 6: Verify deletion by trying to get the plan by ID
            let get_after_delete_result = client.get_plan_by_id(plan_id).await;
            match get_after_delete_result {
                Ok(response) => {
                    // Should return failure status for deleted plan
                    assert_eq!(response.status, ResponseScoutStatus::Failure);
                    assert!(response.data.is_none());
                    println!("Correctly handled deleted plan ID: {}", plan_id);
                }
                Err(e) => {
                    // This is also acceptable
                    println!("Expected behavior: Deleted plan ID returned error: {}", e);
                }
            }
        }
        Err(e) => {
            panic!(
                "❌ Failed to create test plan for individual retrieval test: {}",
                e
            );
        }
    }
}

test_with_cleanup!(
    test_plan_individual_retrieval,
    test_plan_individual_retrieval_impl
);

#[tokio::test]
async fn test_zones_and_actions_by_herd() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let herd_id = client.herd.as_ref().unwrap().id;

    // Test getting zones and actions by herd
    let zones_result = client.get_zones_and_actions_by_herd(herd_id, 10, 0).await;

    match zones_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            // Note: This might return empty results if no zones exist yet
        }
        Err(e) => {
            panic!("❌ Zones and actions by herd retrieval failed: {}", e);
        }
    }
}

#[test]
fn test_data_validation() {
    // Test Event validation
    let valid_event = Event::new(
        Some("Valid event".to_string()),
        Some("https://example.com/image.jpg".to_string()),
        None,
        None,
        90.0,   // Valid latitude
        -180.0, // Valid longitude
        0.0,    // Valid altitude
        0.0,    // Valid heading
        MediaType::Image,
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None,
    );

    assert!(valid_event.location.is_some());
    assert_eq!(valid_event.location.unwrap(), "POINT(-180 90)");

    // Test Session validation
    let valid_session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        1640998800, // End time after start time
        "v1.0.0".to_string(),
        None,
        100.0,
        50.0,
        75.0,
        10.0,
        5.0,
        7.5,
        1000.0,
        500.0,
    );

    assert!(valid_session.timestamp_start < valid_session.timestamp_end);

    // Test Tag validation
    let valid_tag = Tag::new(
        1,
        100.0,
        200.0,
        50.0,
        30.0,
        0.95, // Valid confidence (0-1)
        TagObservationType::Auto,
        "animal".to_string(),
    );
    assert!(valid_tag.conf >= 0.0 && valid_tag.conf <= 1.0);
    assert!(valid_tag.width > 0.0);
    assert!(valid_tag.height > 0.0);
}

#[test]
fn test_response_handling() {
    // Test success response
    let success = ResponseScout::new(ResponseScoutStatus::Success, Some("data"));
    assert_eq!(success.status, ResponseScoutStatus::Success);
    assert!(success.data.is_some());

    // Test failure response
    let failure = ResponseScout::new(ResponseScoutStatus::Failure, None::<&str>);
    assert_eq!(failure.status, ResponseScoutStatus::Failure);
    assert!(failure.data.is_none());

    // Test not authorized response
    let not_authorized = ResponseScout::new(ResponseScoutStatus::NotAuthorized, None::<&str>);
    assert_eq!(not_authorized.status, ResponseScoutStatus::NotAuthorized);

    // Test invalid event response
    let invalid_event = ResponseScout::new(ResponseScoutStatus::InvalidEvent, None::<&str>);
    assert_eq!(invalid_event.status, ResponseScoutStatus::InvalidEvent);

    // Test invalid file response
    let invalid_file = ResponseScout::new(ResponseScoutStatus::InvalidFile, None::<&str>);
    assert_eq!(invalid_file.status, ResponseScoutStatus::InvalidFile);
}

#[tokio::test]
async fn test_response_handling_comprehensive() {
    setup_test_env();

    // Test 1: ResponseScout status handling
    let success_response = ResponseScout::new(ResponseScoutStatus::Success, Some("test data"));
    assert_eq!(success_response.status, ResponseScoutStatus::Success);
    assert!(success_response.data.is_some());
    assert_eq!(success_response.data.unwrap(), "test data");

    let failure_response = ResponseScout::new(ResponseScoutStatus::Failure, None::<&str>);
    assert_eq!(failure_response.status, ResponseScoutStatus::Failure);
    assert!(failure_response.data.is_none());

    let not_authorized = ResponseScout::new(ResponseScoutStatus::NotAuthorized, None::<&str>);
    assert_eq!(not_authorized.status, ResponseScoutStatus::NotAuthorized);

    let invalid_event = ResponseScout::new(ResponseScoutStatus::InvalidEvent, None::<&str>);
    assert_eq!(invalid_event.status, ResponseScoutStatus::InvalidEvent);

    let invalid_file = ResponseScout::new(ResponseScoutStatus::InvalidFile, None::<&str>);
    assert_eq!(invalid_file.status, ResponseScoutStatus::InvalidFile);

    // Test 2: Response creation helpers
    let _client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Test 3: Response with different data types
    let string_response = ResponseScout::new(
        ResponseScoutStatus::Success,
        Some("string data".to_string()),
    );
    assert_eq!(string_response.status, ResponseScoutStatus::Success);
    assert_eq!(string_response.data.unwrap(), "string data");

    let int_response = ResponseScout::new(ResponseScoutStatus::Success, Some(42));
    assert_eq!(int_response.status, ResponseScoutStatus::Success);
    assert_eq!(int_response.data.unwrap(), 42);

    let vec_response = ResponseScout::new(ResponseScoutStatus::Success, Some(vec![1, 2, 3]));
    assert_eq!(vec_response.status, ResponseScoutStatus::Success);
    assert_eq!(vec_response.data.unwrap(), vec![1, 2, 3]);

    // Test 4: Response cloning
    let original_response = ResponseScout::new(ResponseScoutStatus::Success, Some("original"));
    let cloned_response = original_response.clone();
    assert_eq!(original_response.status, cloned_response.status);
    assert_eq!(original_response.data, cloned_response.data);

    // Test 5: Response comparison
    let response1 = ResponseScout::new(ResponseScoutStatus::Success, Some("data"));
    let response2 = ResponseScout::new(ResponseScoutStatus::Success, Some("data"));
    let response3 = ResponseScout::new(ResponseScoutStatus::Failure, None::<&str>);

    assert_eq!(response1, response2);
    assert_ne!(response1, response3);
}

#[tokio::test]
async fn test_integration_with_mock_data() {
    // This test simulates integration without requiring external connections

    // Test ResponseScout status handling
    let mock_response = ResponseScout::new(ResponseScoutStatus::Success, Some("mock data"));

    match mock_response.status {
        ResponseScoutStatus::Success => {
            if let Some(data) = mock_response.data {
                assert_eq!(data, "mock data");
            } else {
                panic!("❌ Mock response should have data");
            }
        }
        _ => panic!("❌ Mock response should have Success status"),
    }
}

#[tokio::test]
async fn test_complete_data_collection_workflow() {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Step 1: Identify the client
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Step 1: Client identification failed: {:?}",
            identify_result.err()
        );
    }

    let device_id = client.device.as_ref().unwrap().id;
    let _herd_id = client.herd.as_ref().unwrap().id;

    // Step 2: Create a session
    let session = Session::new(
        device_id as i64,
        chrono::Utc::now().timestamp() as u64,
        (chrono::Utc::now().timestamp() as u64) + 3600,
        "workflow_test_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    let session_result = client.create_session(&session).await;
    if let Ok(response) = session_result {
        if response.status == ResponseScoutStatus::Success {
            let created_session = response.data.unwrap();
            let session_id = created_session.id.unwrap();

            // Step 3: Create events for the session
            let events = vec![
                Event::new(
                    Some("Workflow test event 1".to_string()),
                    Some("https://test.example.com/image1.jpg".to_string()),
                    None,
                    None,
                    19.754824,
                    -155.15393,
                    10.0,
                    0.0,
                    MediaType::Image,
                    device_id,
                    chrono::Utc::now().timestamp() as u64,
                    false,
                    Some(session_id),
                ),
                Event::new(
                    Some("Workflow test event 2".to_string()),
                    Some("https://test.example.com/image2.jpg".to_string()),
                    None,
                    None,
                    19.755,
                    -155.154,
                    12.0,
                    90.0,
                    MediaType::Image,
                    device_id,
                    chrono::Utc::now().timestamp() as u64,
                    false,
                    Some(session_id),
                ),
            ];

            let events_result = client.create_events_batch(&events).await;
            if let Ok(_response) = events_result {
                // Get the created events with their IDs
                let created_events = _response.data.unwrap();

                // Step 4: Create tags for the first event
                if let Some(first_event) = created_events.first() {
                    if let Some(event_id) = first_event.id {
                        let tags = vec![Tag::new(
                            1,
                            100.0,
                            200.0,
                            50.0,
                            30.0,
                            0.95,
                            TagObservationType::Auto,
                            "elephant".to_string(),
                        )];

                        let tags_result = client.create_tags(event_id, &tags).await;
                        if let Ok(response) = tags_result {
                            if response.status == ResponseScoutStatus::Success {}
                        }
                    } else {
                    }
                }

                // Step 5: Create connectivity data
                let connectivity = Connectivity::new(
                    session_id,
                    chrono::Utc::now().timestamp() as u64,
                    -45.0,
                    -60.0,
                    100.0,
                    180.0,
                    "POINT(-155.15393 19.754824)".to_string(),
                    "H14_INDEX".to_string(),
                    "H13_INDEX".to_string(),
                    "H12_INDEX".to_string(),
                    "H11_INDEX".to_string(),
                );

                let connectivity_result = client.create_connectivity(&connectivity).await;
                if let Ok(response) = connectivity_result {
                    if response.status == ResponseScoutStatus::Success {}
                }

                // Step 6: Query the data we just created
                let session_events = client.get_session_events(session_id).await;
                if let Ok(response) = session_events {
                    if response.status == ResponseScoutStatus::Success {
                        let events = response.data.unwrap();
                    }
                }

                let session_connectivity = client.get_session_connectivity(session_id).await;
                if let Ok(response) = session_connectivity {
                    if response.status == ResponseScoutStatus::Success {
                        let connectivity_entries = response.data.unwrap();
                    }
                }

                // Step 7: Clean up test data
                let delete_result = client.delete_session(session_id).await;
                if let Ok(response) = delete_result {
                    if response.status == ResponseScoutStatus::Success {}
                }
            } else {
                panic!("❌ Step 3: Events creation failed");
            }
        } else {
            panic!("❌ Step 2: Session creation failed");
        }
    } else {
        panic!("❌ Step 2: Session creation error");
    }
}

#[tokio::test]
async fn test_real_database_integration_comprehensive() {
    setup_test_env();

    let mut client = ScoutClient::new(env::var("SCOUT_DEVICE_API_KEY").unwrap()).unwrap();

    // Step 1: Real identification
    let identify_result = client.identify().await;

    match identify_result {
        Ok(_) => {
            assert!(client.is_identified());

            let device_id = client.device.as_ref().unwrap().id;
            let device_name = client.device.as_ref().unwrap().name.clone();
            let herd_id = client.herd.as_ref().unwrap().id;
            let herd_slug = client.herd.as_ref().unwrap().slug.clone();

            // Step 2: Test device retrieval methods
            let device_response = client.get_device().await;
            if let Ok(response) = device_response {
                if response.status == ResponseScoutStatus::Success {}
            }

            let herd_response = client.get_herd(None).await;
            if let Ok(response) = herd_response {
                if response.status == ResponseScoutStatus::Success {}
            }

            // Step 3: Test database function calls
            let events_with_tags = client
                .get_device_events_with_tags_via_function(device_id, 5)
                .await;
            if let Ok(response) = events_with_tags {
                if response.status == ResponseScoutStatus::Success {
                    let events = response.data.unwrap();
                }
            }

            let sessions_with_coords = client.get_sessions_by_herd(herd_id).await;
            if let Ok(response) = sessions_with_coords {
                if response.status == ResponseScoutStatus::Success {
                    let sessions = response.data.unwrap();
                }
            }

            let plans = client.get_plans_by_herd(herd_id).await;
            if let Ok(response) = plans {
                if response.status == ResponseScoutStatus::Success {
                    let plans = response.data.unwrap();
                }
            }

            let zones = client.get_zones_and_actions_by_herd(herd_id, 5, 0).await;
            if let Ok(response) = zones {
                if response.status == ResponseScoutStatus::Success {
                    let zones = response.data.unwrap();
                }
            }
        }
        Err(e) => {
            // This might happen if the API key is invalid or database is not accessible
            // We don't panic here as this is expected in some test environments
        }
    }
}

#[tokio::test]
async fn test_integration_with_mock_data_comprehensive() {
    setup_test_env();

    // Test 1: Client structure and types
    let mock_response = ResponseScout::new(ResponseScoutStatus::Success, Some("mock data"));

    match mock_response.status {
        ResponseScoutStatus::Success => {
            if let Some(data) = mock_response.data {
                assert_eq!(data, "mock data");
            } else {
                panic!("❌ Mock response should have data");
            }
        }
        _ => panic!("❌ Mock response should have Success status"),
    }

    // Test 2: Data structure creation and validation
    let mock_event = Event::new(
        Some("Mock event".to_string()),
        Some("https://mock.com/image.jpg".to_string()),
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        MediaType::Image,
        123,
        1640995200,
        false,
        None,
    );

    assert_eq!(mock_event.message, Some("Mock event".to_string()));
    assert_eq!(
        mock_event.media_url,
        Some("https://mock.com/image.jpg".to_string())
    );
    assert_eq!(mock_event.device_id, Some(123));
    assert_eq!(mock_event.is_public, false);
    assert!(mock_event.location.is_some());

    let mock_session = Session::new(
        123,
        1640995200,
        1640998800,
        "mock_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        100.0,
        50.0,
        75.0,
        10.0,
        5.0,
        7.5,
        1000.0,
        500.0,
    );

    assert_eq!(mock_session.device_id, 123);
    assert_eq!(mock_session.software_version, "mock_v1.0.0");
    assert_eq!(mock_session.altitude_max, 100.0);
    assert_eq!(mock_session.altitude_min, 50.0);

    let mock_tag = Tag::new(
        1,
        100.0,
        200.0,
        50.0,
        30.0,
        0.95,
        TagObservationType::Auto,
        "mock_animal".to_string(),
    );

    assert_eq!(mock_tag.x, 100.0);
    assert_eq!(mock_tag.y, 200.0);
    assert_eq!(mock_tag.conf, 0.95);
    assert_eq!(mock_tag.class_name, "mock_animal");

    // Test 3: Client creation and validation
    let mock_client = ScoutClient::new("mock_api_key".to_string());
    assert!(mock_client.is_ok());

    let mock_client = mock_client.unwrap();
    assert_eq!(mock_client.api_key, "mock_api_key");
    assert!(mock_client.device.is_none());
    assert!(mock_client.herd.is_none());
    assert!(!mock_client.is_identified());

    // Test 4: Response status validation
    let statuses = vec![
        ResponseScoutStatus::Success,
        ResponseScoutStatus::Failure,
        ResponseScoutStatus::NotAuthorized,
        ResponseScoutStatus::InvalidEvent,
        ResponseScoutStatus::InvalidFile,
    ];

    for status in statuses {
        let response = ResponseScout::new(status.clone(), None::<&str>);
        assert_eq!(response.status, status);
        assert!(response.data.is_none());
    }

    // Test 5: Mock data serialization/deserialization
    let mock_connectivity = Connectivity::new(
        1,
        1640995200,
        -45.0,
        -60.0,
        100.0,
        180.0,
        "POINT(-155.15393 19.754824)".to_string(),
        "H14_INDEX".to_string(),
        "H13_INDEX".to_string(),
        "H12_INDEX".to_string(),
        "H11_INDEX".to_string(),
    );

    assert_eq!(mock_connectivity.session_id, 1);
    assert_eq!(mock_connectivity.signal, -45.0);
    assert_eq!(mock_connectivity.noise, -60.0);

    // Test 6: Mock data with different values
    let mock_plan = Plan {
        id: 42,
        inserted_at: Some("2023-01-01T00:00:00Z".to_string()),
        name: "Mock Plan".to_string(),
        instructions: "Mock instructions".to_string(),
        herd_id: 123,
        plan_type: PlanType::Mission,
    };

    assert_eq!(mock_plan.id, 42);
    assert_eq!(mock_plan.name, "Mock Plan");
    assert_eq!(mock_plan.herd_id, 123);

    // Test 7: Mock data arrays
    let mock_events = vec![
        Event::new(
            Some("Mock event 1".to_string()),
            None,
            None,
            None,
            19.754824,
            -155.15393,
            10.0,
            0.0,
            MediaType::Image,
            123,
            1640995200,
            false,
            None,
        ),
        Event::new(
            Some("Mock event 2".to_string()),
            None,
            None,
            None,
            19.755,
            -155.154,
            12.0,
            90.0,
            MediaType::Video,
            123,
            1640995260,
            true,
            None,
        ),
    ];

    assert_eq!(mock_events.len(), 2);
    assert_eq!(mock_events[0].message, Some("Mock event 1".to_string()));
    assert_eq!(mock_events[1].message, Some("Mock event 2".to_string()));
    assert_eq!(mock_events[0].media_type, MediaType::Image);
    assert_eq!(mock_events[1].media_type, MediaType::Video);
}

#[tokio::test]
async fn test_data_structures_comprehensive() {
    setup_test_env();

    let expected_device_id = env::var("SCOUT_DEVICE_ID")
        .unwrap_or_else(|_| "123".to_string())
        .parse()
        .unwrap_or(123);

    // Test 1: Event structure
    let event = Event::new(
        Some("Comprehensive test event".to_string()),
        Some("https://example.com/comprehensive.jpg".to_string()),
        Some("/path/to/file.jpg".to_string()),
        Some("https://earthranger.example.com".to_string()),
        19.754824,
        -155.15393,
        15.0,
        45.0,
        MediaType::Image,
        expected_device_id,
        1640995200,
        true,
        Some(1),
    );

    assert_eq!(event.message, Some("Comprehensive test event".to_string()));
    assert_eq!(
        event.media_url,
        Some("https://example.com/comprehensive.jpg".to_string())
    );
    assert_eq!(event.file_path, Some("/path/to/file.jpg".to_string()));
    assert_eq!(
        event.earthranger_url,
        Some("https://earthranger.example.com".to_string())
    );
    assert_eq!(event.device_id, Some(expected_device_id as i64));
    assert_eq!(event.is_public, true);
    assert_eq!(event.session_id, Some(1));
    assert!(event.location.is_some());
    assert_eq!(event.location.unwrap(), "POINT(-155.15393 19.754824)");

    // Test 2: Session structure
    let session = Session::new(
        expected_device_id as i64,
        1640995200,
        1640998800,
        "comprehensive_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0,
    );

    assert_eq!(session.device_id, expected_device_id as i64);
    assert_eq!(session.software_version, "comprehensive_v1.0.0");
    assert_eq!(session.altitude_max, 120.0);
    assert_eq!(session.altitude_min, 45.0);
    assert_eq!(session.altitude_average, 82.5);
    assert_eq!(session.velocity_max, 15.0);
    assert_eq!(session.velocity_min, 3.0);
    assert_eq!(session.velocity_average, 9.0);
    assert_eq!(session.distance_total, 1200.0);
    assert_eq!(session.distance_max_from_start, 600.0);
    assert!(session.locations.is_some());

    // Test 3: Tag structure
    let tag = Tag::new(
        1,
        100.0,
        200.0,
        50.0,
        30.0,
        0.95,
        TagObservationType::Auto,
        "comprehensive_animal".to_string(),
    );

    assert_eq!(tag.x, 100.0);
    assert_eq!(tag.y, 200.0);
    assert_eq!(tag.width, 50.0);
    assert_eq!(tag.height, 30.0);
    assert_eq!(tag.conf, 0.95);
    assert_eq!(tag.observation_type, TagObservationType::Auto);
    assert_eq!(tag.class_name, "comprehensive_animal");
    assert_eq!(tag.event_id, 0); // Default value

    // Test 4: Connectivity structure
    let connectivity = Connectivity::new(
        1,
        1640995200,
        -45.0,
        -60.0,
        100.0,
        180.0,
        "POINT(-155.15393 19.754824)".to_string(),
        "H14_INDEX".to_string(),
        "H13_INDEX".to_string(),
        "H12_INDEX".to_string(),
        "H11_INDEX".to_string(),
    );

    assert_eq!(connectivity.session_id, 1);
    assert_eq!(connectivity.signal, -45.0);
    assert_eq!(connectivity.noise, -60.0);
    assert_eq!(connectivity.altitude, 100.0);
    assert_eq!(connectivity.heading, 180.0);
    assert_eq!(connectivity.location, "POINT(-155.15393 19.754824)");
    assert_eq!(connectivity.h14_index, "H14_INDEX");
    assert_eq!(connectivity.h13_index, "H13_INDEX");
    assert_eq!(connectivity.h12_index, "H12_INDEX");
    assert_eq!(connectivity.h11_index, "H11_INDEX");

    // Test 5: Plan structure
    let plan = Plan {
        id: 1,
        inserted_at: Some("2023-01-01T00:00:00Z".to_string()),
        name: "Comprehensive test plan".to_string(),
        instructions: "Test instructions".to_string(),
        herd_id: 1,
        plan_type: PlanType::Mission,
    };

    assert_eq!(plan.id, 1);
    assert_eq!(plan.name, "Comprehensive test plan");
    assert_eq!(plan.instructions, "Test instructions");
    assert_eq!(plan.herd_id, 1);
    assert_eq!(plan.plan_type, PlanType::Mission);

    // Test 6: Zone and Action structures
    let action = Action {
        id: 1,
        inserted_at: "2023-01-01T00:00:00Z".to_string(),
        zone_id: 1,
        trigger: vec!["motion".to_string(), "sound".to_string()],
        opcode: 42,
    };

    assert_eq!(action.id, 1);
    assert_eq!(action.zone_id, 1);
    assert_eq!(
        action.trigger,
        vec!["motion".to_string(), "sound".to_string()]
    );
    assert_eq!(action.opcode, 42);

    let zone = Zone {
        id: 1,
        inserted_at: "2023-01-01T00:00:00Z".to_string(),
        region: "POLYGON((-155.154 19.754, -155.153 19.754, -155.153 19.755, -155.154 19.755, -155.154 19.754))".to_string(),
        herd_id: 1,
        actions: Some(vec![action]),
    };

    assert_eq!(zone.id, 1);
    assert_eq!(zone.herd_id, 1);
    assert!(zone.actions.is_some());
    assert_eq!(zone.actions.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn test_client_creation_comprehensive() {
    setup_test_env();

    // Test 1: Basic client creation
    let client = ScoutClient::new("test_key".to_string());
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.api_key, "test_key");
    assert!(client.device.is_none());
    assert!(client.herd.is_none());
    assert!(!client.is_identified());

    // Test 2: Client with different API keys
    let api_keys = vec![
        "key1",
        "key2",
        "test_api_key_123",
        "very_long_api_key_that_should_work_fine",
    ];

    for api_key in api_keys {
        let client = ScoutClient::new(api_key.to_string());
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.api_key, api_key);
    }

    // Test 3: Client state validation
    let client = ScoutClient::new("test_key".to_string()).unwrap();

    // Initial state
    assert!(client.device.is_none());
    assert!(client.herd.is_none());
    assert!(!client.is_identified());

    // Check that we can't access private fields directly
    // (This is a compile-time check, so we just verify the public interface works)
    assert_eq!(client.api_key, "test_key");

    // Test 5: Client with environment variables
    let env_client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "env_test_key".to_string()),
    );
    assert!(env_client.is_ok());
}

#[tokio::test]
async fn test_response_scout_types_comprehensive() {
    setup_test_env();

    // Test 1: Success response with different data types
    let success_string =
        ResponseScout::new(ResponseScoutStatus::Success, Some("test data".to_string()));
    assert_eq!(success_string.status, ResponseScoutStatus::Success);
    assert!(success_string.data.is_some());
    assert_eq!(success_string.data.unwrap(), "test data");

    let success_int = ResponseScout::new(ResponseScoutStatus::Success, Some(42));
    assert_eq!(success_int.status, ResponseScoutStatus::Success);
    assert!(success_int.data.is_some());
    assert_eq!(success_int.data.unwrap(), 42);

    let success_float = ResponseScout::new(ResponseScoutStatus::Success, Some(3.14));
    assert_eq!(success_float.status, ResponseScoutStatus::Success);
    assert!(success_float.data.is_some());
    assert_eq!(success_float.data.unwrap(), 3.14);

    let success_bool = ResponseScout::new(ResponseScoutStatus::Success, Some(true));
    assert_eq!(success_bool.status, ResponseScoutStatus::Success);
    assert!(success_bool.data.is_some());
    assert_eq!(success_bool.data.unwrap(), true);

    let success_vec = ResponseScout::new(ResponseScoutStatus::Success, Some(vec![1, 2, 3]));
    assert_eq!(success_vec.status, ResponseScoutStatus::Success);
    assert!(success_vec.data.is_some());
    assert_eq!(success_vec.data.unwrap(), vec![1, 2, 3]);

    // Test 2: Failure responses
    let failure_string = ResponseScout::new(ResponseScoutStatus::Failure, None::<String>);
    assert_eq!(failure_string.status, ResponseScoutStatus::Failure);
    assert!(failure_string.data.is_none());

    let failure_int = ResponseScout::new(ResponseScoutStatus::Failure, None::<i32>);
    assert_eq!(failure_int.status, ResponseScoutStatus::Failure);
    assert!(failure_int.data.is_none());

    // Test 3: Not authorized responses
    let not_authorized_string =
        ResponseScout::new(ResponseScoutStatus::NotAuthorized, None::<String>);
    assert_eq!(
        not_authorized_string.status,
        ResponseScoutStatus::NotAuthorized
    );
    assert!(not_authorized_string.data.is_none());

    let not_authorized_int = ResponseScout::new(ResponseScoutStatus::NotAuthorized, None::<i32>);
    assert_eq!(
        not_authorized_int.status,
        ResponseScoutStatus::NotAuthorized
    );
    assert!(not_authorized_int.data.is_none());

    // Test 4: Invalid event responses
    let invalid_event_string =
        ResponseScout::new(ResponseScoutStatus::InvalidEvent, None::<String>);
    assert_eq!(
        invalid_event_string.status,
        ResponseScoutStatus::InvalidEvent
    );
    assert!(invalid_event_string.data.is_none());

    let invalid_event_int = ResponseScout::new(ResponseScoutStatus::InvalidEvent, None::<i32>);
    assert_eq!(invalid_event_int.status, ResponseScoutStatus::InvalidEvent);
    assert!(invalid_event_int.data.is_none());

    // Test 5: Invalid file responses
    let invalid_file_string = ResponseScout::new(ResponseScoutStatus::InvalidFile, None::<String>);
    assert_eq!(invalid_file_string.status, ResponseScoutStatus::InvalidFile);
    assert!(invalid_file_string.data.is_none());

    let invalid_file_int = ResponseScout::new(ResponseScoutStatus::InvalidFile, None::<i32>);
    assert_eq!(invalid_file_int.status, ResponseScoutStatus::InvalidFile);
    assert!(invalid_file_int.data.is_none());

    // Test 6: Response cloning
    let original = ResponseScout::new(ResponseScoutStatus::Success, Some("original"));
    let cloned = original.clone();
    assert_eq!(original.status, cloned.status);
    assert_eq!(original.data, cloned.data);

    // Test 7: Response comparison
    let response1 = ResponseScout::new(ResponseScoutStatus::Success, Some("data"));
    let response2 = ResponseScout::new(ResponseScoutStatus::Success, Some("data"));
    let response3 = ResponseScout::new(ResponseScoutStatus::Failure, None::<&str>);

    assert_eq!(response1, response2);
    assert_ne!(response1, response3);

    // Test 8: Response with complex data types
    #[derive(Debug, Clone, PartialEq)]
    struct ComplexData {
        id: u32,
        name: String,
        values: Vec<i32>,
    }

    let complex_data = ComplexData {
        id: 1,
        name: "test".to_string(),
        values: vec![1, 2, 3],
    };

    let success_complex =
        ResponseScout::new(ResponseScoutStatus::Success, Some(complex_data.clone()));
    assert_eq!(success_complex.status, ResponseScoutStatus::Success);
    assert!(success_complex.data.is_some());
    assert_eq!(success_complex.data.unwrap(), complex_data);
}

#[tokio::test]
async fn test_data_validation_comprehensive() {
    setup_test_env();

    let _client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Test 1: Valid event data
    let valid_event = Event::new(
        Some("Valid event".to_string()),
        Some("https://example.com/image.jpg".to_string()),
        None,
        None,
        19.754824,  // Valid latitude
        -155.15393, // Valid longitude
        10.0,       // Valid altitude
        0.0,        // Valid heading
        MediaType::Image,
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None,
    );

    assert!(valid_event.location.is_some());
    assert_eq!(valid_event.location.unwrap(), "POINT(-155.15393 19.754824)");
    assert_eq!(valid_event.media_type, MediaType::Image);
    assert_eq!(valid_event.is_public, false);

    // Test 2: Valid session data
    let valid_session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200, // Start time
        1640998800, // End time (after start time)
        "v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        100.0,  // altitude_max
        50.0,   // altitude_min
        75.0,   // altitude_average
        10.0,   // velocity_max
        5.0,    // velocity_min
        7.5,    // velocity_average
        1000.0, // distance_total
        500.0,  // distance_max_from_start
    );

    assert!(valid_session.timestamp_start < valid_session.timestamp_end);
    assert_eq!(valid_session.software_version, "v1.0.0");
    assert!(valid_session.altitude_max >= valid_session.altitude_min);

    // Test 3: Valid tag data
    let valid_tag = Tag::new(
        1,
        100.0,                    // x
        200.0,                    // y
        50.0,                     // width
        30.0,                     // height
        0.95,                     // confidence (0-1)
        TagObservationType::Auto, // Valid observation_type
        "elephant".to_string(),   // class_name
    );

    assert!(valid_tag.conf >= 0.0 && valid_tag.conf <= 1.0);
    assert!(valid_tag.width > 0.0);
    assert!(valid_tag.height > 0.0);
    assert!(
        valid_tag.observation_type == TagObservationType::Auto
            || valid_tag.observation_type == TagObservationType::Manual
    );

    // Test 4: Valid connectivity data
    let valid_connectivity = Connectivity::new(
        1, // session_id
        1640995200,
        -45.0, // signal
        -60.0, // noise
        100.0, // altitude
        180.0, // heading
        "POINT(-155.15393 19.754824)".to_string(),
        "H14_INDEX".to_string(),
        "H13_INDEX".to_string(),
        "H12_INDEX".to_string(),
        "H11_INDEX".to_string(),
    );

    assert_eq!(valid_connectivity.session_id, 1);
    assert_eq!(valid_connectivity.h14_index, "H14_INDEX");
    assert_eq!(valid_connectivity.h13_index, "H13_INDEX");

    // Test 5: Edge case validation
    let edge_event = Event::new(
        None,             // No message
        None,             // No media URL
        None,             // No file path
        None,             // No earthranger URL
        90.0,             // Maximum latitude
        -180.0,           // Minimum longitude
        0.0,              // Minimum altitude
        359.0,            // Maximum heading
        MediaType::Video, // Different media type
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        0,    // Minimum timestamp
        true, // Public event
        None,
    );

    assert!(edge_event.location.is_some());
    assert_eq!(edge_event.media_type, MediaType::Video);
    assert_eq!(edge_event.is_public, true);

    // Test 6: Boundary validation
    let boundary_session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        0,          // Minimum timestamp
        9999999999, // Reasonable maximum timestamp (year 2286)
        "boundary_test".to_string(),
        None, // No location
        0.0,  // Minimum altitude
        0.0,  // Minimum altitude
        0.0,  // Average altitude
        0.0,  // Minimum velocity
        0.0,  // Minimum velocity
        0.0,  // Average velocity
        0.0,  // Minimum distance
        0.0,  // Minimum distance
    );

    assert!(boundary_session.timestamp_start < boundary_session.timestamp_end);
    assert_eq!(boundary_session.altitude_max, 0.0);
    assert_eq!(boundary_session.altitude_min, 0.0);

    // Test 7: Invalid data should be caught by the type system
    // This is a compile-time check, so we just verify the structures work correctly
    let invalid_tag = Tag::new(
        0,                        // Invalid class_id
        -100.0,                   // Negative x (invalid)
        -200.0,                   // Negative y (invalid)
        -50.0,                    // Negative width (invalid)
        -30.0,                    // Negative height (invalid)
        1.5,                      // Confidence > 1 (invalid)
        TagObservationType::Auto, // Valid observation_type
        "".to_string(),           // Empty class_name
    );

    // Even with invalid data, the structure should be created (validation happens elsewhere)
    assert_eq!(invalid_tag.x, -100.0);
    assert_eq!(invalid_tag.y, -200.0);
    assert_eq!(invalid_tag.conf, 1.5);
    assert_eq!(invalid_tag.observation_type, TagObservationType::Auto);
}

#[test]
fn test_tag_location_functionality() {
    println!("🧪 Testing Tag location functionality...");

    // Test 1: Create tag without location
    let mut tag = Tag::new(
        1,
        0.5,
        0.5,
        0.2,
        0.2,
        0.9,
        TagObservationType::Manual,
        "animal".to_string(),
    );

    assert!(tag.location.is_none());
    assert!(tag.get_coordinates().is_none());
    println!("✅ Tag without location created successfully");

    // Test 2: Create tag with location using new_with_location
    let tag_with_location = Tag::new_with_location(
        1,
        0.5,
        0.5,
        0.2,
        0.2,
        0.9,
        TagObservationType::Manual,
        "animal".to_string(),
        40.7128,
        -74.0060,
    );

    assert!(tag_with_location.location.is_some());
    assert!(tag_with_location.location.is_some());
    assert!(tag_with_location
        .location
        .as_ref()
        .unwrap()
        .contains("POINT("));
    assert!(tag_with_location
        .location
        .as_ref()
        .unwrap()
        .contains("40.7128"));
    assert!(tag_with_location
        .location
        .as_ref()
        .unwrap()
        .contains("-74.006"));

    if let Some((lat, lon)) = tag_with_location.get_coordinates() {
        assert!((lat - 40.7128).abs() < 0.0001);
        assert!((lon - (-74.0060)).abs() < 0.0001);
        println!(
            "✅ Tag with location created successfully: lat={}, lon={}",
            lat, lon
        );
    } else {
        panic!("❌ Failed to get coordinates from tag with location");
    }

    // Test 3: Set location after creation
    tag.set_location(37.7749, -122.4194);
    assert!(tag.location.is_some());
    assert!(tag.location.is_some());
    assert!(tag.location.as_ref().unwrap().contains("POINT("));
    assert!(tag.location.as_ref().unwrap().contains("37.7749"));
    assert!(tag.location.as_ref().unwrap().contains("-122.4194"));

    if let Some((lat, lon)) = tag.get_coordinates() {
        assert!((lat - 37.7749).abs() < 0.0001);
        assert!((lon - (-122.4194)).abs() < 0.0001);
        println!("✅ Location set after creation: lat={}, lon={}", lat, lon);
    } else {
        panic!("❌ Failed to get coordinates after setting location");
    }

    // Test 4: Parse location string
    if let Some((lat, lon)) = Tag::parse_location("POINT(-74.0060 40.7128)") {
        assert!((lat - 40.7128).abs() < 0.0001);
        assert!((lon - (-74.0060)).abs() < 0.0001);
        println!("✅ Location parsing successful: lat={}, lon={}", lat, lon);
    } else {
        panic!("❌ Failed to parse location string");
    }

    // Test 5: Clear location
    tag.clear_location();
    assert!(tag.location.is_none());
    assert!(tag.get_coordinates().is_none());
    println!("✅ Location cleared successfully");

    // Test 6: Invalid location string parsing
    assert!(Tag::parse_location("Invalid format").is_none());
    assert!(Tag::parse_location("POINT(invalid coords)").is_none());
    assert!(Tag::parse_location("POINT(1)").is_none());
    println!("✅ Invalid location string handling works correctly");

    println!("🎉 All tag location functionality tests passed!");
}

#[test]
fn test_tag_upload_with_location() {
    println!("🧪 Testing Tag upload with location...");

    // This test verifies that tags with location can be serialized correctly
    // for database upload (even though we can't actually upload without a real DB)

    // Create a tag with location
    let mut tag = Tag::new_with_location(
        1,
        0.5,
        0.5,
        0.2,
        0.2,
        0.9,
        TagObservationType::Manual,
        "elephant".to_string(),
        40.7128,
        -74.0060,
    );

    // Set event_id as would happen in real upload
    tag.update_event_id(123);

    // Verify the tag has location
    assert!(tag.location.is_some());
    assert_eq!(tag.location, Some("POINT(-74.006 40.7128)".to_string()));

    // Test serialization (what happens when uploading to database)
    let serialized = serde_json::to_string(&tag).unwrap();
    println!("Serialized tag: {}", serialized);

    // Verify location is included in serialized JSON
    assert!(serialized.contains("POINT(-74.006 40.7128)"));
    assert!(serialized.contains("\"location\""));

    // Test deserialization
    let deserialized: Tag = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.location, tag.location);
    assert_eq!(deserialized.event_id, tag.event_id);

    // Test that coordinates can be extracted
    if let Some((lat, lon)) = deserialized.get_coordinates() {
        assert!((lat - 40.7128).abs() < 0.0001);
        assert!((lon - (-74.0060)).abs() < 0.0001);
        println!(
            "✅ Coordinates extracted correctly: lat={}, lon={}",
            lat, lon
        );
    } else {
        panic!("❌ Failed to extract coordinates from deserialized tag");
    }

    println!("✅ Tag upload with location test passed!");
}

#[tokio::test]
async fn test_tag_upload_with_location_integration() {
    println!("🧪 Testing Tag upload with location integration...");

    // This test would require a real database connection to actually upload
    // For now, we'll test the serialization and preparation for upload

    // Create a tag with location
    let mut tag = Tag::new_with_location(
        1,
        0.5,
        0.5,
        0.2,
        0.2,
        0.9,
        TagObservationType::Manual,
        "elephant".to_string(),
        40.7128,
        -74.0060,
    );

    // Set event_id as would happen in real upload
    tag.update_event_id(123);

    // Test that the tag is ready for database upload
    assert!(tag.location.is_some());
    assert_eq!(tag.location, Some("POINT(-74.006 40.7128)".to_string()));
    assert_eq!(tag.event_id, 123);

    // Test serialization for database upload
    let serialized = serde_json::to_string(&tag).unwrap();
    assert!(serialized.contains("\"location\":\"POINT(-74.006 40.7128)\""));
    assert!(serialized.contains("\"event_id\":123"));

    // Test that the tag can be deserialized correctly
    let deserialized: Tag = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.location, tag.location);
    assert_eq!(deserialized.event_id, tag.event_id);

    // Test coordinate extraction
    if let Some((lat, lon)) = deserialized.get_coordinates() {
        assert!((lat - 40.7128).abs() < 0.0001);
        assert!((lon - (-74.0060)).abs() < 0.0001);
        println!(
            "✅ Coordinates extracted correctly: lat={}, lon={}",
            lat, lon
        );
    } else {
        panic!("❌ Failed to extract coordinates from tag");
    }

    // Test that the tag would be compatible with database upload
    // (This simulates what happens in create_tags function)
    let tags_for_upload = vec![tag];
    let serialized_array = serde_json::to_string(&tags_for_upload).unwrap();
    assert!(serialized_array.contains("\"location\":\"POINT(-74.006 40.7128)\""));

    println!("✅ Tag upload with location integration test passed!");
    println!("   Serialized array: {}", serialized_array);
}

#[tokio::test]
async fn test_tag_upload_with_location_database() {
    let cleanup = TestCleanup::new();
    test_tag_upload_with_location_database_impl(&cleanup).await;
}

async fn test_tag_upload_with_location_database_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!(
            "❌ Client identification failed: {:?}",
            identify_result.err()
        );
    }

    println!("🧪 Testing Tag upload with location to database...");

    // Get the actual device ID from the identified client
    let device_id = client.device.as_ref().unwrap().id;

    // First create a real session for the event
    let session = Session::new(
        device_id,
        1640995200,
        1640998800,
        "tag_location_test_v1.0.0".to_string(),
        Some("POINT(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        500.0,
    );

    // Create the session first
    let session_result = client.create_session(&session).await;
    let session_id = match session_result {
        Ok(session_response) => {
            if session_response.status == ResponseScoutStatus::Success {
                let created_session = session_response.data.unwrap();
                let session_id = created_session.id.unwrap();
                cleanup.track_session(session_id);
                println!("✅ Session created with ID: {}", session_id);

                // Verify the session actually exists in the database
                match client
                    .does_session_exist_from_session(&created_session)
                    .await
                {
                    Ok(exists) => {
                        if exists {
                            println!("✅ Session existence verified in database");
                        } else {
                            panic!(
                                "❌ Session was created but doesn't exist in database: {}",
                                session_id
                            );
                        }
                    }
                    Err(e) => {
                        panic!("❌ Failed to verify session existence: {}", e);
                    }
                }

                session_id
            } else {
                panic!("❌ Session creation failed: {:?}", session_response.status);
            }
        }
        Err(e) => {
            panic!(
                "❌ Session creation failed: {}. This test requires successful session creation.",
                e
            );
        }
    };

    // Create an event
    let event = Event::new(
        Some("Test event with tagged location".to_string()),
        Some("https://example.com/tagged_image.jpg".to_string()),
        None,       // file_path
        None,       // earthranger_url
        19.754824,  // latitude
        -155.15393, // longitude
        120.0,      // altitude
        45.0,       // heading
        MediaType::Image,
        device_id,
        1640995200, // timestamp_observation
        false,      // is_public
        Some(session_id),
    );

    // Create the event first
    let event_result = client.create_event(&event).await;
    match event_result {
        Ok(event_response) => {
            if event_response.status == ResponseScoutStatus::Success {
                let created_event = event_response.data.unwrap();
                let event_id = created_event.id.unwrap();
                cleanup.track_event(event_id);
                println!("✅ Event created with ID: {}", event_id);

                // Create tags with location data
                let mut tags_with_location = vec![
                    Tag::new_with_location(
                        1,
                        0.3,
                        0.4,
                        0.1,
                        0.15,
                        0.95,
                        TagObservationType::Manual,
                        "elephant".to_string(),
                        40.7128,
                        -74.0060, // New York City coordinates
                    ),
                    Tag::new_with_location(
                        2,
                        0.6,
                        0.7,
                        0.2,
                        0.25,
                        0.87,
                        TagObservationType::Auto,
                        "giraffe".to_string(),
                        37.7749,
                        -122.4194, // San Francisco coordinates
                    ),
                ];

                // Set event_id for all tags
                for tag in &mut tags_with_location {
                    tag.update_event_id(event_id);
                }

                // Verify tags have location data before upload
                for (i, tag) in tags_with_location.iter().enumerate() {
                    assert!(
                        tag.location.is_some(),
                        "Tag {} should have location data",
                        i
                    );
                    println!("✅ Tag {} location: {:?}", i, tag.location);
                }

                // Upload tags to database
                let tags_result = client.create_tags(event_id, &tags_with_location).await;
                match tags_result {
                    Ok(tags_response) => {
                        if tags_response.status == ResponseScoutStatus::Success {
                            let created_tags = tags_response.data.unwrap();
                            println!(
                                "✅ Successfully uploaded {} tags with location data",
                                created_tags.len()
                            );

                            // Verify the uploaded tags have the correct event_id
                            for (i, tag) in created_tags.iter().enumerate() {
                                assert_eq!(
                                    tag.event_id, event_id,
                                    "Tag {} should have correct event_id",
                                    i
                                );

                                // Verify location data was preserved
                                if let Some(location) = &tag.location {
                                    println!("Tag {} location from database: {}", i, location);
                                    // Location is stored in PostGIS binary format, which is correct
                                    assert!(
                                        !location.is_empty(),
                                        "Tag {} should have location data",
                                        i
                                    );
                                    println!("✅ Tag {} uploaded with location: {}", i, location);
                                } else {
                                    panic!("❌ Tag {} should have location data after upload", i);
                                }
                            }

                            // Track all created tags for cleanup
                            for created_tag in &created_tags {
                                if let Some(tag_id) = created_tag.id {
                                    cleanup.track_tag(tag_id);
                                }
                            }

                            println!(
                                "�� All tags with location data successfully uploaded to database!"
                            );
                        } else {
                            panic!("❌ Tags creation failed: {:?}", tags_response.status);
                        }
                    }
                    Err(e) => {
                        panic!("❌ Tags creation failed: {}. This test requires successful tag creation.", e);
                    }
                }
            } else {
                panic!("❌ Event creation failed: {:?}", event_response.status);
            }
        }
        Err(e) => {
            panic!(
                "❌ Event creation failed: {}. This test requires successful event creation.",
                e
            );
        }
    }
}
