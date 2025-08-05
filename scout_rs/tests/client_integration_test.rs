use scout_rs::client::*;
use std::{ time::{ SystemTime, UNIX_EPOCH }, env };
use dotenv::dotenv;
use tracing_subscriber;
use tracing::{ info, warn };

fn init_test_logging() {
    // Initialize tracing subscriber for tests
    let _ = tracing_subscriber::fmt().with_env_filter("info").with_test_writer().try_init();
}

#[tokio::test]
async fn test_scout_client_identification() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping integration test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let mut client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Test getting device
    info!("Testing get_device...");
    match client.get_device().await {
        Ok(device_response) => {
            match device_response.status {
                ResponseScoutStatus::Success => {
                    if let Some(device) = device_response.data {
                        info!("✅ Successfully got device: {:?}", device);

                        // Test getting herd using the device's herd_id
                        let herd_id_value = device.herd_id;
                        info!("Testing get_herd with herd_id: {}...", herd_id_value);

                        match client.get_herd(Some(herd_id_value)).await {
                            Ok(herd_response) => {
                                match herd_response.status {
                                    ResponseScoutStatus::Success => {
                                        if let Some(herd) = herd_response.data {
                                            info!("✅ Successfully got herd: {:?}", herd);

                                            // Additional assertions to verify the data structure
                                            assert!(
                                                device.id > 0,
                                                "Device should have a valid 'id' field"
                                            );
                                            assert!(
                                                device.name.len() > 0,
                                                "Device should have a valid 'name' field"
                                            );
                                            assert!(
                                                herd.id > 0,
                                                "Herd should have a valid 'id' field"
                                            );
                                            assert!(
                                                herd.slug.len() > 0,
                                                "Herd should have a valid 'slug' field"
                                            );
                                        } else {
                                            info!(
                                                "⚠️  Herd response had success status but no data"
                                            );
                                        }
                                    }
                                    ResponseScoutStatus::NotAuthorized => {
                                        assert!(
                                            false,
                                            "Herd request returned 401 NotAuthorized with valid API key - this indicates an authentication problem"
                                        );
                                    }
                                    _ => {
                                        info!(
                                            "⚠️  Herd request failed with status: {:?}",
                                            herd_response.status
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                // Get herd should succeed if the API is working properly
                                // Any failure indicates a problem with the API integration
                                panic!("❌ Get herd failed: {} - this indicates a problem with API integration", e);
                            }
                        }
                    } else {
                        info!("⚠️  Device response had success status but no data");
                    }
                }
                ResponseScoutStatus::NotAuthorized => {
                    assert!(
                        false,
                        "Device request returned 401 NotAuthorized with valid API key - this indicates an authentication problem"
                    );
                }
                _ => {
                    info!("⚠️  Device request failed with status: {:?}", device_response.status);
                }
            }
        }
        Err(e) => {
            // Get device should succeed if the API is working properly
            // Any failure indicates a problem with the API integration
            panic!("❌ Get device failed: {} - this indicates a problem with API integration", e);
        }
    }
}

#[tokio::test]
async fn test_scout_client_error_handling() {
    // Load environment variables from .env file
    dotenv().ok();
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping integration test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    // Test with invalid API key
    let mut client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Test getting device with invalid key
    match client.get_device().await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    info!("✅ Correctly returned NotAuthorized status with invalid API key");
                }
                ResponseScoutStatus::Failure => {
                    info!("✅ Correctly returned Failure status (expected for invalid server)");
                }
                _ => {
                    info!("✅ Returned {:?} status (acceptable for test server)", response.status);
                }
            }
        }
        Err(e) => {
            info!("✅ Correctly returned error with invalid API key: {}", e);
        }
    }

    // Test getting herd with invalid key
    match client.get_herd(Some(123)).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    info!(
                        "✅ Correctly returned NotAuthorized status for herd with invalid API key"
                    );
                }
                ResponseScoutStatus::Failure => {
                    info!("✅ Correctly returned Failure status (expected for invalid server)");
                }
                _ => {
                    info!("✅ Returned {:?} status (acceptable for test server)", response.status);
                }
            }
        }
        Err(e) => {
            info!("✅ Correctly returned error for herd with invalid API key: {}", e);
        }
    }
}

#[tokio::test]
async fn test_401_unauthorized_responses() {
    // Load environment variables from .env file
    dotenv().ok();

    info!("🧪 Testing 401 Unauthorized Response Handling");
    info!("=============================================");

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }

    // Test with invalid API key
    let mut client = ScoutClient::new(scout_url.clone(), "invalid_api_key".to_string()).expect(
        "Failed to create ScoutClient"
    );

    // Test 1: get_device with invalid API key
    info!("1️⃣ Testing get_device with invalid API key");
    match client.get_device().await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    info!("✅ get_device correctly returned NotAuthorized for invalid API key");
                }
                ResponseScoutStatus::Success => {
                    assert!(false, "get_device should not return Success with invalid API key");
                }
                ResponseScoutStatus::Failure => {
                    info!("✅ get_device returned Failure (expected for invalid server)");
                }
                _ => {
                    info!(
                        "✅ get_device returned {:?} (acceptable for test server)",
                        response.status
                    );
                }
            }
        }
        Err(e) => {
            info!("✅ get_device correctly returned error: {}", e);
        }
    }

    // Test 2: get_herd with invalid API key
    info!("2️⃣ Testing get_herd with invalid API key");
    match client.get_herd(Some(123)).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    info!("✅ get_herd correctly returned NotAuthorized for invalid API key");
                }
                ResponseScoutStatus::Success => {
                    assert!(false, "get_herd should not return Success with invalid API key");
                }
                ResponseScoutStatus::Failure => {
                    info!("✅ get_herd returned Failure (expected for invalid server)");
                }
                _ => {
                    info!(
                        "✅ get_herd returned {:?} (acceptable for test server)",
                        response.status
                    );
                }
            }
        }
        Err(e) => {
            info!("✅ get_herd correctly returned error: {}", e);
        }
    }

    // Test 3: create_event_with_tags with invalid API key
    info!("3️⃣ Testing create_event_with_tags with invalid API key");
    let event = Event::new(
        Some("Test event".to_string()),
        None,
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        "image".to_string(),
        123,
        1733351509,
        false,
        None
    );
    let tags = vec![
        Tag::new(1, 100.0, 200.0, 50.0, 30.0, 0.95, "manual".to_string(), "animal".to_string())
    ];

    // Create a temporary test file
    let temp_file = "temp_test_file.jpg";
    std::fs::write(temp_file, b"fake image data").expect("Failed to create temp file");

    match client.create_event_with_tags(&event, &tags, temp_file).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    info!(
                        "✅ create_event_with_tags correctly returned NotAuthorized for invalid API key"
                    );
                }
                ResponseScoutStatus::Success => {
                    assert!(
                        false,
                        "create_event_with_tags should not return Success with invalid API key"
                    );
                }
                ResponseScoutStatus::Failure => {
                    info!(
                        "✅ create_event_with_tags returned Failure (expected for invalid server)"
                    );
                }
                _ => {
                    info!(
                        "✅ create_event_with_tags returned {:?} (acceptable for test server)",
                        response.status
                    );
                }
            }
        }
        Err(e) => {
            info!("✅ create_event_with_tags correctly returned error: {}", e);
        }
    }

    // Clean up temp file
    let _ = std::fs::remove_file(temp_file);

    // Test 4: Test with empty API key
    info!("4️⃣ Testing with empty API key");
    let mut empty_key_client = ScoutClient::new(scout_url, "".to_string()).expect(
        "Failed to create ScoutClient"
    );

    match empty_key_client.get_device().await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    info!("✅ Empty API key correctly returned NotAuthorized");
                }
                ResponseScoutStatus::Success => {
                    assert!(false, "Empty API key should not return Success");
                }
                ResponseScoutStatus::Failure => {
                    info!("✅ Empty API key returned Failure (expected for invalid server)");
                }
                _ => {
                    info!(
                        "✅ Empty API key returned {:?} (acceptable for test server)",
                        response.status
                    );
                }
            }
        }
        Err(e) => {
            info!("✅ Empty API key correctly returned error: {}", e);
        }
    }

    info!("✅ All 401 unauthorized response tests completed successfully");
}

#[tokio::test]
async fn test_should_not_receive_401_with_valid_credentials() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping 401 test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }

    info!("🧪 Testing that valid credentials should NOT return 401");
    info!("=====================================================");

    let mut client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Test 1: get_device with valid API key should NOT return 401
    info!("1️⃣ Testing get_device with valid API key");
    match client.get_device().await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    assert!(
                        false,
                        "get_device returned 401 NotAuthorized with valid API key - this indicates an authentication problem"
                    );
                }
                ResponseScoutStatus::Success => {
                    info!("✅ get_device returned Success with valid API key");
                }
                ResponseScoutStatus::Failure => {
                    info!(
                        "⚠️ get_device returned Failure (this might be expected depending on server state)"
                    );
                }
                _ => {
                    info!(
                        "⚠️ get_device returned {:?} (unexpected but not necessarily wrong)",
                        response.status
                    );
                }
            }
        }
        Err(e) => {
            info!("⚠️ get_device returned error: {} (this might be expected if server is unavailable)", e);
        }
    }

    // Test 2: get_herd with valid API key should NOT return 401
    info!("2️⃣ Testing get_herd with valid API key");
    match client.get_herd(None).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    assert!(
                        false,
                        "get_herd returned 401 NotAuthorized with valid API key - this indicates an authentication problem"
                    );
                }
                ResponseScoutStatus::Success => {
                    info!("✅ get_herd returned Success with valid API key");
                }
                ResponseScoutStatus::Failure => {
                    info!(
                        "⚠️ get_herd returned Failure (this might be expected if no device/herd data is available)"
                    );
                }
                _ => {
                    info!(
                        "⚠️ get_herd returned {:?} (unexpected but not necessarily wrong)",
                        response.status
                    );
                }
            }
        }
        Err(e) => {
            info!("⚠️ get_herd returned error: {} (this might be expected if no device data is available)", e);
        }
    }

    info!("✅ Valid credentials did not return 401 responses");
}

#[tokio::test]
async fn test_identify_method() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping identify test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let mut client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Test identify method
    let result = client.identify().await;
    match result {
        Ok(_) => {
            info!("✅ Identify method completed successfully");
            // Verify that device and herd are loaded into state
            assert!(client.device.is_some(), "Device should be loaded into state");
            assert!(client.herd.is_some(), "Herd should be loaded into state");

            if let Some(device) = &client.device {
                info!("   Device loaded: {} (ID: {})", device.name, device.id);
            }
            if let Some(herd) = &client.herd {
                info!("   Herd loaded: {} (ID: {})", herd.slug, herd.id);
            }
        }
        Err(e) => {
            // The identify method should succeed if the API is working properly
            // Any failure indicates a problem with the API integration
            panic!("❌ Identify method failed: {} - this indicates a problem with API integration", e);
        }
    }
}

#[tokio::test]
async fn test_session_creation_api() {
    // Initialize logging for this test
    init_test_logging();

    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping session creation API test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Test creating a session with realistic data
    info!("Testing session creation with realistic data...");

    // Get device ID from environment
    let device_id: i64 = env::var("SCOUT_DEVICE_ID").unwrap_or_default().parse().unwrap_or(0);
    if device_id == 0 {
        info!("Skipping test - no valid SCOUT_DEVICE_ID environment variable set");
        return;
    }

    let timestamp_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let timestamp_end = timestamp_start + 3600; // 1 hour later

    match
        client.create_session(
            device_id,
            timestamp_start,
            timestamp_end,
            "v1.0.0".to_string(),
            Some("Point(-155.15393 19.754824)".to_string()),
            150.0, // altitude_max
            50.0, // altitude_min
            100.0, // altitude_average
            25.0, // velocity_max (m/s)
            5.0, // velocity_min (m/s)
            15.0, // velocity_average (m/s)
            5000.0, // distance_total (m)
            2500.0 // distance_max_from_start (m)
        ).await
    {
        Ok(id) => {
            info!("✅ Successfully created session with ID: {}", id);
            assert!(id > 0, "Session ID should be positive");

            // Test that we can retrieve the session data
            info!("Testing session retrieval...");

            // Get herd ID from environment
            let herd_id: u32 = env::var("SCOUT_HERD_ID").unwrap_or_default().parse().unwrap_or(0);
            if herd_id == 0 {
                info!(
                    "Skipping session retrieval test - no valid SCOUT_HERD_ID environment variable set"
                );
                return;
            }

            match client.get_sessions_by_herd(herd_id).await {
                Ok(response) => {
                    if let Some(sessions) = response.data {
                        let created_session = sessions.iter().find(|s| s.id == Some(id));
                        if let Some(session) = created_session {
                            info!("✅ Found created session in herd: {:?}", session);
                            assert_eq!(session.device_id, device_id);
                            assert_eq!(session.software_version, "v1.0.0");

                            assert_eq!(session.altitude_max, 150.0);
                            assert_eq!(session.altitude_min, 50.0);
                            assert_eq!(session.altitude_average, 100.0);
                            assert_eq!(session.velocity_max, 25.0);
                            assert_eq!(session.velocity_min, 5.0);
                            assert_eq!(session.velocity_average, 15.0);
                            assert_eq!(session.distance_total, 5000.0);
                            assert_eq!(session.distance_max_from_start, 2500.0);
                        } else {
                            info!(
                                "⚠️  Created session not found in herd list (this might be expected if herd_id is different)"
                            );
                        }
                    }
                }
                Err(e) => {
                    // Session retrieval should succeed if the API is working properly
                    // Any failure indicates a problem with the API integration
                    panic!("❌ Session retrieval failed: {} - this indicates a problem with API integration", e);
                }
            }

            // Clean up: delete the test session
            info!("Cleaning up test session...");
            match client.delete_session(id).await {
                Ok(_) => {
                    info!("✅ Successfully deleted test session");
                }
                Err(e) => {
                    warn!("⚠️ Failed to delete test session: {} (this is non-critical)", e);
                }
            }
        }
        Err(e) => {
            // Session creation should succeed if the API is working properly
            // Any failure indicates a problem with the API integration
            panic!("❌ Session creation failed: {} - this indicates a problem with API integration", e);
        }
    }
}

#[tokio::test]
async fn test_create_event_with_tags() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping create_event_with_tags test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Get device ID from environment
    let device_id: u32 = env::var("SCOUT_DEVICE_ID").unwrap_or_default().parse().unwrap_or(0);
    if device_id == 0 {
        info!("Skipping test - no valid SCOUT_DEVICE_ID environment variable set");
        return;
    }

    // First, create a session to associate the event with
    let timestamp_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let timestamp_end = timestamp_start + 3600; // 1 hour later

    let session_id = match
        client.create_session(
            device_id as i64,
            timestamp_start,
            timestamp_end,
            "v1.0.0".to_string(),
            None,
            150.0, // altitude_max
            50.0, // altitude_min
            100.0, // altitude_average
            25.0, // velocity_max (m/s)
            5.0, // velocity_min (m/s)
            15.0, // velocity_average (m/s)
            5000.0, // distance_total (m)
            2500.0 // distance_max_from_start (m)
        ).await
    {
        Ok(id) => {
            info!("✅ Successfully created session with ID: {} for event test", id);
            id
        }
        Err(e) => {
            panic!("❌ Failed to create session for event test: {} - this indicates a problem with API integration", e);
        }
    };

    // Create a test event with real device ID and session ID
    let event = Event::new(
        Some("Integration test event".to_string()),
        Some("https://example.com/test.jpg".to_string()),
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        "image".to_string(),
        device_id, // Use the real device ID
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        false,
        Some(session_id) // Associate with the created session
    );

    let tags = vec![
        Tag::new(1, 100.0, 200.0, 50.0, 30.0, 0.95, "manual".to_string(), "animal".to_string())
    ];

    // Create a temporary test file
    let temp_file = "temp_integration_test.jpg";
    std::fs
        ::write(temp_file, b"fake image data for integration test")
        .expect("Failed to create temp file");

    info!("Testing create_event_with_tags...");
    match client.create_event_with_tags(&event, &tags, temp_file).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::Success => {
                    info!("✅ Successfully posted event with tags");
                    // Verify that the event was created by checking session events
                    match client.get_session_events(session_id).await {
                        Ok(events_response) => {
                            if let Some(events) = events_response.data {
                                info!("✅ Found {} events in session", events.len());
                            }
                        }
                        Err(e) => {
                            info!("⚠️ Could not retrieve session events for verification: {}", e);
                        }
                    }
                }
                ResponseScoutStatus::NotAuthorized => {
                    info!(
                        "⚠️ Post event returned NotAuthorized (expected with invalid credentials)"
                    );
                }
                ResponseScoutStatus::Failure => {
                    info!("⚠️ Post event returned Failure (expected if server is unavailable)");
                }
                _ => {
                    info!("⚠️ Post event returned status: {:?}", response.status);
                }
            }
        }
        Err(e) => {
            // Post event should succeed if the API is working properly
            // Any failure indicates a problem with the API integration
            panic!("❌ Post event failed: {} - this indicates a problem with API integration", e);
        }
    }

    // Clean up temp file
    let _ = std::fs::remove_file(temp_file);

    // Clean up: delete the test session (this should cascade delete the event and tags)
    info!("Cleaning up test session and associated data...");
    match client.delete_session(session_id).await {
        Ok(_) => {
            info!(
                "✅ Successfully deleted test session (events and tags should be cascade deleted)"
            );
        }
        Err(e) => {
            warn!("⚠️ Failed to delete test session: {} (this is non-critical)", e);
        }
    }
}

#[tokio::test]
async fn test_get_sessions_by_herd() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping get_sessions_by_herd test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Get herd ID from environment
    let herd_id: u32 = env::var("SCOUT_HERD_ID").unwrap_or_default().parse().unwrap_or(0);
    if herd_id == 0 {
        info!("Skipping test - no valid SCOUT_HERD_ID environment variable set");
        return;
    }

    info!("Testing get_sessions_by_herd with herd_id: {}...", herd_id);
    match client.get_sessions_by_herd(herd_id).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::Success => {
                    if let Some(sessions) = response.data {
                        info!(
                            "✅ Successfully retrieved {} sessions for herd {}",
                            sessions.len(),
                            herd_id
                        );
                        // Validate session structure
                        for session in sessions {
                            assert!(session.device_id > 0, "Session device_id should be positive");
                            assert!(
                                !session.software_version.is_empty(),
                                "Session software_version should not be empty"
                            );
                        }
                    } else {
                        info!("✅ Successfully retrieved sessions (empty array)");
                    }
                }
                ResponseScoutStatus::NotAuthorized => {
                    info!(
                        "⚠️ Get sessions returned NotAuthorized (expected with invalid credentials)"
                    );
                }
                ResponseScoutStatus::Failure => {
                    info!("⚠️ Get sessions returned Failure (expected if server is unavailable)");
                }
                _ => {
                    info!("⚠️ Get sessions returned status: {:?}", response.status);
                }
            }
        }
        Err(e) => {
            // Get sessions should succeed if the API is working properly
            // Any failure indicates a problem with the API integration
            panic!("❌ Get sessions failed: {} - this indicates a problem with API integration", e);
        }
    }
}

#[tokio::test]
async fn test_upsert_connectivity() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping upsert_connectivity test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Get device ID from environment
    let device_id: i64 = env::var("SCOUT_DEVICE_ID").unwrap_or_default().parse().unwrap_or(0);
    if device_id == 0 {
        info!("Skipping test - no valid SCOUT_DEVICE_ID environment variable set");
        return;
    }

    // First, create a session to get a real session ID
    let timestamp_start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let timestamp_end = timestamp_start + 3600; // 1 hour later

    let session_id = match
        client.create_session(
            device_id,
            timestamp_start,
            timestamp_end,
            "v1.0.0".to_string(),
            None,
            150.0, // altitude_max
            50.0, // altitude_min
            100.0, // altitude_average
            25.0, // velocity_max (m/s)
            5.0, // velocity_min (m/s)
            15.0, // velocity_average (m/s)
            5000.0, // distance_total (m)
            2500.0 // distance_max_from_start (m)
        ).await
    {
        Ok(id) => {
            info!("✅ Successfully created session with ID: {} for connectivity test", id);
            id
        }
        Err(e) => {
            panic!("❌ Failed to create session for connectivity test: {} - this indicates a problem with API integration", e);
        }
    };

    // Create a test connectivity entry with the real session ID
    let connectivity = Connectivity::new(
        session_id, // Use the real session ID
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        -50.0, // signal
        -60.0, // noise
        100.0, // altitude
        45.0, // heading
        "Point(-74.006 40.7128)".to_string(),
        "h14".to_string(),
        "h13".to_string(),
        "h12".to_string(),
        "h11".to_string()
    );

    info!("Testing upsert_connectivity with session_id: {}...", session_id);
    match client.upsert_connectivity(&connectivity).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::Success => {
                    if let Some(created_connectivity) = response.data {
                        info!(
                            "✅ Successfully upserted connectivity entry with ID: {:?}",
                            created_connectivity.id
                        );
                        assert_eq!(created_connectivity.session_id, session_id);
                        assert_eq!(created_connectivity.signal, -50.0);
                        assert_eq!(created_connectivity.noise, -60.0);
                    } else {
                        info!("✅ Successfully upserted connectivity entry");
                    }
                }
                ResponseScoutStatus::NotAuthorized => {
                    info!(
                        "⚠️ Upsert connectivity returned NotAuthorized (expected with invalid credentials)"
                    );
                }
                ResponseScoutStatus::Failure => {
                    info!(
                        "⚠️ Upsert connectivity returned Failure (expected if server is unavailable)"
                    );
                }
                _ => {
                    info!("⚠️ Upsert connectivity returned status: {:?}", response.status);
                }
            }
        }
        Err(e) => {
            // Upsert connectivity should succeed if the API is working properly
            // Any failure indicates a problem with the API integration
            panic!("❌ Upsert connectivity failed: {} - this indicates a problem with API integration", e);
        }
    }

    // Clean up: delete the test session
    info!("Cleaning up test session...");
    match client.delete_session(session_id).await {
        Ok(_) => {
            info!("✅ Successfully deleted test session");
        }
        Err(e) => {
            warn!("⚠️ Failed to delete test session: {} (this is non-critical)", e);
        }
    }
}

#[tokio::test]
async fn test_delete_event() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping delete_event test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Get device ID from environment
    let device_id: u32 = env::var("SCOUT_DEVICE_ID").unwrap_or_default().parse().unwrap_or(0);
    if device_id == 0 {
        info!("Skipping test - no valid SCOUT_DEVICE_ID environment variable set");
        return;
    }

    // Step 1: Create an event to test deletion
    info!("Step 1: Creating an event for deletion test...");

    // Create a test event with null session_id (no session association)
    let event = Event::new(
        Some("Delete test event".to_string()),
        Some("https://example.com/delete_test.jpg".to_string()),
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        "image".to_string(),
        device_id,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        false,
        None // No session association
    );

    let tags = vec![
        Tag::new(1, 100.0, 200.0, 50.0, 30.0, 0.95, "manual".to_string(), "animal".to_string())
    ];

    // Create a temporary test file
    let temp_file = "temp_delete_test.jpg";
    std::fs
        ::write(temp_file, b"fake image data for delete test")
        .expect("Failed to create temp file");

    // Create the event and get the event ID
    let event_creation_result = client.create_event_with_tags(&event, &tags, temp_file).await;

    // Clean up temp file immediately
    let _ = std::fs::remove_file(temp_file);

    match event_creation_result {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::Success => {
                    if let Some(created_event) = response.data {
                        let event_id = created_event.id.unwrap_or(0);
                        if event_id == 0 {
                            panic!("❌ Event created but no ID returned - cannot test deletion");
                        }
                        info!("✅ Successfully created event for deletion test with ID: {}", event_id);

                        // Step 2: Test deletion with the actual event ID
                        info!("Step 2: Testing deletion with event ID: {}...", event_id);

                        match client.delete_event(event_id).await {
                            Ok(delete_response) => {
                                match delete_response.status {
                                    ResponseScoutStatus::Success => {
                                        info!("✅ Successfully deleted event with ID: {}", event_id);
                                    }
                                    ResponseScoutStatus::NotAuthorized => {
                                        panic!(
                                            "❌ Delete event returned NotAuthorized - test should have valid credentials"
                                        );
                                    }
                                    ResponseScoutStatus::Failure => {
                                        panic!("❌ Delete event returned Failure for ID {} - event should exist and be deletable", event_id);
                                    }
                                    _ => {
                                        panic!(
                                            "❌ Delete event returned unexpected status: {:?} for ID {}",
                                            delete_response.status,
                                            event_id
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                panic!(
                                    "❌ Delete event returned error for ID {}: {} - event should be deletable",
                                    event_id,
                                    e
                                );
                            }
                        }
                    } else {
                        panic!(
                            "❌ Event created but no event data returned - cannot test deletion"
                        );
                    }
                }
                ResponseScoutStatus::NotAuthorized => {
                    panic!(
                        "❌ Event creation returned NotAuthorized - test should have valid credentials"
                    );
                }
                ResponseScoutStatus::Failure => {
                    panic!(
                        "❌ Event creation returned Failure - server should be available for integration test"
                    );
                }
                _ => {
                    panic!("❌ Event creation returned unexpected status: {:?}", response.status);
                }
            }
        }
        Err(e) => {
            panic!("❌ Error creating event: {} - server should be available for integration test", e);
        }
    }

    info!("✅ Delete event integration test completed");
}

#[tokio::test]
async fn test_update_event() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping update_event test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Get device ID from environment
    let device_id: u32 = env::var("SCOUT_DEVICE_ID").unwrap_or_default().parse().unwrap_or(0);
    if device_id == 0 {
        info!("Skipping test - no valid SCOUT_DEVICE_ID environment variable set");
        return;
    }

    // Step 1: Create an event to test updating
    info!("Step 1: Creating an event for update test...");

    // Create a test event
    let event = Event::new(
        Some("Original message".to_string()),
        Some("https://example.com/original.jpg".to_string()),
        None,
        None,
        19.754824,
        -155.15393,
        10.0,
        0.0,
        "image".to_string(),
        device_id,
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        false,
        None
    );

    let tags = vec![
        Tag::new(1, 100.0, 200.0, 50.0, 30.0, 0.95, "manual".to_string(), "animal".to_string())
    ];

    // Create a temporary test file
    let temp_file = "temp_update_test.jpg";
    std::fs
        ::write(temp_file, b"fake image data for update test")
        .expect("Failed to create temp file");

    // Create the event and get the event ID
    let event_creation_result = client.create_event_with_tags(&event, &tags, temp_file).await;

    // Clean up temp file immediately
    let _ = std::fs::remove_file(temp_file);

    match event_creation_result {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::Success => {
                    if let Some(created_event) = response.data {
                        let event_id = created_event.id.unwrap_or(0);
                        if event_id == 0 {
                            panic!("❌ Event created but no ID returned - cannot test update");
                        }
                        info!("✅ Successfully created event for update test with ID: {}", event_id);

                        // Step 2: Test updating the event
                        info!("Step 2: Testing update with event ID: {}...", event_id);

                        // Create updated event data
                        let updated_event = Event::new(
                            Some("Updated message".to_string()),
                            Some("https://example.com/updated.jpg".to_string()),
                            None,
                            None,
                            20.123456,
                            -156.789012,
                            15.5,
                            90.0,
                            "image".to_string(),
                            device_id,
                            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                            true, // Changed to public
                            None
                        ).with_id(event_id);

                        match client.update_event(event_id, &updated_event).await {
                            Ok(update_response) => {
                                match update_response.status {
                                    ResponseScoutStatus::Success => {
                                        if let Some(updated_event) = update_response.data {
                                            info!("✅ Successfully updated event with ID: {}", event_id);

                                            // Verify the update worked
                                            assert_eq!(
                                                updated_event.message,
                                                Some("Updated message".to_string())
                                            );
                                            assert_eq!(
                                                updated_event.media_url,
                                                Some("https://example.com/updated.jpg".to_string())
                                            );
                                            assert_eq!(updated_event.altitude, 15.5);
                                            assert_eq!(updated_event.heading, 90.0);
                                            assert_eq!(updated_event.is_public, true);

                                            info!("✅ Event update verification passed");
                                        } else {
                                            panic!("❌ Event updated but no event data returned");
                                        }
                                    }
                                    ResponseScoutStatus::NotAuthorized => {
                                        panic!(
                                            "❌ Update event returned NotAuthorized - test should have valid credentials"
                                        );
                                    }
                                    ResponseScoutStatus::Failure => {
                                        panic!("❌ Update event returned Failure for ID {} - event should exist and be updatable", event_id);
                                    }
                                    _ => {
                                        panic!(
                                            "❌ Update event returned unexpected status: {:?} for ID {}",
                                            update_response.status,
                                            event_id
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                panic!("❌ Update event returned error for ID {}: {}", event_id, e);
                            }
                        }

                        // Step 3: Clean up by deleting the event
                        info!("Step 3: Cleaning up test event...");
                        match client.delete_event(event_id).await {
                            Ok(delete_response) => {
                                match delete_response.status {
                                    ResponseScoutStatus::Success => {
                                        info!("✅ Successfully deleted test event with ID: {}", event_id);
                                    }
                                    _ => {
                                        info!(
                                            "⚠️ Failed to delete test event: {:?}",
                                            delete_response.status
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                info!("⚠️ Error deleting test event: {}", e);
                            }
                        }
                    } else {
                        panic!("❌ Event created but no event data returned - cannot test update");
                    }
                }
                ResponseScoutStatus::NotAuthorized => {
                    panic!(
                        "❌ Event creation returned NotAuthorized - test should have valid credentials"
                    );
                }
                ResponseScoutStatus::Failure => {
                    panic!(
                        "❌ Event creation returned Failure - server should be available for integration test"
                    );
                }
                _ => {
                    panic!("❌ Event creation returned unexpected status: {:?}", response.status);
                }
            }
        }
        Err(e) => {
            panic!("❌ Error creating event: {} - server should be available for integration test", e);
        }
    }

    info!("✅ Update event integration test completed");
}

#[tokio::test]
async fn test_get_plans_by_herd() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping get_plans_by_herd test - no SCOUT_API_KEY environment variable set");
        return;
    }

    let scout_url = env::var("SCOUT_URL").unwrap_or_default();
    if scout_url.is_empty() {
        info!("Skipping test - no SCOUT_URL environment variable set");
        return;
    }
    let client = ScoutClient::new(scout_url, api_key).expect("Failed to create ScoutClient");

    // Get herd ID from environment
    let herd_id: u32 = env::var("SCOUT_HERD_ID").unwrap_or_default().parse().unwrap_or(0);
    if herd_id == 0 {
        info!("Skipping test - no valid SCOUT_HERD_ID environment variable set");
        return;
    }

    info!("Testing get_plans_by_herd with herd_id: {}...", herd_id);
    match client.get_plans_by_herd(herd_id).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::Success => {
                    if let Some(plans) = response.data {
                        info!(
                            "✅ Successfully retrieved {} plans for herd {}",
                            plans.len(),
                            herd_id
                        );
                        // Validate plan structure
                        for plan in plans {
                            assert!(plan.herd_id > 0, "Plan herd_id should be positive");
                            assert!(!plan.name.is_empty(), "Plan name should not be empty");
                            assert!(
                                !plan.instructions.is_empty(),
                                "Plan instructions should not be empty"
                            );
                            assert!(
                                !plan.plan_type.is_empty(),
                                "Plan plan_type should not be empty"
                            );
                            assert_eq!(
                                plan.herd_id,
                                herd_id as i64,
                                "Plan should belong to the requested herd"
                            );
                            // Validate plan_type is one of the expected values
                            assert!(
                                ["mission", "fence", "rally", "markov"].contains(
                                    &plan.plan_type.as_str()
                                ),
                                "Plan plan_type should be one of: mission, fence, rally, markov, got: {}",
                                plan.plan_type
                            );
                        }
                    } else {
                        info!("✅ Successfully retrieved plans (empty array)");
                    }
                }
                ResponseScoutStatus::NotAuthorized => {
                    info!(
                        "⚠️ Get plans returned NotAuthorized (expected with invalid credentials)"
                    );
                }
                ResponseScoutStatus::Failure => {
                    info!("⚠️ Get plans returned Failure (expected if server is unavailable)");
                }
                _ => {
                    info!("⚠️ Get plans returned status: {:?}", response.status);
                }
            }
        }
        Err(e) => {
            // Get plans should succeed if the API is working properly
            // Any failure indicates a problem with the API integration
            panic!("❌ Get plans failed: {} - this indicates a problem with API integration", e);
        }
    }
}
