use scout_rs::models::{ArtifactLocal, Syncable};
use scout_rs::storage::{SimpleHttpHandler, StorageClient, StorageConfig};
use scout_rs::tus::Client;
use std::env;
use std::path::Path;

fn setup_storage_test_env() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    let missing_vars = vec![
        (
            "SCOUT_DATABASE_REST_URL",
            env::var("SCOUT_DATABASE_REST_URL").is_err(),
        ),
        (
            "SUPABASE_PUBLIC_API_KEY",
            env::var("SUPABASE_PUBLIC_API_KEY").is_err(),
        ),
        ("SCOUT_DEVICE_ID", env::var("SCOUT_DEVICE_ID").is_err()),
        ("SCOUT_HERD_ID", env::var("SCOUT_HERD_ID").is_err()),
        (
            "SCOUT_DEVICE_API_KEY",
            env::var("SCOUT_DEVICE_API_KEY").is_err(),
        ),
    ];

    let missing: Vec<&str> = missing_vars
        .into_iter()
        .filter(|(_, is_missing)| *is_missing)
        .map(|(name, _)| name)
        .collect();

    if !missing.is_empty() {
        panic!(
            "❌ Missing required environment variables for storage tests: {}. Please check your .env file.",
            missing.join(", ")
        );
    }
}

fn create_test_storage_config() -> StorageConfig {
    let supabase_rest_url =
        env::var("SCOUT_DATABASE_REST_URL").expect("SCOUT_DATABASE_REST_URL must be set");

    let supabase_url = supabase_rest_url.replace("/rest/v1", "");

    StorageConfig {
        supabase_url,
        supabase_anon_key: env::var("SUPABASE_PUBLIC_API_KEY")
            .expect("SUPABASE_PUBLIC_API_KEY must be set"),
        scout_api_key: env::var("SCOUT_DEVICE_API_KEY")
            .expect("SCOUT_DEVICE_API_KEY must be set"),
        bucket_name: "artifacts".to_string(),
        allowed_extensions: vec![".mp4".to_string()],
    }
}

#[tokio::test]
async fn test_resumable_upload() {
    setup_storage_test_env();

    let device_id: i64 = env::var("SCOUT_DEVICE_ID")
        .expect("SCOUT_DEVICE_ID required")
        .parse()
        .expect("SCOUT_DEVICE_ID must be valid integer");

    let herd_id: i64 = env::var("SCOUT_HERD_ID")
        .expect("SCOUT_HERD_ID required")
        .parse()
        .expect("SCOUT_HERD_ID must be valid integer");

    // Use the sample1.mp4 file from test data
    let sample_file_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/sample1.mp4");

    if !sample_file_path.exists() {
        panic!("Sample file not found: {:?}", sample_file_path);
    }

    let config = create_test_storage_config();
    let client = StorageClient::new(config).expect("Failed to create storage client");

    let mut artifact = ArtifactLocal::new(
        sample_file_path.to_string_lossy().to_string(),
        None,
        device_id,
        Some("video".to_string()),
        None,
    );
    artifact.set_id_local("resumable_test_artifact".to_string());

    println!("🔧 Testing resumable upload functionality...");

    // Step 1: Generate upload URL
    let mut artifacts = vec![artifact.clone()];
    client
        .generate_upload_urls(&mut artifacts, herd_id)
        .await
        .expect("Failed to generate upload URLs");

    assert!(
        artifacts[0].upload_url.is_some(),
        "Upload URL should be generated"
    );

    let upload_url = artifacts[0].upload_url.as_ref().unwrap().clone();
    println!("✅ Upload URL generated: {}", upload_url);

    // Step 2: Start upload with small chunks (256KB) to allow quick interruption
    let chunk_size = 256 * 1024; // 256KB chunks
    let (upload_handle1, mut progress_rx1) = client
        .spawn_upload_artifact(artifacts[0].clone(), herd_id, Some(chunk_size), None);

    println!("🚀 Starting first upload with {}KB chunks...", chunk_size / 1024);

    // Step 3: Monitor progress and interrupt after some progress
    let mut last_progress_bytes = 0;
    let cancel_after_progress = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel_flag = cancel_after_progress.clone();

    let progress_monitor = tokio::spawn(async move {
        let mut captured_progress = Vec::new();
        let mut progress_count = 0;
        while let Ok(progress) = progress_rx1.recv().await {
            let percent = (progress.bytes_uploaded as f64 / progress.total_bytes as f64) * 100.0;
            let progress_msg = format!(
                "First upload: {:.1}% ({}/{} bytes)",
                percent, progress.bytes_uploaded, progress.total_bytes
            );
            println!("   {}", progress_msg);
            captured_progress.push((progress.bytes_uploaded, progress.total_bytes));
            progress_count += 1;
            
            // Cancel after 2-3 progress updates to ensure we interrupt mid-upload
            if progress_count >= 3 {
                cancel_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                break;
            }
        }
        captured_progress
    });

    // Wait for progress monitor to signal cancellation or timeout after 5 seconds
    let cancel_task = tokio::spawn(async move {
        loop {
            if cancel_after_progress.load(std::sync::atomic::Ordering::Relaxed) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });
    
    tokio::select! {
        _ = cancel_task => {
            upload_handle1.abort();
        }
        _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
            upload_handle1.abort();
        }
    }

    println!("🛑 First upload cancelled");
    println!("   Note: With cancellation support, the upload should stop when the task is aborted");

    // Get progress from first upload
    let first_upload_progress = progress_monitor
        .await
        .expect("Failed to get first upload progress");

    if !first_upload_progress.is_empty() {
        last_progress_bytes = first_upload_progress.last().unwrap().0;
        let progress_count = first_upload_progress.len();
        println!(
            "📊 First upload reached {} bytes in {} progress updates before cancellation",
            last_progress_bytes, progress_count
        );
    }

    // Wait longer for the cancellation to fully propagate and any in-flight requests to complete
    // The upload task might continue uploading chunks even after abort() is called
    println!("⏳ Waiting for upload cancellation to fully propagate...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Step 4: Verify upload URL is still valid using TUS client directly
    println!("🔍 Verifying upload URL is still valid...");

    // Create HTTP handler in a blocking context to avoid runtime issues
    let auth_token = env::var("SUPABASE_PUBLIC_API_KEY")
        .expect("SUPABASE_PUBLIC_API_KEY must be set");
    let scout_api_key = env::var("SCOUT_DEVICE_API_KEY")
        .expect("SCOUT_DEVICE_API_KEY must be set");
    let upload_url_for_info = upload_url.clone();

    let upload_info = tokio::task::spawn_blocking(move || {
        let http_client = reqwest::Client::new();
        let http_handler = SimpleHttpHandler::with_auth_and_api_key(
            http_client,
            auth_token,
            scout_api_key,
        );
        let tus_client = Client::new(http_handler);
        
        // Get info about the upload to verify it's still valid and get the current offset
        tus_client.get_info(&upload_url_for_info)
    })
    .await
    .expect("Failed to spawn get_info task")
    .expect("Failed to get upload info - upload URL may have expired or been deleted");

    println!(
        "✅ Upload URL is valid. Current offset: {} bytes, Total size: {:?}",
        upload_info.bytes_uploaded,
        upload_info.total_size
    );

    // Key assertion: The upload should have made progress before cancellation
    assert!(
        upload_info.bytes_uploaded > 0,
        "Upload should have made progress before cancellation. Got {} bytes uploaded",
        upload_info.bytes_uploaded
    );

    // Verify the offset matches what we expect (should be close to last_progress_bytes)
    // Note: The offset might be slightly different due to chunk boundaries
    println!(
        "📊 Server reports {} bytes uploaded (we saw {} bytes in progress updates)",
        upload_info.bytes_uploaded,
        last_progress_bytes
    );

    // Step 5: Resume the upload using the same URL
    let expected_offset = upload_info.bytes_uploaded;
    println!("🔄 Resuming upload from offset {}...", expected_offset);

    // Double-check the upload state right before resuming
    let upload_url_for_double_check = upload_url.clone();
    let expected_offset_for_check = expected_offset;
    let double_check_info = tokio::task::spawn_blocking(move || {
        let auth_token = env::var("SUPABASE_PUBLIC_API_KEY").expect("SUPABASE_PUBLIC_API_KEY must be set");
        let scout_api_key = env::var("SCOUT_DEVICE_API_KEY").expect("SCOUT_DEVICE_API_KEY must be set");
        let http_client = reqwest::Client::new();
        let http_handler = SimpleHttpHandler::with_auth_and_api_key(http_client, auth_token, scout_api_key);
        let tus_client = Client::new(http_handler);
        tus_client.get_info(&upload_url_for_double_check)
    })
    .await
    .expect("Failed to spawn double-check task");

    match double_check_info {
        Ok(info) => {
            println!("   Double-check: Upload URL still valid, offset: {} bytes", info.bytes_uploaded);
            if info.bytes_uploaded != expected_offset_for_check {
                println!("   ⚠️  Offset changed from {} to {} - upload may have continued after cancellation", 
                    expected_offset_for_check, info.bytes_uploaded);
            }
        }
        Err(e) => {
            println!("   ⚠️  Double-check failed: {}. Upload URL may have expired.", e);
            // Continue anyway to see what happens
        }
    }

    let file_path = sample_file_path.clone();
    let upload_url_clone = upload_url.clone();
    let http_handler_clone = SimpleHttpHandler::with_auth_and_api_key(
        reqwest::Client::new(),
        env::var("SUPABASE_PUBLIC_API_KEY").expect("SUPABASE_PUBLIC_API_KEY must be set"),
        env::var("SCOUT_DEVICE_API_KEY").expect("SCOUT_DEVICE_API_KEY must be set"),
    );
    // Use Arc<Mutex> to share state between the callback and the main thread
    let first_resumed_bytes = std::sync::Arc::new(std::sync::Mutex::new(None::<usize>));
    let first_resumed_bytes_clone = first_resumed_bytes.clone();
    
    let resumed_upload_result = tokio::task::spawn_blocking(move || {
        let tus_client = Client::new(&http_handler_clone);

        // Create progress callback to track resumed upload
        let first_bytes_shared = first_resumed_bytes_clone.clone();
        let progress_callback = move |bytes_uploaded: usize, total_bytes: usize| {
            // Capture the first progress update
            let mut first = first_bytes_shared.lock().unwrap();
            if first.is_none() {
                *first = Some(bytes_uploaded);
                println!(
                    "   Resumed upload first progress: {} bytes (expected offset: {} bytes)",
                    bytes_uploaded, expected_offset
                );
            }
            
            let percent = (bytes_uploaded as f64 / total_bytes as f64) * 100.0;
            if bytes_uploaded % (chunk_size * 4) == 0 || bytes_uploaded == total_bytes {
                // Print progress every 4 chunks or at completion
                println!(
                    "   Resumed upload: {:.1}% ({}/{} bytes)",
                    percent, bytes_uploaded, total_bytes
                );
            }
        };

        tus_client.upload_with_chunk_size(
            &upload_url_clone,
            Path::new(&file_path),
            chunk_size,
            Some(&progress_callback),
        )
    })
    .await
    .expect("Failed to spawn resume upload task");

    let first_resumed_bytes_value = first_resumed_bytes.lock().unwrap().take();

    // Step 6: Verify the upload completed successfully
    match resumed_upload_result {
        Ok(_) => {
            println!("✅ Resumed upload completed successfully!");

            // Verify the upload actually completed by checking the final state
            // Create a new TUS client for verification in a blocking context
            let upload_url_for_verify = upload_url.clone();
            let final_info = tokio::task::spawn_blocking(move || {
                let verify_http_handler = SimpleHttpHandler::with_auth_and_api_key(
                    reqwest::Client::new(),
                    env::var("SUPABASE_PUBLIC_API_KEY").expect("SUPABASE_PUBLIC_API_KEY must be set"),
                    env::var("SCOUT_DEVICE_API_KEY").expect("SCOUT_DEVICE_API_KEY must be set"),
                );
                let verify_tus_client = Client::new(verify_http_handler);
                verify_tus_client.get_info(&upload_url_for_verify)
            })
            .await
            .expect("Failed to spawn verify task")
            .expect("Failed to get final upload info");

            let file_size = std::fs::metadata(&sample_file_path)
                .map(|m| m.len() as usize)
                .expect("Failed to get file size");

            assert_eq!(
                final_info.bytes_uploaded, file_size,
                "Upload should be complete. Expected {} bytes, got {} bytes",
                file_size, final_info.bytes_uploaded
            );

            println!(
                "✅ Upload verification: {} bytes uploaded (file size: {} bytes)",
                final_info.bytes_uploaded, file_size
            );

            // Verify that the upload resumed from the correct offset (not from 0)
            if let Some(first_bytes) = first_resumed_bytes_value {
                assert!(
                    first_bytes >= expected_offset,
                    "Upload should resume from offset {} or later, but started at {} bytes",
                    expected_offset, first_bytes
                );
                assert!(
                    first_bytes > 0,
                    "Upload should not restart from 0. Got {} bytes as first progress",
                    first_bytes
                );
                println!(
                    "   ✅ Verified: Upload resumed from {} bytes (expected: {} bytes, not 0)",
                    first_bytes, expected_offset
                );
            }

            println!("🎉 Resumable upload test completed successfully!");
            println!(
                "   ✅ Upload was interrupted at ~{} bytes",
                upload_info.bytes_uploaded
            );
            println!(
                "   ✅ Upload resumed from {} bytes (not from 0)",
                upload_info.bytes_uploaded
            );
            println!("   ✅ Upload completed successfully");
        }
        Err(e) => {
            // Try to get info again to see if the URL is still valid
            let upload_url_for_error = upload_url.clone();
            let error_info = tokio::task::spawn_blocking(move || {
                let verify_http_handler = SimpleHttpHandler::with_auth_and_api_key(
                    reqwest::Client::new(),
                    env::var("SUPABASE_PUBLIC_API_KEY").expect("SUPABASE_PUBLIC_API_KEY must be set"),
                    env::var("SCOUT_DEVICE_API_KEY").expect("SCOUT_DEVICE_API_KEY must be set"),
                );
                let verify_tus_client = Client::new(verify_http_handler);
                verify_tus_client.get_info(&upload_url_for_error)
            })
            .await
            .expect("Failed to spawn error info task");
            
            match error_info {
                Ok(info) => {
                    panic!(
                        "❌ Resumed upload failed: {}. Upload URL is still valid (offset: {} bytes), but resume failed. This indicates a resumability bug.",
                        e, info.bytes_uploaded
                    );
                }
                Err(info_err) => {
                    panic!(
                        "❌ Resumed upload failed: {}. Upload URL is also invalid now: {}. The upload URL may have expired or been cleaned up.",
                        e, info_err
                    );
                }
            }
        }
    }
}
