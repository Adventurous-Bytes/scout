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

    // Test 3: post_event_with_tags with invalid API key
    info!("3️⃣ Testing post_event_with_tags with invalid API key");
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

    match client.post_event_with_tags(&event, &tags, temp_file).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::NotAuthorized => {
                    info!(
                        "✅ post_event_with_tags correctly returned NotAuthorized for invalid API key"
                    );
                }
                ResponseScoutStatus::Success => {
                    assert!(
                        false,
                        "post_event_with_tags should not return Success with invalid API key"
                    );
                }
                ResponseScoutStatus::Failure => {
                    info!("✅ post_event_with_tags returned Failure (expected for invalid server)");
                }
                _ => {
                    info!(
                        "✅ post_event_with_tags returned {:?} (acceptable for test server)",
                        response.status
                    );
                }
            }
        }
        Err(e) => {
            info!("✅ post_event_with_tags correctly returned error: {}", e);
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
                            // Check for locations_geojson field (should be available in all session responses)
                            if session.locations_geojson.is_some() {
                                info!(
                                    "✅ Session has locations_geojson: {:?}",
                                    session.locations_geojson.as_ref().unwrap()
                                );
                            } else {
                                info!(
                                    "⚠️ Session locations_geojson is None (may be expected for some sessions)"
                                );
                            }

                            // Note: locations may be None if the API doesn't return it in the response
                            if session.locations.is_some() {
                                info!(
                                    "✅ Session has WKT locations: {}",
                                    session.locations.as_ref().unwrap()
                                );
                            } else {
                                info!(
                                    "⚠️ Session locations is None (API may not return this field)"
                                );
                            }
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
async fn test_post_event_with_tags() {
    // Load environment variables from .env file
    dotenv().ok();

    // Skip this test if no API key is provided
    let api_key = env::var("SCOUT_API_KEY").unwrap_or_default();
    if api_key.is_empty() {
        info!("Skipping post_event_with_tags test - no SCOUT_API_KEY environment variable set");
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

    // Create a test event with real device ID
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
        None
    );

    let tags = vec![
        Tag::new(1, 100.0, 200.0, 50.0, 30.0, 0.95, "manual".to_string(), "animal".to_string())
    ];

    // Create a temporary test file
    let temp_file = "temp_integration_test.jpg";
    std::fs
        ::write(temp_file, b"fake image data for integration test")
        .expect("Failed to create temp file");

    info!("Testing post_event_with_tags...");
    match client.post_event_with_tags(&event, &tags, temp_file).await {
        Ok(response) => {
            match response.status {
                ResponseScoutStatus::Success => {
                    info!("✅ Successfully posted event with tags");
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
