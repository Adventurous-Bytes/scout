//! Storage module for uploading artifacts to Supabase storage using TUS protocol

use crate::models::ArtifactLocal;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;
use tus_client::http::{HttpHandler, HttpMethod, HttpRequest, HttpResponse};
use tus_client::{Client, Error as TusError};

/// Simplified HTTP handler for TUS client using modern reqwest
pub struct SimpleHttpHandler {
    client: reqwest::Client,
    auth_token: String,
}

impl SimpleHttpHandler {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client,
            auth_token: String::new(),
        }
    }

    pub fn with_auth(client: reqwest::Client, auth_token: String) -> Self {
        Self { client, auth_token }
    }
}

impl HttpHandler for SimpleHttpHandler {
    fn handle_request(&self, req: HttpRequest<'_>) -> Result<HttpResponse, TusError> {
        // Use a truly blocking HTTP client for synchronous operations
        let blocking_client = reqwest::blocking::Client::new();
        let mut request_builder = match req.method {
            HttpMethod::Post => blocking_client.post(&req.url),
            HttpMethod::Patch => blocking_client.patch(&req.url),
            HttpMethod::Head => blocking_client.head(&req.url),
            HttpMethod::Options => blocking_client.request(reqwest::Method::OPTIONS, &req.url),
            HttpMethod::Delete => blocking_client.delete(&req.url),
        };

        // Add auth header if available
        if !self.auth_token.is_empty() {
            request_builder =
                request_builder.header("Authorization", format!("Bearer {}", self.auth_token));
        }

        // Add x-upsert header to allow overwriting existing files (fixes 409 Conflict)
        request_builder = request_builder.header("x-upsert", "true");

        for (key, value) in &req.headers {
            request_builder = request_builder.header(key, value);
        }

        if let Some(body) = req.body {
            request_builder = request_builder.body(body.to_vec());
        }

        let response = request_builder
            .send()
            .map_err(|e| TusError::HttpHandlerError(e.to_string()))?;

        let status_code = response.status().as_u16() as usize;
        let mut headers = HashMap::new();

        for (key, value) in response.headers() {
            if let Ok(value_str) = value.to_str() {
                headers.insert(key.to_string(), value_str.to_string());
            }
        }

        Ok(HttpResponse {
            status_code,
            headers,
        })
    }
}

impl HttpHandler for &SimpleHttpHandler {
    fn handle_request(&self, req: HttpRequest<'_>) -> Result<HttpResponse, TusError> {
        (*self).handle_request(req)
    }
}

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub bucket_name: String,
}

pub struct StorageClient {
    config: StorageConfig,
    http_client: reqwest::Client,
    http_handler: Box<SimpleHttpHandler>,
}

impl Clone for SimpleHttpHandler {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            auth_token: self.auth_token.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub artifact_id_local: String,
    pub success: bool,
    pub storage_path: Option<String>,
    pub error: Option<String>,
}

impl StorageClient {
    pub fn new(config: StorageConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", config.supabase_anon_key))
                .map_err(|e| anyhow!("Invalid auth header: {}", e))?,
        );

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;

        let http_handler = Box::new(SimpleHttpHandler::with_auth(
            http_client.clone(),
            config.supabase_anon_key.clone(),
        ));

        Ok(Self {
            config,
            http_client,
            http_handler,
        })
    }

    /// Generate upload URLs for artifacts that need them
    pub async fn generate_upload_urls(
        &self,
        artifacts: &mut Vec<ArtifactLocal>,
        herd_id: i64,
    ) -> Result<()> {
        let now = Utc::now();

        for artifact in artifacts.iter_mut() {
            // Skip if we already have a recent URL
            if let Some(generated_at_str) = &artifact.upload_url_generated_at {
                if let Ok(generated_at) = DateTime::parse_from_rfc3339(generated_at_str) {
                    let age = now.signed_duration_since(generated_at.with_timezone(&Utc));
                    if age.num_hours() < 24 && artifact.upload_url.is_some() {
                        continue;
                    }
                }
            }

            let upload_url = self
                .generate_upload_url_for_artifact(artifact, herd_id)
                .await?;
            artifact.upload_url = Some(upload_url);
            artifact.upload_url_generated_at = Some(now.to_rfc3339());
        }

        Ok(())
    }

    /// Upload artifacts to storage using TUS protocol
    pub async fn upload_artifacts_to_storage(
        &self,
        artifacts: &mut Vec<ArtifactLocal>,
        herd_id: i64,
    ) -> Result<Vec<UploadResult>> {
        let mut results = Vec::new();

        for artifact in artifacts.iter_mut() {
            let artifact_id = artifact
                .id_local
                .as_deref()
                .unwrap_or("unknown")
                .to_string();

            // Skip if already uploaded
            if artifact.has_uploaded_file_to_storage {
                results.push(UploadResult {
                    artifact_id_local: artifact_id,
                    success: true,
                    storage_path: Some(artifact.file_path.clone()),
                    error: None,
                });
                continue;
            }

            // Check if upload URL is available
            let upload_url = match &artifact.upload_url {
                Some(url) => url.clone(),
                None => {
                    results.push(UploadResult {
                        artifact_id_local: artifact_id,
                        success: false,
                        storage_path: None,
                        error: Some("No upload URL available".to_string()),
                    });
                    continue;
                }
            };

            // Perform the upload
            match self
                .upload_single_artifact(artifact, &upload_url, herd_id)
                .await
            {
                Ok(storage_path) => {
                    artifact.has_uploaded_file_to_storage = true;
                    results.push(UploadResult {
                        artifact_id_local: artifact_id,
                        success: true,
                        storage_path: Some(storage_path),
                        error: None,
                    });
                }
                Err(e) => {
                    results.push(UploadResult {
                        artifact_id_local: artifact_id,
                        success: false,
                        storage_path: None,
                        error: Some(format!("Upload failed: {}", e)),
                    });
                }
            }
        }

        Ok(results)
    }

    /// Generate a TUS upload URL
    async fn generate_upload_url_for_artifact(
        &self,
        artifact: &ArtifactLocal,
        herd_id: i64,
    ) -> Result<String> {
        let file_name = Path::new(&artifact.file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Invalid file path: {}", artifact.file_path))?;

        let _object_path = format!("artifacts/{}/{}/{}", herd_id, artifact.device_id, file_name);

        // Extract project ID from supabase_url for TUS endpoint
        let url_parts = self
            .config
            .supabase_url
            .replace("https://", "")
            .replace(".supabase.co", "");
        let project_id = url_parts.split('.').next().unwrap_or("unknown");

        let tus_endpoint = format!(
            "https://{}.storage.supabase.co/storage/v1/upload/resumable",
            project_id
        );

        // Create TUS client and generate upload URL using spawn_blocking
        let http_handler = self.http_handler.clone();
        let file_path = artifact.file_path.clone();
        let endpoint = tus_endpoint.clone();
        let device_id = artifact.device_id;
        let file_name_owned = file_name.to_string();

        tokio::task::spawn_blocking(move || {
            let tus_client = Client::new(http_handler.as_ref());

            // Create metadata for Supabase
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("bucketName".to_string(), "artifacts".to_string());
            metadata.insert(
                "objectName".to_string(),
                format!("artifacts/{}/{}/{}", herd_id, device_id, file_name_owned),
            );
            metadata.insert("cacheControl".to_string(), "3600".to_string());
            metadata.insert("upsert".to_string(), "true".to_string());

            match tus_client.create_with_metadata(&endpoint, Path::new(&file_path), metadata) {
                Ok(upload_url) => {
                    tracing::debug!("Generated TUS upload URL: {}", upload_url);
                    Ok(upload_url)
                }
                Err(e) => {
                    tracing::error!("Failed to create TUS upload: {}", e);
                    Err(anyhow!("TUS upload creation failed: {}", e))
                }
            }
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
    }

    /// Upload a single artifact using TUS protocol for resumable uploads
    async fn upload_single_artifact(
        &self,
        artifact: &ArtifactLocal,
        upload_url: &str,
        herd_id: i64,
    ) -> Result<String> {
        // Verify file exists
        if !std::path::Path::new(&artifact.file_path).exists() {
            return Err(anyhow!("File does not exist: {}", artifact.file_path));
        }

        // Create TUS client for upload using spawn_blocking
        let http_handler = self.http_handler.clone();
        let file_path = artifact.file_path.clone();
        let upload_url = upload_url.to_string();
        let device_id = artifact.device_id;

        tokio::task::spawn_blocking(move || {
            let tus_client = Client::new(http_handler.as_ref());

            // Perform TUS upload with resumable capability
            match tus_client.upload(&upload_url, Path::new(&file_path)) {
                Ok(_) => {
                    let file_name = Path::new(&file_path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown");

                    let storage_path = format!("artifacts/{}/{}/{}", herd_id, device_id, file_name);

                    tracing::info!(
                        "Successfully uploaded {} via TUS to {}",
                        file_path,
                        storage_path
                    );

                    Ok(storage_path)
                }
                Err(e) => {
                    tracing::error!("TUS upload failed for {}: {}", file_path, e);
                    Err(anyhow!("TUS upload failed: {}", e))
                }
            }
        })
        .await
        .map_err(|e| anyhow!("Task join error: {}", e))?
    }

    /// Get artifacts that need upload URLs generated
    pub fn get_artifacts_needing_urls(&self, artifacts: &[ArtifactLocal]) -> Vec<ArtifactLocal> {
        let now = Utc::now();

        artifacts
            .iter()
            .filter(|artifact| {
                // Skip already uploaded artifacts
                if artifact.has_uploaded_file_to_storage {
                    return false;
                }

                // Check if URL is missing or expired
                if let Some(generated_at_str) = &artifact.upload_url_generated_at {
                    if let Ok(generated_at) = DateTime::parse_from_rfc3339(generated_at_str) {
                        let age = now.signed_duration_since(generated_at.with_timezone(&Utc));
                        return age.num_hours() >= 24 || artifact.upload_url.is_none();
                    }
                }

                // No upload URL generated yet
                true
            })
            .cloned()
            .collect()
    }
}

// to run just these tests do cargo test -- storage
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::v1::Syncable;
    use std::env;

    fn setup_storage_test_env() {
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
            bucket_name: "artifacts".to_string(),
        }
    }

    #[test]
    fn test_storage_client_creation() {
        setup_storage_test_env();

        let config = create_test_storage_config();
        let result = StorageClient::new(config);

        assert!(
            result.is_ok(),
            "Storage client creation should succeed with valid config"
        );
    }

    #[test]
    fn test_get_artifacts_needing_urls() {
        setup_storage_test_env();

        let config = create_test_storage_config();
        let client = StorageClient::new(config).expect("Failed to create storage client");

        let device_id: i64 = env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        let mut artifact = ArtifactLocal::new(
            "test-file.jpg".to_string(),
            None,
            device_id,
            Some("image".to_string()),
            None,
        );
        artifact.set_id_local("test-artifact".to_string());

        let artifacts = vec![artifact.clone()];
        let needing_urls = client.get_artifacts_needing_urls(&artifacts);
        assert_eq!(needing_urls.len(), 1);

        // Test with uploaded artifact
        artifact.has_uploaded_file_to_storage = true;
        let artifacts = vec![artifact];
        let needing_urls = client.get_artifacts_needing_urls(&artifacts);
        assert_eq!(needing_urls.len(), 0);
    }

    #[tokio::test]
    async fn test_generate_upload_urls_with_sample_file() {
        setup_storage_test_env();

        let device_id: i64 = env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        let herd_id: i64 = env::var("SCOUT_HERD_ID")
            .expect("SCOUT_HERD_ID required")
            .parse()
            .expect("SCOUT_HERD_ID must be valid integer");

        // Use the sample.mp4 file from test data
        let sample_file_path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/sample.mp4");

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
        artifact.set_id_local("test_sample_video".to_string());

        let mut artifacts = vec![artifact];

        println!("🔧 Testing upload URL generation with real Supabase config...");

        // Test URL generation
        let url_result = client.generate_upload_urls(&mut artifacts, herd_id).await;

        // Write results to file regardless of success or failure
        let output_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_output")
            .join("generated_urls.txt");

        // Create directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let mut url_info = format!(
            "Single File Upload URL Test\n\
             Test Run: {}\n\
             Herd ID: {}\n\
             Device ID: {}\n\
             File Path: {}\n\
             {}\n\n",
            chrono::Utc::now().to_rfc3339(),
            herd_id,
            device_id,
            sample_file_path.display(),
            "=".repeat(60)
        );

        match url_result {
            Ok(_) => {
                println!("✅ Successfully generated upload URL");
                assert!(artifacts[0].upload_url.is_some());
                assert!(artifacts[0].upload_url_generated_at.is_some());
                let generated_url = artifacts[0].upload_url.as_ref().unwrap();
                println!("   Generated URL: {}", generated_url);

                url_info.push_str(&format!(
                    "Status: SUCCESS\n\
                     Generated URL: {}\n\
                     Generated At: {}\n\
                     URL Length: {} characters\n\n",
                    generated_url,
                    artifacts[0].upload_url_generated_at.as_ref().unwrap(),
                    generated_url.len()
                ));

                // Test actual upload
                println!("🚀 Testing actual file upload...");
                let upload_result = client
                    .upload_artifacts_to_storage(&mut artifacts, herd_id)
                    .await;

                match upload_result {
                    Ok(results) => {
                        assert_eq!(results.len(), 1);
                        let result = &results[0];

                        if result.success {
                            println!("✅ File upload successful!");
                            println!("   Storage path: {:?}", result.storage_path);
                            url_info.push_str(&format!(
                                "Upload Status: SUCCESS\nStorage Path: {:?}\n",
                                result.storage_path
                            ));
                            assert!(artifacts[0].has_uploaded_file_to_storage);
                        } else {
                            println!("❌ Upload failed: {:?}", result.error);
                            url_info.push_str(&format!(
                                "Upload Status: FAILED\nError: {:?}\n",
                                result.error
                            ));
                        }
                    }
                    Err(e) => {
                        println!("⚠️  Upload error: {}", e);
                        url_info.push_str(&format!("Upload Status: ERROR\nError: {}\n", e));
                    }
                }
            }
            Err(e) => {
                println!("⚠️  URL generation error: {}", e);
                url_info.push_str(&format!("Status: FAILED\nError: {}\n", e));

                // Fail loudly if URL generation fails
                panic!("URL generation failed: {}", e);
            }
        }

        match std::fs::write(&output_path, &url_info) {
            Ok(_) => println!("📝 Test results written to: {}", output_path.display()),
            Err(e) => println!("⚠️  Failed to write results to file: {}", e),
        }
    }

    #[tokio::test]
    async fn test_generate_multiple_upload_urls() {
        setup_storage_test_env();

        let device_id: i64 = env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        let herd_id: i64 = env::var("SCOUT_HERD_ID")
            .expect("SCOUT_HERD_ID required")
            .parse()
            .expect("SCOUT_HERD_ID must be valid integer");

        let config = create_test_storage_config();
        let client = StorageClient::new(config).expect("Failed to create storage client");

        // Create multiple test artifacts with different file types
        let test_files = vec![
            ("test_video.mp4", "video"),
            ("test_audio.wav", "audio"),
            ("test_image.jpg", "image"),
            ("test_data.csv", "data"),
        ];

        let mut artifacts = Vec::new();
        for (i, (filename, modality)) in test_files.iter().enumerate() {
            let mut artifact = ArtifactLocal::new(
                format!("/tmp/{}", filename),
                None,
                device_id,
                Some(modality.to_string()),
                None,
            );
            artifact.set_id_local(format!("test_artifact_{}", i));
            artifacts.push(artifact);
        }

        println!("🔧 Testing multiple upload URL generation...");

        // Create test files that actually exist for better testing
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut actual_artifacts = Vec::new();

        // Use the sample.mp4 file if it exists, otherwise create temporary files
        let sample_file_path = manifest_dir.join("tests/data/sample.mp4");
        if sample_file_path.exists() {
            let mut artifact = ArtifactLocal::new(
                sample_file_path.to_string_lossy().to_string(),
                None,
                device_id,
                Some("video".to_string()),
                None,
            );
            artifact.set_id_local("test_sample_video".to_string());
            actual_artifacts.push(artifact);
        } else {
            // Create temporary files for testing
            use std::io::Write;
            let temp_dir = std::env::temp_dir();

            for (i, (filename, modality)) in test_files.iter().enumerate() {
                let temp_file_path = temp_dir.join(format!("scout_test_{}", filename));

                // Create a small temporary file
                if let Ok(mut file) = std::fs::File::create(&temp_file_path) {
                    let _ = file.write_all(b"test data");

                    let mut artifact = ArtifactLocal::new(
                        temp_file_path.to_string_lossy().to_string(),
                        None,
                        device_id,
                        Some(modality.to_string()),
                        None,
                    );
                    artifact.set_id_local(format!("test_artifact_{}", i));
                    actual_artifacts.push(artifact);
                }
            }
        }

        println!(
            "🔧 Testing multiple upload URL generation with {} artifacts...",
            actual_artifacts.len()
        );

        // Generate URLs
        let url_result = client
            .generate_upload_urls(&mut actual_artifacts, herd_id)
            .await;

        let output_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("test_output")
            .join("multiple_generated_urls.txt");

        // Create directory if it doesn't exist
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let mut output_content = format!(
            "Multiple URL Generation Test\n\
             Test Run: {}\n\
             Herd ID: {}\n\
             Device ID: {}\n\
             Test Files Count: {}\n\
             {}\n\n",
            chrono::Utc::now().to_rfc3339(),
            herd_id,
            device_id,
            actual_artifacts.len(),
            "=".repeat(80)
        );

        match url_result {
            Ok(_) => {
                println!(
                    "✅ Successfully processed {} upload URLs",
                    actual_artifacts.len()
                );

                for (i, artifact) in actual_artifacts.iter().enumerate() {
                    let success_status = if artifact.upload_url.is_some() {
                        "SUCCESS"
                    } else {
                        "FAILED"
                    };
                    let url_info = format!(
                        "Artifact #{}: {} ({})\n\
                         File: {}\n\
                         Modality: {:?}\n\
                         URL: {}\n\
                         Generated At: {}\n\
                         URL Length: {} characters\n\
                         {}\n\n",
                        i + 1,
                        artifact.id_local.as_ref().unwrap_or(&"unknown".to_string()),
                        success_status,
                        artifact.file_path,
                        artifact.modality,
                        artifact.upload_url.as_ref().unwrap_or(&"NONE".to_string()),
                        artifact
                            .upload_url_generated_at
                            .as_ref()
                            .unwrap_or(&"NONE".to_string()),
                        artifact.upload_url.as_ref().map(|u| u.len()).unwrap_or(0),
                        "-".repeat(60)
                    );
                    output_content.push_str(&url_info);
                }

                let successful_count = actual_artifacts
                    .iter()
                    .filter(|a| a.upload_url.is_some())
                    .count();
                output_content.push_str(&format!(
                    "\nSummary: {}/{} URLs generated successfully\n",
                    successful_count,
                    actual_artifacts.len()
                ));
            }
            Err(e) => {
                let error_info = format!("❌ URL generation failed: {}\nError Details: {}\n", e, e);
                output_content.push_str(&error_info);
                println!("⚠️  URL generation error: {}", e);

                // Write error to file first, then fail loudly
                let _ = std::fs::write(&output_path, &output_content);
                panic!("Multiple URL generation failed: {}", e);
            }
        }

        // Clean up temporary files
        for artifact in &actual_artifacts {
            if artifact.file_path.contains("scout_test_") {
                let _ = std::fs::remove_file(&artifact.file_path);
            }
        }

        match std::fs::write(&output_path, &output_content) {
            Ok(_) => println!(
                "📝 Multiple URLs test results written to: {}",
                output_path.display()
            ),
            Err(e) => println!("⚠️  Failed to write URLs to file: {}", e),
        }
    }
}
