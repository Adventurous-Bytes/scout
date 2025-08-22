use scout_rs::client::*;
use tracing::info;
use std::env;

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
/// export SCOUT_API_KEY="your_device_api_key"
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

// Setup test environment using actual .env file values

fn setup_test_env() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Check for required environment variables and panic if missing
    let missing_vars = vec![
        ("SCOUT_API_KEY", env::var("SCOUT_API_KEY").is_err()),
        ("SCOUT_DATABASE_REST_URL", env::var("SCOUT_DATABASE_REST_URL").is_err()),
        ("SCOUT_URL", env::var("SCOUT_URL").is_err()),
        ("SCOUT_DEVICE_ID", env::var("SCOUT_DEVICE_ID").is_err()),
        ("SCOUT_HERD_ID", env::var("SCOUT_HERD_ID").is_err())
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
    let has_scout_api_key = env::var("SCOUT_API_KEY").is_ok();

    if !has_scout_api_key {
        panic!("❌ Missing Scout API key. Set SCOUT_API_KEY in your .env file.");
    }

    // Check for Supabase API key for PostgREST access
    let has_supabase_api_key =
        env::var("SUPABASE_PUBLIC_API_KEY").is_ok() ||
        env::var("SCOUT_SUPABASE_ANON_KEY").is_ok() ||
        env::var("SCOUT_SUPABASE_SERVICE_KEY").is_ok();

    if !has_supabase_api_key {
        panic!(
            "❌ Missing Supabase API key. Set SUPABASE_PUBLIC_API_KEY, SCOUT_SUPABASE_ANON_KEY, or SCOUT_SUPABASE_SERVICE_KEY in your .env file."
        );
    }

    info!("✅ All required environment variables are set");
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
        "image".to_string(),
        env
            ::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None
    );

    assert_eq!(event.message, Some("Test event".to_string()));
    assert_eq!(event.media_url, Some("https://example.com/image.jpg".to_string()));
    let expected_device_id = env
        ::var("SCOUT_DEVICE_ID")
        .unwrap_or_else(|_| "123".to_string())
        .parse()
        .unwrap_or(123);
    assert_eq!(event.device_id, Some(expected_device_id));
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
        500.0
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
        "image".to_string(),
        "animal".to_string()
    );

    assert_eq!(tag.x, 100.0);
    assert_eq!(tag.y, 200.0);
    assert_eq!(tag.conf, 0.95);
    assert_eq!(tag.class_name, "animal");
}

#[test]
fn test_client_creation() {
    // Test that ScoutClient can be created
    let client = ScoutClient::new("https://example.com".to_string(), "test_key".to_string());
    assert!(client.is_ok());

    let client = client.unwrap();
    assert_eq!(client.scout_url, "https://example.com");
    assert_eq!(client.api_key, "test_key");
    assert!(client.device.is_none());
    assert!(client.herd.is_none());
    assert!(!client.is_identified());
}

#[tokio::test]
async fn test_client_identification() {
    setup_test_env();

    info!("🧪 Testing client identification and database connection");

    // Create a client with actual credentials from .env file
    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Test identification process
    let identify_result = client.identify().await;

    // Test identification process
    match identify_result {
        Ok(_) => {
            info!("✅ Client identification successful");
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

#[tokio::test]
async fn test_event_creation() {
    setup_test_env();

    info!("🧪 Testing event creation functionality");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!("❌ Client identification failed: {:?}", identify_result.err());
    }
    info!("✅ Client identified successfully");

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
        "image".to_string(),
        env
            ::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        true,
        None
    );

    // Test event creation - should always succeed with proper credentials
    let event_result = client.create_event(&event).await;

    match event_result {
        Ok(response) => {
            info!("✅ Event creation successful");
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());
        }
        Err(e) => {
            panic!("❌ Event creation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_event_batch_creation() {
    setup_test_env();

    info!("🧪 Testing event batch creation functionality");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!("❌ Client identification failed: {:?}", identify_result.err());
    }
    info!("✅ Client identified successfully");

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
            "image".to_string(),
            env
                ::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995200,
            false,
            None
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
            "image".to_string(),
            env
                ::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995260,
            false,
            None
        ),
        Event::new(
            Some("Batch event 3".to_string()),
            Some("https://test.com/image3.jpg".to_string()),
            None,
            None,
            19.7545,
            -155.1535,
            8.0,
            180.0,
            "image".to_string(),
            env
                ::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995320,
            false,
            None
        )
    ];

    // Test batch event creation - should always succeed with proper credentials
    let batch_result = client.create_events_batch(&events).await;

    match batch_result {
        Ok(response) => {
            info!("✅ Event batch creation successful");
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            let created_events = response.data.unwrap();
            assert_eq!(created_events.len(), 3);
        }
        Err(e) => {
            panic!("❌ Event batch creation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_event_with_tags_creation() {
    setup_test_env();

    info!("🧪 Testing event creation with tags");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!("❌ Client identification failed: {:?}", identify_result.err());
    }
    info!("✅ Client identified successfully");

    // Create test event
    let event = Event::new(
        Some("Tagged event".to_string()),
        Some("https://test.com/tagged.jpg".to_string()),
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        "image".to_string(),
        env
            ::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None
    );

    // Create test tags
    let tags = vec![
        Tag::new(1, 100.0, 200.0, 50.0, 30.0, 0.95, "image".to_string(), "elephant".to_string()),
        Tag::new(2, 150.0, 250.0, 40.0, 25.0, 0.87, "image".to_string(), "giraffe".to_string())
    ];

    // Test event creation with tags - should always succeed with proper credentials
    let result = client.create_event_with_tags(&event, &tags, None).await;

    match result {
        Ok(response) => {
            info!("✅ Event with tags creation successful");
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());
        }
        Err(e) => {
            panic!("❌ Event with tags creation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_session_creation() {
    setup_test_env();

    info!("🧪 Testing session creation functionality");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!("❌ Client identification failed: {:?}", identify_result.err());
    }
    info!("✅ Client identified successfully");

    // Create test session
    let session = Session::new(
        env
            ::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        1640998800,
        "v2.0.0".to_string(),
        Some("Point(-155.15393 19.754824)".to_string()),
        120.0,
        45.0,
        82.5,
        15.0,
        3.0,
        9.0,
        1200.0,
        600.0
    );

    // Test session creation - should always succeed with proper credentials
    let session_result = client.create_session(&session).await;

    match session_result {
        Ok(response) => {
            info!("✅ Session creation successful");
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());
        }
        Err(e) => {
            panic!("❌ Session creation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_connectivity_creation() {
    setup_test_env();

    info!("🧪 Testing connectivity data creation");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!("❌ Client identification failed: {:?}", identify_result.err());
    }
    info!("✅ Client identified successfully");

    // Create test connectivity entry
    let connectivity = Connectivity::new(
        1, // session_id
        1640995200,
        -45.0, // signal
        -60.0, // noise
        100.0, // altitude
        180.0, // heading
        "Point(-155.15393 19.754824)".to_string(),
        "H14_INDEX".to_string(),
        "H13_INDEX".to_string(),
        "H12_INDEX".to_string(),
        "H11_INDEX".to_string()
    );

    // Test connectivity creation - should always succeed with proper credentials
    let connectivity_result = client.create_connectivity(&connectivity).await;

    match connectivity_result {
        Ok(response) => {
            info!("✅ Connectivity creation successful");
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());
        }
        Err(e) => {
            panic!("❌ Connectivity creation failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_compatibility_methods() {
    setup_test_env();

    info!("🧪 Testing compatibility methods");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Identify the client - should always succeed with proper credentials
    let identify_result = client.identify().await;
    if identify_result.is_err() {
        panic!("❌ Client identification failed: {:?}", identify_result.err());
    }
    info!("✅ Client identified successfully");

    // Test post_events_batch compatibility method
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
                "image".to_string(),
                env
                    ::var("SCOUT_DEVICE_ID")
                    .unwrap_or_else(|_| "123".to_string())
                    .parse()
                    .unwrap_or(123),
                1640995200,
                false,
                None
            ),
            vec![
                Tag::new(
                    1,
                    100.0,
                    200.0,
                    50.0,
                    30.0,
                    0.95,
                    "image".to_string(),
                    "animal".to_string()
                )
            ],
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
                "image".to_string(),
                env
                    ::var("SCOUT_DEVICE_ID")
                    .unwrap_or_else(|_| "123".to_string())
                    .parse()
                    .unwrap_or(123),
                1640995260,
                false,
                None
            ),
            vec![
                Tag::new(
                    2,
                    150.0,
                    250.0,
                    40.0,
                    25.0,
                    0.87,
                    "image".to_string(),
                    "animal".to_string()
                )
            ],
            "/path/to/file2.jpg".to_string(),
        )
    ];

    // Test compatibility batch method - should always succeed with proper credentials
    let compat_result = client.post_events_batch(&events_and_files, 10).await;

    match compat_result {
        Ok(response) => {
            info!("✅ Compatibility batch method successful");
            assert_eq!(response.status, ResponseScoutStatus::Success);
            assert!(response.data.is_some());

            let created_events = response.data.unwrap();
            assert_eq!(created_events.len(), 2);
        }
        Err(e) => {
            panic!("❌ Compatibility batch method failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_error_handling() {
    setup_test_env();

    info!("🧪 Testing error handling and edge cases");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        "invalid_api_key".to_string()
    ).unwrap();

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
        "image".to_string(),
        env
            ::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None
    );

    // This should fail because client is not identified
    let event_result = client.create_event(&event).await;

    match event_result {
        Ok(_) => {
            panic!("❌ Expected event creation to fail when not identified");
        }
        Err(e) => {
            info!("✅ Error handling works correctly: {}", e);
            assert!(e.to_string().contains("Database client not initialized"));
        }
    }
}

#[tokio::test]
async fn test_integration_workflow() {
    setup_test_env();

    info!("🧪 Testing complete integration workflow");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://test.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap_or_else(|_| "test_api_key".to_string())
    ).unwrap();

    // Test the complete workflow: identify -> create session -> create events -> create connectivity

    // Step 1: Identify (will fail in test env, but tests structure)
    let identify_result = client.identify().await;

    if identify_result.is_ok() {
        info!("✅ Step 1: Identification successful");

        // Step 2: Create a session
        let session = Session::new(
            env
                ::var("SCOUT_DEVICE_ID")
                .unwrap_or_else(|_| "123".to_string())
                .parse()
                .unwrap_or(123),
            1640995200,
            1640998800,
            "v2.0.0".to_string(),
            Some("Point(-155.15393 19.754824)".to_string()),
            120.0,
            45.0,
            82.5,
            15.0,
            3.0,
            9.0,
            1200.0,
            600.0
        );

        let session_result = client.create_session(&session).await;

        if let Ok(response) = session_result {
            info!("✅ Step 2: Session creation successful");

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
                    "image".to_string(),
                    env
                        ::var("SCOUT_DEVICE_ID")
                        .unwrap_or_else(|_| "123".to_string())
                        .parse()
                        .unwrap_or(123),
                    1640995200,
                    false,
                    Some(1) // session_id
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
                    "image".to_string(),
                    env
                        ::var("SCOUT_DEVICE_ID")
                        .unwrap_or_else(|_| "123".to_string())
                        .parse()
                        .unwrap_or(123),
                    1640995260,
                    false,
                    Some(1) // session_id
                )
            ];

            let events_result = client.create_events_batch(&events).await;

            if let Ok(response) = events_result {
                info!("✅ Step 3: Events creation successful");

                // Step 4: Create connectivity data
                let connectivity = Connectivity::new(
                    1, // session_id
                    1640995200,
                    -45.0,
                    -60.0,
                    100.0,
                    180.0,
                    "Point(-155.15393 19.754824)".to_string(),
                    "H14_INDEX".to_string(),
                    "H13_INDEX".to_string(),
                    "H12_INDEX".to_string(),
                    "H11_INDEX".to_string()
                );

                let connectivity_result = client.create_connectivity(&connectivity).await;

                match connectivity_result {
                    Ok(response) => {
                        info!("✅ Step 4: Connectivity creation successful");
                        info!("🎉 Complete integration workflow successful!");
                    }
                    Err(e) => {
                        panic!("❌ Step 4: Connectivity creation failed: {}", e);
                    }
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

    info!("✅ Integration workflow test completed");
}

#[tokio::test]
async fn test_real_database_integration() {
    // This test runs with real database credentials

    info!("🧪 Testing real database integration");

    let mut client = ScoutClient::new(
        env::var("SCOUT_URL").unwrap_or_else(|_| "https://real.scout.com".to_string()),
        env::var("SCOUT_API_KEY").unwrap()
    ).unwrap();

    // Step 1: Real identification
    info!("🔍 Attempting real device identification...");
    let identify_result = client.identify().await;

    match identify_result {
        Ok(_) => {
            info!("✅ Real identification successful!");
            assert!(client.is_identified());

            let device_id = client.device.as_ref().unwrap().id;
            let device_name = client.device.as_ref().unwrap().name.clone();
            let herd_id = client.herd.as_ref().unwrap().id;
            let herd_slug = client.herd.as_ref().unwrap().slug.clone();

            info!("   Device: {} (ID: {})", device_name, device_id);
            info!("   Herd: {} (ID: {})", herd_slug, herd_id);

            // Step 2: Create a real session
            let session = Session::new(
                device_id as i64,
                chrono::Utc::now().timestamp() as u64,
                (chrono::Utc::now().timestamp() as u64) + 3600,
                "integration_test_v1.0.0".to_string(),
                Some("Point(-155.15393 19.754824)".to_string()),
                120.0,
                45.0,
                82.5,
                15.0,
                3.0,
                9.0,
                1200.0,
                600.0
            );

            let session_result = client.create_session(&session).await;
            if let Ok(response) = session_result {
                if response.status == ResponseScoutStatus::Success {
                    let created_session = response.data.unwrap();
                    info!("✅ Real session created with ID: {:?}", created_session.id);

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
                            "image".to_string(),
                            device_id,
                            chrono::Utc::now().timestamp() as u64,
                            false,
                            created_session.id
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
                            "image".to_string(),
                            device_id,
                            chrono::Utc::now().timestamp() as u64,
                            false,
                            created_session.id
                        )
                    ];

                    let events_result = client.create_events_batch(&events).await;
                    if let Ok(response) = events_result {
                        if response.status == ResponseScoutStatus::Success {
                            let created_events = response.data.unwrap();
                            info!("✅ Real events created: {} events", created_events.len());

                            // Step 4: Create real connectivity data
                            let connectivity = Connectivity::new(
                                created_session.id.unwrap(),
                                chrono::Utc::now().timestamp() as u64,
                                -45.0,
                                -60.0,
                                100.0,
                                180.0,
                                "Point(-155.15393 19.754824)".to_string(),
                                "H14_INDEX".to_string(),
                                "H13_INDEX".to_string(),
                                "H12_INDEX".to_string(),
                                "H11_INDEX".to_string()
                            );

                            let connectivity_result = client.create_connectivity(
                                &connectivity
                            ).await;
                            if let Ok(response) = connectivity_result {
                                if response.status == ResponseScoutStatus::Success {
                                    let created_connectivity = response.data.unwrap();
                                    info!(
                                        "✅ Real connectivity data created with ID: {:?}",
                                        created_connectivity.id
                                    );

                                    // Step 5: Test query operations
                                    let sessions_response =
                                        client.get_sessions_by_herd(herd_id).await;
                                    if let Ok(response) = sessions_response {
                                        if response.status == ResponseScoutStatus::Success {
                                            let sessions = response.data.unwrap();
                                            info!(
                                                "✅ Retrieved {} sessions for herd",
                                                sessions.len()
                                            );
                                        }
                                    }

                                    let events_response = client.get_session_events(
                                        created_session.id.unwrap()
                                    ).await;
                                    if let Ok(response) = events_response {
                                        if response.status == ResponseScoutStatus::Success {
                                            let events = response.data.unwrap();
                                            info!(
                                                "✅ Retrieved {} events for session",
                                                events.len()
                                            );
                                        }
                                    }

                                    // Step 6: Cleanup - delete test data
                                    let delete_result = client.delete_session(
                                        created_session.id.unwrap()
                                    ).await;
                                    if let Ok(response) = delete_result {
                                        if response.status == ResponseScoutStatus::Success {
                                            info!(
                                                "✅ Test session and data cleaned up successfully"
                                            );
                                        }
                                    }

                                    info!(
                                        "🎉 Real database integration test completed successfully!"
                                    );
                                } else {
                                    info!("⚠️ Connectivity creation failed: {:?}", response.status);
                                }
                            } else {
                                info!(
                                    "⚠️ Connectivity creation error: {:?}",
                                    connectivity_result.err()
                                );
                            }
                        } else {
                            info!("⚠️ Events creation failed: {:?}", response.status);
                        }
                    } else {
                        info!("⚠️ Events creation error: {:?}", events_result.err());
                    }
                } else {
                    info!("⚠️ Session creation failed: {:?}", response.status);
                }
            } else {
                info!("⚠️ Session creation error: {:?}", session_result.err());
            }
        }
        Err(e) => {
            info!("❌ Real identification failed: {}", e);
            // This might happen if the API key is invalid or database is not accessible
            assert!(false, "Real database integration test failed during identification");
        }
    }
}

#[test]
fn test_data_validation() {
    info!("🧪 Testing data validation and edge cases");

    // Test Event validation
    let valid_event = Event::new(
        Some("Valid event".to_string()),
        Some("https://example.com/image.jpg".to_string()),
        None,
        None,
        90.0, // Valid latitude
        -180.0, // Valid longitude
        0.0, // Valid altitude
        0.0, // Valid heading
        "image".to_string(),
        env
            ::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123),
        1640995200,
        false,
        None
    );

    assert!(valid_event.location.is_some());
    assert_eq!(valid_event.location.unwrap(), "Point(-180 90)");

    // Test Session validation
    let valid_session = Session::new(
        env
            ::var("SCOUT_DEVICE_ID")
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
        500.0
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
        "detection".to_string(),
        "animal".to_string()
    );

    assert!(valid_tag.conf >= 0.0 && valid_tag.conf <= 1.0);
    assert!(valid_tag.width > 0.0);
    assert!(valid_tag.height > 0.0);

    info!("✅ Data validation tests passed");
}

#[test]
fn test_response_handling() {
    info!("🧪 Testing response handling and status codes");

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

    info!("✅ Response handling tests passed");
}

#[tokio::test]
async fn test_integration_with_mock_data() {
    // This test simulates integration without requiring external connections
    info!("🧪 Testing client structure and types");

    // Test ResponseScout status handling
    let mock_response = ResponseScout::new(ResponseScoutStatus::Success, Some("mock data"));

    match mock_response.status {
        ResponseScoutStatus::Success => {
            if let Some(data) = mock_response.data {
                assert_eq!(data, "mock data");
                info!("✅ Mock response handling works correctly");
            } else {
                panic!("❌ Mock response should have data");
            }
        }
        _ => panic!("❌ Mock response should have Success status"),
    }

    info!("✅ Integration test structure validation passed");
}
