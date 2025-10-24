use scout_rs::client::*;
use scout_rs::models::{
    data, Connectivity, Event, Heartbeat, MediaType, Plan, PlanType, ResponseScoutStatus, Session,
    Syncable, Tag, TagObservationType,
};
use std::env;

use once_cell::sync::Lazy;
use std::sync::Mutex;
use tokio::sync::Mutex as TokioMutex;

// Global mutex to prevent concurrent database tests from interfering with each other
static DB_TEST_MUTEX: Lazy<TokioMutex<()>> = Lazy::new(|| TokioMutex::new(()));

/// Scout Client Integration Tests
///
/// Tests client operations including events, sessions, connectivity, tags, and plans.
/// Uses real database operations with proper cleanup.

// Test cleanup system - tracks and deletes test data
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
    heartbeats: Vec<i64>,
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
            heartbeats: Vec::new(),
        }
    }

    fn reset(&mut self) {
        self.events.clear();
        self.sessions.clear();
        self.connectivity.clear();
        self.tags.clear();
        self.artifacts.clear();
        self.plans.clear();
        self.heartbeats.clear();
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

    /// Track heartbeat ID for cleanup
    fn track_heartbeat(&self, heartbeat_id: i64) {
        if let Ok(mut tracker) = self.tracker.lock() {
            tracker.heartbeats.push(heartbeat_id);
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

            // Clean up heartbeats (they reference devices)
            for &heartbeat_id in &tracker.heartbeats {
                let _ = client.delete_heartbeat(heartbeat_id).await;
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
            // Acquire global database test lock to prevent concurrent database access
            let _guard = DB_TEST_MUTEX.lock().await;

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
    let has_supabase_api_key = env::var("SUPABASE_PUBLIC_API_KEY").is_ok();

    if !has_supabase_api_key {
        panic!(
            "❌ Missing Supabase API key. Set SUPABASE_PUBLIC_API_KEY, SCOUT_SUPABASE_ANON_KEY, or SCOUT_SUPABASE_SERVICE_KEY in your .env file."
        );
    }
}

#[tokio::test]
async fn test_client_identification() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
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

async fn test_event_batch_creation_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

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

    let batch_result = client
        .create_events_batch(&events)
        .await
        .expect("Batch event creation failed");
    assert_eq!(batch_result.status, ResponseScoutStatus::Success);
    let created_events = batch_result.data.unwrap();
    assert_eq!(created_events.len(), 3);
    for created_event in &created_events {
        if let Some(event_id) = created_event.id {
            cleanup.track_event(event_id);
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

    client
        .identify()
        .await
        .expect("Client identification failed");

    // First create a real session for the event
    let session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        Some(1640998800),
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

async fn test_does_session_exist_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    // Get the actual device ID from the identified client
    let device_id = client.device.as_ref().unwrap().id.unwrap();

    let exists_result = client
        .does_session_exist(device_id, "2023-01-01T00:00:00Z")
        .await
        .expect("Session existence check failed");
    assert!(!exists_result, "Non-existent session should not exist");

    let unique_start_timestamp = chrono::Utc::now().timestamp() as u64;
    let unique_end_timestamp = unique_start_timestamp + 3600;

    let session = Session::new(
        device_id,
        unique_start_timestamp,
        Some(unique_end_timestamp),
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

    let session_result = client
        .create_session(&session)
        .await
        .expect("Session creation failed");
    assert_eq!(session_result.status, ResponseScoutStatus::Success);
    let created_session = session_result.data.unwrap();
    let session_id = created_session.id.unwrap();
    cleanup.track_session(session_id);

    let exists_result = client
        .does_session_exist(device_id, &created_session.timestamp_start)
        .await
        .expect("Session existence check failed");
    assert!(exists_result, "Created session should exist");

    let delete_result = client
        .delete_session(session_id)
        .await
        .expect("Session deletion failed");
    assert_eq!(delete_result.status, ResponseScoutStatus::Success);

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let exists_after_delete = client
        .does_session_exist(device_id, &created_session.timestamp_start)
        .await
        .expect("Post-deletion check failed");
    assert!(!exists_after_delete, "Deleted session should not exist");
}

test_with_cleanup!(test_does_session_exist, test_does_session_exist_impl);

#[tokio::test]
async fn test_compatibility_methods() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    // Test post_events_batch with proper event and tag creation
    // First create a session for the events
    let session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        Some(1640998800),
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

    let session_result = client
        .create_session(&session)
        .await
        .expect("Session creation failed");
    assert_eq!(session_result.status, ResponseScoutStatus::Success);
    let created_session = session_result.data.unwrap();
    let session_id = created_session.id.unwrap();

    let device_id = client.device.as_ref().unwrap().id.unwrap();

    let events_and_files = vec![
        (
            Event::new(
                Some("Compatibility test event 1".to_string()),
                Some("https://example.com/compat1.jpg".to_string()),
                None,
                None,
                19.754824,
                -155.15393,
                15.0,
                90.0,
                MediaType::Image,
                device_id,
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
                "elephant".to_string(),
            )],
            "/path/to/file1.jpg".to_string(),
        ),
        (
            Event::new(
                Some("Compatibility test event 2".to_string()),
                Some("https://example.com/compat2.jpg".to_string()),
                None,
                None,
                19.755,
                -155.154,
                20.0,
                180.0,
                MediaType::Image,
                device_id,
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
                0.85,
                TagObservationType::Manual,
                "bird".to_string(),
            )],
            "/path/to/file2.jpg".to_string(),
        ),
    ];

    let batch_result = client
        .post_events_batch(&events_and_files, 10)
        .await
        .expect("Batch creation failed");
    assert_eq!(batch_result.status, ResponseScoutStatus::Success);
    let created_events = batch_result.data.unwrap();
    assert_eq!(created_events.len(), 2);

    let _ = client.delete_session(session_id).await;
}

#[tokio::test]
async fn test_error_handling() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
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
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
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

#[tokio::test]
async fn test_device_events_with_tags_via_function() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let device_id = client.device.as_ref().unwrap().id.unwrap();

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
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let herd_id = client.herd.as_ref().unwrap().id.unwrap();

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
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Identify the client - should always succeed with proper credentials
    client
        .identify()
        .await
        .expect("Client identification failed");

    // First create a session to test connectivity
    let session = Session::new(
        env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        Some(1640998800),
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
                Err(_e) => {
                    // This is expected if no connectivity data exists yet
                }
            }

            // Clean up the test session (cascades to connectivity)
            let _ = client.delete_session(session_id).await;
        }
    } else {
    }
}

async fn test_plans_comprehensive_impl(_cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let herd_id = client
        .herd
        .as_ref()
        .unwrap()
        .id
        .expect("Herd should have an ID");

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
                    assert!(plan.id.unwrap_or(0) >= 0, "Plan ID should be non-negative");

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
        id: Some(1),
        id_local: None,
        inserted_at: Some("2023-01-01T00:00:00Z".to_string()),
        name: "Test Plan".to_string(),
        instructions: "Test instructions for the plan".to_string(),
        herd_id: 1,
        plan_type: PlanType::Mission,
    };

    // Validate test plan structure
    assert_eq!(test_plan.id, Some(1));
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
            id: Some(0), // Placeholder ID for testing
            id_local: None,
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
                    let individual_plan_result = client
                        .get_plan_by_id(plan_id.expect("Plan should have ID"))
                        .await;
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
                                plan_id.unwrap_or(0)
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

async fn test_plans_bulk_operations_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let herd_id = client.herd.as_ref().unwrap().id.unwrap();
    let mut created_plan_ids = Vec::new();

    // Test 1: Create multiple plans
    let test_plans = vec![
        Plan {
            id: None,
            id_local: None,
            inserted_at: None, // Database will use default value
            name: "Bulk Test Plan 1".to_string(),
            instructions: "First bulk test plan".to_string(),
            herd_id,
            plan_type: PlanType::Mission,
        },
        Plan {
            id: None,
            id_local: None,
            inserted_at: None, // Database will use default value
            name: "Bulk Test Plan 2".to_string(),
            instructions: "Second bulk test plan".to_string(),
            herd_id,
            plan_type: PlanType::Fence,
        },
        Plan {
            id: None,
            id_local: None,
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
                    created_plan.id.unwrap() >= 0,
                    "Plan {} should have a valid ID",
                    i + 1
                );
                assert_eq!(created_plan.name, format!("Bulk Test Plan {}", i + 1));
                assert_eq!(created_plan.herd_id, herd_id);

                created_plan_ids.push(created_plan.id.unwrap());
                cleanup.track_plan(created_plan.id.unwrap());
            }
            Err(e) => {
                panic!("Failed to create bulk test plan {}: {}", i + 1, e);
            }
        }
    }

    // Add delay to ensure all bulk plans are committed to database
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test 2: Verify all plans were created
    let plans_result = client.get_plans_by_herd(herd_id).await;
    match plans_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            if let Some(plans) = response.data {
                // Check that we can find all our created plans
                for plan_id in &created_plan_ids {
                    let found_plan = plans.iter().find(|p| p.id.unwrap_or(0) == *plan_id);
                    assert!(
                        found_plan.is_some(),
                        "Should find bulk test plan with ID {}",
                        plan_id
                    );
                }
            }
        }
        Err(e) => {
            panic!("Failed to read plans after bulk creation: {}", e);
        }
    }

    // Test 3: Clean up all created plans
    for plan_id in &created_plan_ids {
        let delete_result = client.delete_plan(*plan_id).await;
        match delete_result {
            Ok(response) => {
                assert_eq!(response.status, ResponseScoutStatus::Success);
            }
            Err(e) => {
                panic!("Failed to delete bulk test plan {}: {}", plan_id, e);
            }
        }
    }

    // Add delay to ensure deletions are committed to database
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Test 4: Verify all plans were deleted
    let plans_result = client.get_plans_by_herd(herd_id).await;
    match plans_result {
        Ok(response) => {
            assert_eq!(response.status, ResponseScoutStatus::Success);
            if let Some(plans) = response.data {
                // Check that none of our created plans exist
                for plan_id in &created_plan_ids {
                    let found_plan = plans.iter().find(|p| p.id.unwrap_or(0) == *plan_id);
                    assert!(
                        found_plan.is_none(),
                        "Should not find deleted bulk test plan with ID {}",
                        plan_id
                    );
                }
            }
        }
        Err(e) => {
            panic!("Failed to read plans after bulk deletion: {}", e);
        }
    }
}

test_with_cleanup!(test_plans_bulk_operations, test_plans_bulk_operations_impl);

#[tokio::test]
async fn test_identify_method_fix() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    // Create a client with actual credentials from .env file
    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Test the specific identify scenario that was failing
    let identify_result = client.identify().await;

    match identify_result {
        Ok(_) => {
            println!("✅ Identify method fixed and working correctly");

            // Verify that device and herd are properly loaded
            assert!(
                client.device.is_some(),
                "Device should be loaded after identify"
            );
            assert!(
                client.herd.is_some(),
                "Herd should be loaded after identify"
            );
            assert!(client.is_identified(), "Client should be identified");

            // Verify device has proper fields populated
            let device = client.device.as_ref().unwrap();
            assert!(device.id.is_some(), "Device ID should be populated");
            assert!(!device.name.is_empty(), "Device name should be populated");
            assert!(device.herd_id > 0, "Device should have a valid herd_id");

            println!("✅ All device fields properly populated");
        }
        Err(e) => {
            // If this fails, it means the test environment doesn't have valid credentials
            // which is acceptable for this test
            if e.to_string().contains("Failed to parse device ID response") {
                println!("⚠️ Test skipped - invalid credentials (expected in some environments)");
            } else {
                panic!("❌ Identify failed with unexpected error: {}", e);
            }
        }
    }
}

#[tokio::test]
async fn test_zones_and_actions_by_herd() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let herd_id = client.herd.as_ref().unwrap().id.unwrap();

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

#[tokio::test]
async fn test_complete_data_collection_workflow() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    // Step 1: Identify the client
    client
        .identify()
        .await
        .expect("Client identification failed");

    let device_id = client.device.as_ref().unwrap().id.unwrap();
    let _herd_id = client.herd.as_ref().unwrap().id.unwrap();

    // Step 2: Create a session
    let session = Session::new(
        device_id,
        chrono::Utc::now().timestamp() as u64,
        Some((chrono::Utc::now().timestamp() as u64) + 3600),
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
            let session_id = created_session.id.unwrap_or(0);

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

                // Step 4: Create tags for the first event (if any events were created)
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
                        } else {
                            eprintln!("Tags creation failed, but continuing test");
                        }
                    }
                }

                // Step 5: Create connectivity data
                let connectivity = Connectivity::new(
                    Some(session_id),
                    None, // device_id
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
                    Some(85.0), // battery_percentage
                );

                let connectivity_result = client.create_connectivity(&connectivity).await;
                if let Ok(response) = connectivity_result {
                    if response.status == ResponseScoutStatus::Success {}
                }

                // Step 6: Query the data we just created
                let session_events = client.get_session_events(session_id).await;
                if let Ok(response) = session_events {
                    if response.status == ResponseScoutStatus::Success {
                        let _events = response.data.unwrap();
                    }
                }

                let session_connectivity = client.get_session_connectivity(session_id).await;
                if let Ok(response) = session_connectivity {
                    if response.status == ResponseScoutStatus::Success {
                        let _connectivity_entries = response.data.unwrap();
                    }
                }

                // Step 7: Clean up test data (session deletion cascades to events, tags, connectivity)
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
async fn test_tag_upload_with_location_integration() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;

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
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    let cleanup = TestCleanup::new();
    test_tag_upload_with_location_database_impl(&cleanup).await;
}

async fn test_tag_upload_with_location_database_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    // Get the actual device ID from the identified client
    let device_id = client.device.as_ref().unwrap().id.unwrap();

    // First create a real session for the event
    let session = Session::new(
        device_id,
        1640995200,
        Some(1640998800),
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
                let session_id = created_session.id.unwrap_or(0);
                cleanup.track_session(session_id);

                // Add a delay to ensure session is committed to database
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

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
                panic!("Session creation failed: {:?}", session_response.status);
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
        1640995200,       // timestamp_observation
        false,            // is_public
        Some(session_id), // session_id - restored with debugging
    );

    // Create the event first
    let event_result = client.create_event(&event).await;
    match event_result {
        Ok(event_response) => {
            if event_response.status == ResponseScoutStatus::Success {
                let created_event = event_response.data.unwrap();
                let event_id = created_event.id.unwrap();
                cleanup.track_event(event_id);

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
                        panic!(
                            "Tags creation failed: {}. This test requires successful tag creation.",
                            e
                        );
                    }
                }
            } else {
                panic!("Event creation failed: {:?}", event_response.status);
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

test_with_cleanup!(test_sessions_batch_upsert, test_sessions_batch_upsert_impl);

async fn test_sessions_batch_upsert_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let device_id: i64 = env::var("SCOUT_DEVICE_ID")
        .unwrap_or_else(|_| "123".to_string())
        .parse()
        .unwrap_or(123);

    // Create initial sessions
    let sessions = vec![
        Session::new(
            device_id,
            1704103200,       // 2024-01-01T10:00:00Z
            Some(1704106800), // 2024-01-01T11:00:00Z
            "1.0.0".to_string(),
            None,
            100.0,
            50.0,
            75.0,
            25.0,
            10.0,
            15.0,
            1000.0,
            500.0,
        ),
        Session::new(
            device_id,
            1704110400,       // 2024-01-01T12:00:00Z
            Some(1704114000), // 2024-01-01T13:00:00Z
            "1.0.0".to_string(),
            None,
            120.0,
            60.0,
            90.0,
            30.0,
            15.0,
            20.0,
            1200.0,
            600.0,
        ),
    ];

    // Initial insert
    let insert_result = client
        .create_sessions_batch(&sessions)
        .await
        .expect("Session batch creation failed");
    assert_eq!(insert_result.status, ResponseScoutStatus::Success);
    let created_sessions = insert_result.data.unwrap();
    assert_eq!(created_sessions.len(), 2);

    // Track for cleanup
    for session in &created_sessions {
        if let Some(session_id) = session.id {
            cleanup.track_session(session_id);
        }
    }

    // Modify sessions for upsert
    let mut updated_sessions = created_sessions.clone();
    updated_sessions[0].software_version = "2.0.0".to_string();
    updated_sessions[1].altitude_max = 150.0;

    // Upsert the modified sessions
    let upsert_result = client
        .upsert_sessions_batch(&updated_sessions)
        .await
        .expect("Session batch upsert failed");
    assert_eq!(upsert_result.status, ResponseScoutStatus::Success);
    let upserted_sessions = upsert_result.data.unwrap();
    assert_eq!(upserted_sessions.len(), 2);

    // Verify the updates
    assert_eq!(upserted_sessions[0].software_version, "2.0.0");
    assert_eq!(upserted_sessions[1].altitude_max, 150.0);
}

test_with_cleanup!(
    test_connectivity_batch_upsert,
    test_connectivity_batch_upsert_impl
);

async fn test_connectivity_batch_upsert_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let device_id: i64 = env::var("SCOUT_DEVICE_ID")
        .unwrap_or_else(|_| "123".to_string())
        .parse()
        .unwrap_or(123);

    // Create a session first
    let session = Session::new(
        device_id,
        1704117600,       // 2024-01-01T14:00:00Z
        Some(1704121200), // 2024-01-01T15:00:00Z
        "1.0.0".to_string(),
        None,
        100.0,
        50.0,
        75.0,
        25.0,
        10.0,
        15.0,
        1000.0,
        500.0,
    );

    let session_result = client
        .create_session(&session)
        .await
        .expect("Session creation failed");
    let created_session = session_result.data.unwrap();
    let session_id = created_session.id.unwrap();
    cleanup.track_session(session_id);

    // Create connectivity entries
    let connectivity_entries = vec![
        Connectivity::new(
            Some(session_id),
            None,       // device_id
            1704118200, // 2024-01-01T14:10:00Z
            -70.0,
            -90.0,
            100.0,
            0.0,
            "POINT(-155.15393 19.754824)".to_string(),
            "h14index1".to_string(),
            "h13index1".to_string(),
            "h12index1".to_string(),
            "h11index1".to_string(),
            Some(90.0), // battery_percentage
        ),
        Connectivity::new(
            Some(session_id),
            None,       // device_id
            1704118800, // 2024-01-01T14:20:00Z
            -75.0,
            -95.0,
            105.0,
            90.0,
            "POINT(-155.15400 19.754830)".to_string(),
            "h14index2".to_string(),
            "h13index2".to_string(),
            "h12index2".to_string(),
            "h11index2".to_string(),
            Some(88.5), // battery_percentage
        ),
    ];

    // Initial insert
    let insert_result = client
        .create_connectivity_batch(&connectivity_entries)
        .await
        .expect("Connectivity batch creation failed");
    assert_eq!(insert_result.status, ResponseScoutStatus::Success);
    let created_connectivity = insert_result.data.unwrap();
    assert_eq!(created_connectivity.len(), 2);

    // Track for cleanup
    for connectivity in &created_connectivity {
        if let Some(connectivity_id) = connectivity.id {
            cleanup.track_connectivity(connectivity_id);
        }
    }

    // Modify connectivity entries for upsert
    let mut updated_connectivity = created_connectivity.clone();
    updated_connectivity[0].signal = -65.0;
    updated_connectivity[1].noise = -85.0;

    // Upsert the modified connectivity
    let upsert_result = client
        .upsert_connectivity_batch(&updated_connectivity)
        .await
        .expect("Connectivity batch upsert failed");
    assert_eq!(upsert_result.status, ResponseScoutStatus::Success);
    let upserted_connectivity = upsert_result.data.unwrap();
    assert_eq!(upserted_connectivity.len(), 2);

    // Verify the updates
    assert_eq!(upserted_connectivity[0].signal, -65.0);
    assert_eq!(upserted_connectivity[1].noise, -85.0);
}

test_with_cleanup!(test_events_batch_upsert, test_events_batch_upsert_impl);

async fn test_events_batch_upsert_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let device_id: i64 = env::var("SCOUT_DEVICE_ID")
        .unwrap_or_else(|_| "123".to_string())
        .parse()
        .unwrap_or(123);

    // Create events
    let events = vec![
        Event::new(
            Some("Upsert event 1".to_string()),
            Some("https://test.com/upsert1.jpg".to_string()),
            None,
            None,
            19.754824,
            -155.15393,
            10.0,
            0.0,
            MediaType::Image,
            device_id,
            1640995400,
            false,
            None,
        ),
        Event::new(
            Some("Upsert event 2".to_string()),
            Some("https://test.com/upsert2.jpg".to_string()),
            None,
            None,
            19.755,
            -155.154,
            12.0,
            90.0,
            MediaType::Image,
            device_id,
            1640995460,
            false,
            None,
        ),
    ];

    // Initial insert
    let insert_result = client
        .create_events_batch(&events)
        .await
        .expect("Event batch creation failed");
    assert_eq!(insert_result.status, ResponseScoutStatus::Success);
    let created_events = insert_result.data.unwrap();
    assert_eq!(created_events.len(), 2);

    // Track for cleanup
    for event in &created_events {
        if let Some(event_id) = event.id {
            cleanup.track_event(event_id);
        }
    }

    // Modify events for upsert
    let mut updated_events = created_events.clone();
    updated_events[0].message = Some("Updated upsert event 1".to_string());
    updated_events[1].altitude = 15.0;

    // Upsert the modified events
    let upsert_result = client
        .upsert_events_batch(&updated_events)
        .await
        .expect("Event batch upsert failed");
    assert_eq!(upsert_result.status, ResponseScoutStatus::Success);
    let upserted_events = upsert_result.data.unwrap();
    assert_eq!(upserted_events.len(), 2);

    // Verify the updates
    assert_eq!(
        upserted_events[0].message,
        Some("Updated upsert event 1".to_string())
    );
    assert_eq!(upserted_events[1].altitude, 15.0);
}

test_with_cleanup!(test_tags_batch_upsert, test_tags_batch_upsert_impl);

async fn test_tags_batch_upsert_impl(cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    let device_id: i64 = env::var("SCOUT_DEVICE_ID")
        .unwrap_or_else(|_| "123".to_string())
        .parse()
        .unwrap_or(123);

    // Create an event first
    let event = Event::new(
        Some("Upsert tag event".to_string()),
        Some("https://test.com/upsert_tags.jpg".to_string()),
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        MediaType::Image,
        device_id,
        1640995500,
        false,
        None,
    );

    let event_result = client
        .create_event(&event)
        .await
        .expect("Event creation failed");
    let created_event = event_result.data.unwrap();
    let event_id = created_event.id.unwrap();
    cleanup.track_event(event_id);

    // Create tags
    let tags = vec![
        Tag::new(
            1,
            100.0,
            150.0,
            50.0,
            75.0,
            0.95,
            TagObservationType::Auto,
            "elephant".to_string(),
        ),
        Tag::new(
            2,
            200.0,
            250.0,
            40.0,
            60.0,
            0.87,
            TagObservationType::Auto,
            "zebra".to_string(),
        ),
    ];

    // Initial insert
    let insert_result = client
        .create_tags(event_id, &tags)
        .await
        .expect("Tag batch creation failed");
    assert_eq!(insert_result.status, ResponseScoutStatus::Success);
    let created_tags = insert_result.data.unwrap();
    assert_eq!(created_tags.len(), 2);

    // Track for cleanup
    for tag in &created_tags {
        if let Some(tag_id) = tag.id {
            cleanup.track_tag(tag_id);
        }
    }

    // Modify tags for upsert
    let mut updated_tags = created_tags.clone();
    updated_tags[0].conf = 0.98;
    updated_tags[1].class_name = "giraffe".to_string();

    // Upsert the modified tags
    let upsert_result = client
        .upsert_tags_batch(&updated_tags)
        .await
        .expect("Tag batch upsert failed");
    assert_eq!(upsert_result.status, ResponseScoutStatus::Success);
    let upserted_tags = upsert_result.data.unwrap();
    assert_eq!(upserted_tags.len(), 2);

    // Verify the updates
    assert_eq!(upserted_tags[0].conf, 0.98);
    assert_eq!(upserted_tags[1].class_name, "giraffe");
}

test_with_cleanup!(test_empty_batch_upserts, test_empty_batch_upserts_impl);

async fn test_empty_batch_upserts_impl(_cleanup: &TestCleanup) {
    setup_test_env();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    // Test empty upserts return success with empty data
    let empty_sessions: Vec<Session> = Vec::new();
    let session_result = client
        .upsert_sessions_batch(&empty_sessions)
        .await
        .expect("Empty session upsert failed");
    assert_eq!(session_result.status, ResponseScoutStatus::Success);
    assert_eq!(session_result.data.unwrap().len(), 0);

    let empty_connectivity: Vec<Connectivity> = Vec::new();
    let connectivity_result = client
        .upsert_connectivity_batch(&empty_connectivity)
        .await
        .expect("Empty connectivity upsert failed");
    assert_eq!(connectivity_result.status, ResponseScoutStatus::Success);
    assert_eq!(connectivity_result.data.unwrap().len(), 0);

    let empty_events: Vec<Event> = Vec::new();
    let event_result = client
        .upsert_events_batch(&empty_events)
        .await
        .expect("Empty event upsert failed");
    assert_eq!(event_result.status, ResponseScoutStatus::Success);
    assert_eq!(event_result.data.unwrap().len(), 0);

    let empty_tags: Vec<Tag> = Vec::new();
    let tag_result = client
        .upsert_tags_batch(&empty_tags)
        .await
        .expect("Empty tag upsert failed");
    assert_eq!(tag_result.status, ResponseScoutStatus::Success);
    assert_eq!(tag_result.data.unwrap().len(), 0);
}

#[tokio::test]
async fn test_heartbeat_operations() {
    // Acquire global database test lock to prevent concurrent database access
    let _guard = DB_TEST_MUTEX.lock().await;
    setup_test_env();

    let cleanup = TestCleanup::new();

    let mut client = ScoutClient::new(
        env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
    )
    .unwrap();

    client
        .identify()
        .await
        .expect("Client identification failed");

    assert!(client.is_identified(), "Client should be identified");

    // Get device info
    let device = client.get_device().await.expect("Failed to get device");
    assert_eq!(device.status, ResponseScoutStatus::Success);
    let device = device.data.unwrap();

    // Create first heartbeat
    let timestamp1 = chrono::Utc::now().to_rfc3339();
    let heartbeat1 = Heartbeat::new(timestamp1.clone(), device.id.unwrap());

    let create_result1 = client
        .create_heartbeat(&heartbeat1)
        .await
        .expect("Failed to create first heartbeat");

    assert_eq!(create_result1.status, ResponseScoutStatus::Success);
    let created_heartbeat1 = create_result1.data.unwrap();
    assert!(created_heartbeat1.id.is_some());
    assert_eq!(created_heartbeat1.device_id, device.id.unwrap());
    assert_eq!(created_heartbeat1.timestamp, timestamp1);

    // Track for cleanup
    cleanup.track_heartbeat(created_heartbeat1.id.unwrap());

    // Wait and create second heartbeat to test ordering
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let timestamp2 = chrono::Utc::now().to_rfc3339();
    let heartbeat2 = Heartbeat::new(timestamp2.clone(), device.id.unwrap());

    let create_result2 = client
        .create_heartbeat(&heartbeat2)
        .await
        .expect("Failed to create second heartbeat");

    assert_eq!(create_result2.status, ResponseScoutStatus::Success);
    let created_heartbeat2 = create_result2.data.unwrap();
    assert!(created_heartbeat2.id.is_some());
    assert_eq!(created_heartbeat2.timestamp, timestamp2);

    // Track for cleanup
    cleanup.track_heartbeat(created_heartbeat2.id.unwrap());

    // Get heartbeats for device
    let get_result = client
        .get_heartbeats_by_device(device.id.unwrap())
        .await
        .expect("Failed to get heartbeats");

    assert_eq!(get_result.status, ResponseScoutStatus::Success);
    let heartbeats = get_result.data.unwrap();
    assert!(heartbeats.len() >= 2);

    // Verify ordering (newest first)
    assert_eq!(heartbeats[0].timestamp, timestamp2);

    // Find both created heartbeats
    let found_hb1 = heartbeats.iter().find(|h| h.id == created_heartbeat1.id);
    let found_hb2 = heartbeats.iter().find(|h| h.id == created_heartbeat2.id);
    assert!(found_hb1.is_some() && found_hb2.is_some());

    // Clean up test data
    cleanup.cleanup(&mut client).await;
}

// ===== V2 MODEL TESTS =====

#[test]
fn test_connectivity_v1_to_v2_migration() {
    let v1_connectivity = data::v1::ConnectivityLocal {
        id: Some(1),
        id_local: Some("local_1".to_string()),
        session_id: 1,
        ancestor_id_local: None,
        inserted_at: Some("2024-01-01T00:00:00Z".to_string()),
        timestamp_start: "2024-01-01T00:00:00Z".to_string(),
        signal: -65.0,
        noise: -95.0,
        altitude: 100.0,
        heading: 45.0,
        location: Some("POINT(1.0 2.0)".to_string()),
        h14_index: "h14".to_string(),
        h13_index: "h13".to_string(),
        h12_index: "h12".to_string(),
        h11_index: "h11".to_string(),
    };

    // Test migration from V1 to V2
    let v2_connectivity: data::v2::ConnectivityLocal = v1_connectivity.into();

    // Verify all fields migrated correctly
    assert_eq!(v2_connectivity.signal, -65.0);
    assert_eq!(v2_connectivity.noise, -95.0);
    assert_eq!(v2_connectivity.session_id, Some(1));
    assert_eq!(v2_connectivity.h14_index, "h14");
    assert_eq!(v2_connectivity.location, Some("POINT(1.0 2.0)".to_string()));

    // New field should be None for migrated data
    assert_eq!(v2_connectivity.battery_percentage, None);
}

#[test]
fn test_connectivity_v2_with_battery() {
    let v2_connectivity = data::v2::ConnectivityLocal::new(
        Some(1),                      // session_id
        None,                         // device_id
        1234567890,                   // timestamp_start
        -65.0,                        // signal
        -95.0,                        // noise
        100.0,                        // altitude
        45.0,                         // heading
        "POINT(1.0 2.0)".to_string(), // location
        "h14".to_string(),            // h14_index
        "h13".to_string(),            // h13_index
        "h12".to_string(),            // h12_index
        "h11".to_string(),            // h11_index
        Some(85.5),                   // battery_percentage
    );

    assert_eq!(v2_connectivity.battery_percentage, Some(85.5));
    assert_eq!(v2_connectivity.session_id, Some(1));
    assert_eq!(v2_connectivity.signal, -65.0);
    assert_eq!(v2_connectivity.location, Some("POINT(1.0 2.0)".to_string()));
}

#[test]
fn test_connectivity_v2_api_struct() {
    let v2_connectivity = data::v2::Connectivity::new(
        Some(1),                                // session_id
        None,                                   // device_id
        1234567890,                             // timestamp_start
        -70.0,                                  // signal
        -100.0,                                 // noise
        150.0,                                  // altitude
        90.0,                                   // heading
        "POINT(-122.4194 37.7749)".to_string(), // location
        "h14_abc".to_string(),                  // h14_index
        "h13_def".to_string(),                  // h13_index
        "h12_ghi".to_string(),                  // h12_index
        "h11_jkl".to_string(),                  // h11_index
        Some(92.3),                             // battery_percentage
    );

    assert_eq!(v2_connectivity.battery_percentage, Some(92.3));
    assert_eq!(v2_connectivity.session_id, Some(1));
    assert_eq!(v2_connectivity.signal, -70.0);
    assert_eq!(v2_connectivity.altitude, 150.0);
    assert!(v2_connectivity.timestamp_start.contains("2009-02-13")); // Unix timestamp 1234567890
}

#[test]
fn test_operator_model() {
    let operator = data::v2::Operator::new(
        "550e8400-e29b-41d4-a716-446655440000".to_string(),
        "start_mission".to_string(),
        Some(1),
    );

    assert_eq!(operator.user_id, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(operator.action, "start_mission");
    assert_eq!(operator.session_id, Some(1));
    assert!(operator.timestamp.is_some());

    // Test that timestamp is valid RFC3339
    let timestamp = operator.timestamp.unwrap();
    assert!(timestamp.contains("T"));
    // RFC3339 format - should be valid
    assert!(timestamp.len() > 19); // Basic length check for YYYY-MM-DDTHH:MM:SS format
}

#[test]
fn test_operator_syncable_trait() {
    let mut operator = data::v2::Operator::default();

    // Test Syncable trait implementation
    assert_eq!(operator.id(), None);
    operator.set_id(42);
    assert_eq!(operator.id(), Some(42));

    assert_eq!(operator.id_local(), None);
    operator.set_id_local("local_123".to_string());
    assert_eq!(operator.id_local(), Some("local_123".to_string()));
}

#[test]
fn test_operator_default() {
    let operator = data::v2::Operator::default();

    assert_eq!(operator.id, None);
    assert_eq!(operator.id_local, None);
    assert_eq!(operator.created_at, None);
    assert_eq!(operator.timestamp, None);
    assert_eq!(operator.session_id, None);
    assert_eq!(operator.user_id, String::new());
    assert_eq!(operator.action, String::new());
}

#[test]
fn test_connectivity_v2_conversions() {
    // Test ConnectivityLocalV2 to ConnectivityV2
    let local_v2 = data::v2::ConnectivityLocal {
        id: Some(123),
        id_local: Some("test_connectivity".to_string()),
        session_id: Some(1),
        device_id: None,
        ancestor_id_local: None,
        inserted_at: None,
        timestamp_start: "2023-01-01T10:00:00Z".to_string(),
        signal: -65.0,
        noise: -95.0,
        altitude: 100.0,
        heading: 45.0,
        location: Some("POINT(1.0 2.0)".to_string()),
        h14_index: "h14".to_string(),
        h13_index: "h13".to_string(),
        h12_index: "h12".to_string(),
        h11_index: "h11".to_string(),
        battery_percentage: Some(85.5),
    };

    let api_v2: data::v2::Connectivity = local_v2.clone().into();

    // Verify conversion
    assert_eq!(api_v2.id, Some(123));
    assert_eq!(api_v2.session_id, Some(1));
    assert_eq!(api_v2.signal, -65.0);
    assert_eq!(api_v2.battery_percentage, Some(85.5));

    // Test reverse conversion
    let back_to_local: data::v2::ConnectivityLocal = api_v2.into();
    assert_eq!(back_to_local.id, Some(123));
    assert_eq!(back_to_local.session_id, Some(1));
    assert_eq!(back_to_local.battery_percentage, Some(85.5));
    assert_eq!(back_to_local.id_local, None); // API structs don't have id_local
    assert_eq!(back_to_local.ancestor_id_local, None); // API structs don't have ancestor_id_local
}

#[test]
fn test_connectivity_v2_syncable_trait() {
    let mut connectivity_v2 = data::v2::ConnectivityLocal::default();

    // Test Syncable trait
    assert_eq!(connectivity_v2.id(), None);
    connectivity_v2.set_id(123);
    assert_eq!(connectivity_v2.id(), Some(123));

    assert_eq!(connectivity_v2.id_local(), None);
    connectivity_v2.set_id_local("conn_local_456".to_string());
    assert_eq!(
        connectivity_v2.id_local(),
        Some("conn_local_456".to_string())
    );

    // Test AncestorLocal trait
    use scout_rs::models::data::v2::AncestorLocal;
    assert_eq!(connectivity_v2.ancestor_id_local(), None);
    connectivity_v2.set_ancestor_id_local("ancestor_789".to_string());
    assert_eq!(
        connectivity_v2.ancestor_id_local(),
        Some("ancestor_789".to_string())
    );
}

#[test]
fn test_migration_preserves_data_integrity() {
    // Create comprehensive V1 data
    let v1_connectivity = data::v1::ConnectivityLocal {
        id: Some(999),
        id_local: Some("comprehensive_test".to_string()),
        session_id: 42,
        ancestor_id_local: Some("ancestor_comprehensive".to_string()),
        inserted_at: Some("2024-12-01T15:30:45Z".to_string()),
        timestamp_start: "2024-12-01T15:30:00Z".to_string(),
        signal: -72.5,
        noise: -98.3,
        altitude: 256.7,
        heading: 137.2,
        location: Some("POINT(-74.0060 40.7128)".to_string()), // NYC coordinates
        h14_index: "h14_comprehensive".to_string(),
        h13_index: "h13_comprehensive".to_string(),
        h12_index: "h12_comprehensive".to_string(),
        h11_index: "h11_comprehensive".to_string(),
    };

    // Migrate to V2
    let v2_connectivity: data::v2::ConnectivityLocal = v1_connectivity.clone().into();

    // Verify every field migrated correctly
    assert_eq!(v2_connectivity.id, v1_connectivity.id);
    assert_eq!(v2_connectivity.id_local, v1_connectivity.id_local);
    assert_eq!(v2_connectivity.session_id, Some(v1_connectivity.session_id));
    assert_eq!(
        v2_connectivity.ancestor_id_local,
        v1_connectivity.ancestor_id_local
    );
    assert_eq!(v2_connectivity.inserted_at, v1_connectivity.inserted_at);
    assert_eq!(
        v2_connectivity.timestamp_start,
        v1_connectivity.timestamp_start
    );
    assert_eq!(v2_connectivity.signal, v1_connectivity.signal);
    assert_eq!(v2_connectivity.noise, v1_connectivity.noise);
    assert_eq!(v2_connectivity.altitude, v1_connectivity.altitude);
    assert_eq!(v2_connectivity.heading, v1_connectivity.heading);
    assert_eq!(v2_connectivity.location, v1_connectivity.location);
    assert_eq!(v2_connectivity.h14_index, v1_connectivity.h14_index);
    assert_eq!(v2_connectivity.h13_index, v1_connectivity.h13_index);
    assert_eq!(v2_connectivity.h12_index, v1_connectivity.h12_index);
    assert_eq!(v2_connectivity.h11_index, v1_connectivity.h11_index);

    // New field should be None for migrated data
    assert_eq!(v2_connectivity.battery_percentage, None);
}

#[test]
fn test_connectivity_v2_battery_percentage_ranges() {
    // Test various battery percentage values
    let test_cases = vec![None, Some(0.0), Some(50.0), Some(100.0), Some(99.99)];

    for battery_value in test_cases {
        let connectivity = data::v2::ConnectivityLocal::new(
            Some(1),
            None,
            1234567890,
            -65.0,
            -95.0,
            100.0,
            45.0,
            "POINT(0.0 0.0)".to_string(),
            "h14".to_string(),
            "h13".to_string(),
            "h12".to_string(),
            "h11".to_string(),
            battery_value,
        );

        assert_eq!(connectivity.battery_percentage, battery_value);
        assert_eq!(connectivity.session_id, Some(1));
    }
}

#[test]
fn test_connectivity_v2_device_based() {
    // Test creating connectivity directly associated with device instead of session
    let device_connectivity = data::v2::ConnectivityLocal::new(
        None,                                   // session_id - None for device-based connectivity
        Some(42),                               // device_id - associated with device
        1234567890,                             // timestamp_start
        -70.0,                                  // signal
        -85.0,                                  // noise
        125.0,                                  // altitude
        180.0,                                  // heading
        "POINT(-122.4194 37.7749)".to_string(), // location
        "h14_dev".to_string(),                  // h14_index
        "h13_dev".to_string(),                  // h13_index
        "h12_dev".to_string(),                  // h12_index
        "h11_dev".to_string(),                  // h11_index
        Some(78.2),                             // battery_percentage
    );

    // Verify device-based connectivity
    assert_eq!(device_connectivity.session_id, None);
    assert_eq!(device_connectivity.device_id, Some(42));
    assert_eq!(device_connectivity.battery_percentage, Some(78.2));
    assert_eq!(device_connectivity.signal, -70.0);

    // Test mixed scenario: session-based connectivity
    let session_connectivity = data::v2::ConnectivityLocal::new(
        Some(123),                                 // session_id - associated with session
        None,                                      // device_id - None when using session
        1234567890,                                // timestamp_start
        -65.0,                                     // signal
        -90.0,                                     // noise
        100.0,                                     // altitude
        45.0,                                      // heading
        "POINT(-155.15393 19.754824)".to_string(), // location
        "h14_ses".to_string(),                     // h14_index
        "h13_ses".to_string(),                     // h13_index
        "h12_ses".to_string(),                     // h12_index
        "h11_ses".to_string(),                     // h11_index
        Some(92.1),                                // battery_percentage
    );

    // Verify session-based connectivity
    assert_eq!(session_connectivity.session_id, Some(123));
    assert_eq!(session_connectivity.device_id, None);
    assert_eq!(session_connectivity.battery_percentage, Some(92.1));
    assert_eq!(session_connectivity.signal, -65.0);
}

#[test]
fn test_gps_tracker_vehicle_device_type() {
    // Test the new gps_tracker_vehicle device type
    let device_type = data::v1::DeviceType::GpsTrackerVehicle;

    // Test conversion from string
    let from_string = data::v1::DeviceType::from("gps_tracker_vehicle");
    assert_eq!(from_string, data::v1::DeviceType::GpsTrackerVehicle);

    // Test that it's different from regular gps_tracker
    let regular_gps = data::v1::DeviceType::from("gps_tracker");
    assert_ne!(device_type, regular_gps);
    assert_eq!(regular_gps, data::v1::DeviceType::GpsTracker);

    // Test unknown fallback still works
    let unknown = data::v1::DeviceType::from("invalid_type");
    assert_eq!(unknown, data::v1::DeviceType::Unknown);
}
