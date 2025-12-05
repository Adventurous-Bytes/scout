//! Storage module for uploading artifacts to Supabase storage using TUS protocol

use crate::models::ArtifactLocal;
use crate::tus::http::{HttpHandler, HttpMethod, HttpRequest, HttpResponse};
use crate::tus::{Client, Error as TusError};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::broadcast;

/// Progress information for upload operations
#[derive(Debug, Clone)]
pub struct UploadProgress {
    pub bytes_uploaded: usize,
    pub total_bytes: usize,
    pub file_name: String,
}

/// Simplified HTTP handler for TUS client using modern reqwest
pub struct SimpleHttpHandler {
    client: reqwest::Client,
    auth_token: String,
    scout_api_key: String,
}

impl SimpleHttpHandler {
    pub fn new(client: reqwest::Client) -> Self {
        Self {
            client,
            auth_token: String::new(),
            scout_api_key: String::new(),
        }
    }

    pub fn with_auth(client: reqwest::Client, auth_token: String) -> Self {
        Self {
            client,
            auth_token,
            scout_api_key: String::new(),
        }
    }

    pub fn with_auth_and_api_key(
        client: reqwest::Client,
        auth_token: String,
        scout_api_key: String,
    ) -> Self {
        Self {
            client,
            auth_token,
            scout_api_key,
        }
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

        // Add apikey header (similar to db_client.rs)
        if !self.auth_token.is_empty() {
            request_builder = request_builder.header("apikey", &self.auth_token);
        }

        // Add device API key header (similar to db_client.rs)
        if !self.scout_api_key.is_empty() {
            request_builder = request_builder.header("api_key", &self.scout_api_key);
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
    pub scout_api_key: String,
    pub bucket_name: String,
    pub allowed_extensions: Vec<String>,
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
            scout_api_key: self.scout_api_key.clone(),
        }
    }
}

impl StorageClient {
    pub fn new(config: StorageConfig) -> Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", config.supabase_anon_key))
                .map_err(|e| anyhow!("Invalid auth header: {}", e))?,
        );

        // Add apikey header (similar to db_client.rs)
        headers.insert(
            "apikey",
            reqwest::header::HeaderValue::from_str(&config.supabase_anon_key)
                .map_err(|e| anyhow!("Invalid apikey header: {}", e))?,
        );

        // Add device API key header (similar to db_client.rs)
        headers.insert(
            "api_key",
            reqwest::header::HeaderValue::from_str(&config.scout_api_key)
                .map_err(|e| anyhow!("Invalid api_key header: {}", e))?,
        );

        let http_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;

        let http_handler = Box::new(SimpleHttpHandler::with_auth_and_api_key(
            http_client.clone(),
            config.supabase_anon_key.clone(),
            config.scout_api_key.clone(),
        ));

        Ok(Self {
            config,
            http_client,
            http_handler,
        })
    }

    pub fn with_allowed_extensions(
        supabase_url: String,
        supabase_anon_key: String,
        scout_api_key: String,
        bucket_name: String,
        allowed_extensions: Vec<String>,
    ) -> Result<Self> {
        let config = StorageConfig {
            supabase_url,
            supabase_anon_key,
            scout_api_key,
            bucket_name,
            allowed_extensions,
        };
        Self::new(config)
    }

    pub fn with_device_api_key(
        supabase_url: String,
        supabase_anon_key: String,
        scout_api_key: String,
        bucket_name: String,
        allowed_extensions: Vec<String>,
    ) -> Result<Self> {
        let config = StorageConfig {
            supabase_url,
            supabase_anon_key,
            scout_api_key,
            bucket_name,
            allowed_extensions,
        };
        Self::new(config)
    }

    /// Generate upload URLs for artifacts that need them
    ///
    /// This method filters artifacts based on allowed file extensions and
    /// generates TUS upload URLs only for artifacts that:
    /// - Have allowed file extensions (e.g., .mp4)
    /// - Don't already have recent upload URLs (within 24 hours)
    /// - Haven't already been uploaded
    ///
    /// # Arguments
    /// * `artifacts` - Vector of artifacts to process (modified in place)
    /// * `herd_id` - The herd ID for the storage path
    ///
    /// # Returns
    /// Result<()> - Success or error from URL generation process
    pub async fn generate_upload_urls(
        &self,
        artifacts: &mut Vec<ArtifactLocal>,
        herd_id: i64,
    ) -> Result<()> {
        let now = Utc::now();

        for artifact in artifacts.iter_mut() {
            // Check file extension filter
            if let Some(extension) = Path::new(&artifact.file_path).extension() {
                if let Some(ext_str) = extension.to_str() {
                    if !self
                        .config
                        .allowed_extensions
                        .contains(&format!(".{}", ext_str))
                    {
                        tracing::warn!(
                            "Skipping artifact {} - extension .{} not in allowed list: {:?}",
                            artifact.file_path,
                            ext_str,
                            self.config.allowed_extensions
                        );
                        continue;
                    }
                }
            } else {
                tracing::warn!(
                    "Skipping artifact {} - no file extension found",
                    artifact.file_path
                );
                continue;
            }

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

    /// Upload a single artifact to storage using TUS protocol in a spawned task
    ///
    /// This method spawns a non-blocking background task to upload the artifact,
    /// allowing the caller to continue processing while the upload happens.
    ///
    /// # Arguments
    /// * `artifact` - The artifact to upload (must have upload_url set)
    /// * `herd_id` - The herd ID for the storage path
    /// * `chunk_size` - Size of upload chunks in bytes (default: 1MB for better progress granularity)
    ///
    /// # Returns
    /// A tuple of (JoinHandle, progress_receiver) where:
    /// - JoinHandle: Resolves to Result<(ArtifactLocal, String)>
    /// - progress_receiver: Broadcast receiver for upload progress updates
    ///
    /// # Example - Complete Artifact Management Workflow
    /// ```rust,no_run
    /// # use scout_rs::sync::SyncEngine;
    /// # use scout_rs::models::ArtifactLocal;
    /// # use scout_rs::storage::StorageConfig;
    /// # use scout_rs::client::ScoutClient;
    /// # use scout_rs::db_client::DatabaseConfig;
    /// # async fn example() -> anyhow::Result<()> {
    /// // 1. Create scout client and storage config
    /// let db_config = DatabaseConfig::from_env()?;
    /// let scout_client = ScoutClient::new(db_config);
    /// let storage_config = StorageConfig {
    ///     supabase_url: "https://your-project.supabase.co".to_string(),
    ///     supabase_anon_key: "your-anon-key".to_string(),
    ///     scout_api_key: "your-device-api-key".to_string(),
    ///     bucket_name: "artifacts".to_string(),
    ///     allowed_extensions: vec![".mp4".to_string()],
    /// };
    /// let mut sync_engine = SyncEngine::new(scout_client, "db.path".to_string(), None, None, false)?
    ///     .with_storage(storage_config)?;
    ///
    /// // 2. Query artifacts by various criteria
    /// let all_artifacts = sync_engine.get_all_artifacts()?;
    /// let pending_upload = sync_engine.get_artifacts_pending_upload()?;
    /// let ready_for_upload = sync_engine.get_artifacts_ready_for_upload()?;
    /// let need_urls = sync_engine.get_artifacts_needing_upload_urls()?;
    /// let uploaded_artifacts = sync_engine.get_artifacts_by_upload_status(true)?;
    /// let specific_artifact = sync_engine.get_artifact_by_local_id("artifact_123")?;
    ///
    /// // 3. Generate upload URLs for artifacts that need them
    /// let mut artifacts_needing_urls = sync_engine.get_artifacts_needing_upload_urls()?;
    /// sync_engine.generate_upload_urls(&mut artifacts_needing_urls).await?;
    ///
    /// // 4. Upload artifacts with progress monitoring and custom chunk size
    /// let ready_artifacts = sync_engine.get_artifacts_ready_for_upload()?;
    /// for artifact in ready_artifacts {
    ///     let (upload_handle, mut progress_rx) = sync_engine
    ///         .spawn_upload_artifact(artifact.clone(), Some(512 * 1024))?; // 512KB chunks
    ///
    ///     // Monitor progress in background
    ///     tokio::spawn(async move {
    ///         while let Ok(progress) = progress_rx.recv().await {
    ///             let percent = (progress.bytes_uploaded as f64 / progress.total_bytes as f64) * 100.0;
    ///             println!("Progress: {:.1}% ({}/{} bytes) for {}",
    ///                      percent, progress.bytes_uploaded, progress.total_bytes, progress.file_name);
    ///         }
    ///     });
    ///
    ///     // Handle upload completion or cancellation
    ///     match upload_handle.await {
    ///         Ok(Ok((updated_artifact, storage_path))) => {
    ///             println!("✅ Uploaded {} to {}", updated_artifact.file_path, storage_path);
    ///             sync_engine.upsert_items(vec![updated_artifact])?; // Update database
    ///         }
    ///         Ok(Err(e)) => println!("❌ Upload failed: {}", e),
    ///         Err(_) => println!("🛑 Upload cancelled"), // Task was aborted
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn spawn_upload_artifact(
        &self,
        mut artifact: ArtifactLocal,
        herd_id: i64,
        chunk_size: Option<usize>,
    ) -> (
        tokio::task::JoinHandle<Result<(ArtifactLocal, String)>>,
        broadcast::Receiver<UploadProgress>,
    ) {
        let storage_client_handler = self.http_handler.clone();
        let chunk_size = chunk_size.unwrap_or(1024 * 1024); // Default 1MB for better progress granularity

        // Create broadcast channel for progress updates
        let (progress_tx, progress_rx) = broadcast::channel(1000);

        let upload_handle = tokio::spawn(async move {
            // Check if already uploaded
            if artifact.has_uploaded_file_to_storage {
                let storage_path = format!(
                    "{}/{}/{}",
                    herd_id,
                    artifact.device_id,
                    Path::new(&artifact.file_path)
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("unknown")
                );
                return Ok((artifact, storage_path));
            }

            // Check if upload URL is available
            let upload_url = match &artifact.upload_url {
                Some(url) => url.clone(),
                None => return Err(anyhow!("No upload URL available")),
            };

            // Verify file exists
            if !std::path::Path::new(&artifact.file_path).exists() {
                return Err(anyhow!("File does not exist: {}", artifact.file_path));
            }

            // Perform TUS upload using spawn_blocking
            let file_path = artifact.file_path.clone();
            let device_id = artifact.device_id;
            let file_name = Path::new(&file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            // Get file size for progress tracking
            let file_size = std::fs::metadata(&file_path)
                .map(|m| m.len() as usize)
                .unwrap_or(0);

            let storage_path = tokio::task::spawn_blocking(move || {
                let tus_client = Client::new(storage_client_handler.as_ref());

                // Create progress callback
                let progress_callback = move |bytes_uploaded: usize, total_bytes: usize| {
                    let progress = UploadProgress {
                        bytes_uploaded,
                        total_bytes: if total_bytes > 0 {
                            total_bytes
                        } else {
                            file_size
                        },
                        file_name: file_name.clone(),
                    };
                    let _ = progress_tx.send(progress); // Ignore send errors if no receivers
                };

                // Perform TUS upload with resumable capability and progress tracking
                match tus_client.upload_with_chunk_size(
                    &upload_url,
                    Path::new(&file_path),
                    chunk_size,
                    Some(&progress_callback),
                ) {
                    Ok(_) => {
                        let file_name = Path::new(&file_path)
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown");

                        let storage_path = format!("{}/{}/{}", herd_id, device_id, file_name);

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
            .map_err(|e| anyhow!("Task join error: {}", e))??;

            // Mark as uploaded
            artifact.has_uploaded_file_to_storage = true;
            Ok((artifact, storage_path))
        });

        (upload_handle, progress_rx)
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
                format!("{}/{}/{}", herd_id, device_id, file_name_owned),
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
            scout_api_key: env::var("SCOUT_DEVICE_API_KEY")
                .expect("SCOUT_DEVICE_API_KEY must be set"),
            bucket_name: "artifacts".to_string(),
            allowed_extensions: vec![".mp4".to_string()],
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
        artifact.set_id_local("test_sample1_video".to_string());

        let mut artifacts = vec![artifact];

        println!("🔧 Testing upload URL generation with real Supabase config...");

        // Test URL generation
        let url_result = client.generate_upload_urls(&mut artifacts, herd_id).await;

        // Write results to file regardless of success or failure
        let output_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/output/storage")
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

                // Test actual upload using spawn with progress tracking
                println!("🚀 Testing actual file upload with progress...");
                let (upload_handle, mut progress_rx) =
                    client.spawn_upload_artifact(artifacts[0].clone(), herd_id, None);

                // Spawn task to listen for progress updates

                let progress_task = tokio::spawn(async move {
                    let mut last_progress = 0;
                    let mut progress_updates = Vec::new();
                    while let Ok(progress) = progress_rx.recv().await {
                        if progress.bytes_uploaded > last_progress {
                            let percent = (progress.bytes_uploaded as f64
                                / progress.total_bytes as f64)
                                * 100.0;
                            let progress_msg = format!(
                                "   Progress: {:.1}% ({}/{} bytes) for {}",
                                percent,
                                progress.bytes_uploaded,
                                progress.total_bytes,
                                progress.file_name
                            );
                            println!("{}", progress_msg);
                            progress_updates.push(progress_msg);
                            last_progress = progress.bytes_uploaded;
                        }
                    }
                    progress_updates
                });

                match upload_handle.await {
                    Ok(Ok((updated_artifact, storage_path))) => {
                        println!("✅ File upload successful!");
                        println!("   Storage path: {}", storage_path);
                        url_info.push_str(&format!(
                            "Upload Status: SUCCESS\nStorage Path: {}\n",
                            storage_path
                        ));
                        assert!(updated_artifact.has_uploaded_file_to_storage);
                        artifacts[0] = updated_artifact;

                        // Get progress updates and cancel task
                        if let Ok(progress_updates) = progress_task.await {
                            url_info.push_str("Progress Updates:\n");
                            for update in progress_updates {
                                url_info.push_str(&format!("{}\n", update));
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ Upload failed: {}", e);
                        url_info.push_str(&format!("Upload Status: FAILED\nError: {}\n", e));
                        progress_task.abort();
                    }
                    Err(e) => {
                        println!("⚠️  Task join error: {}", e);
                        url_info.push_str(&format!("Upload Status: TASK_ERROR\nError: {}\n", e));
                        progress_task.abort();
                    }
                }
            }
            Err(e) => {
                panic!("URL generation failed: {}", e);
            }
        }

        match std::fs::write(&output_path, &url_info) {
            Ok(_) => println!("📝 Test results written to: {}", output_path.display()),
            Err(e) => println!("⚠️  Failed to write results to file: {}", e),
        }
    }

    #[tokio::test]
    async fn test_multiple_files_upload() {
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

        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let sample_files = vec![
            ("tests/data/sample1.mp4", "test_sample1_video"),
            ("tests/data/sample2.mp4", "test_sample2_video"),
        ];

        let mut artifacts = Vec::new();
        for (file_path, id_local) in sample_files {
            let sample_file_path = manifest_dir.join(file_path);
            if !sample_file_path.exists() {
                panic!("Sample file not found: {:?}", sample_file_path);
            }

            let mut artifact = ArtifactLocal::new(
                sample_file_path.to_string_lossy().to_string(),
                None,
                device_id,
                Some("video".to_string()),
                None,
            );
            artifact.set_id_local(id_local.to_string());
            artifacts.push(artifact);
        }

        // Generate URLs
        client
            .generate_upload_urls(&mut artifacts, herd_id)
            .await
            .expect("Failed to generate upload URLs");

        // Test spawned uploads with progress tracking
        let mut upload_handles = Vec::new();
        let mut progress_receivers = Vec::new();

        for artifact in artifacts {
            let (handle, progress_rx) = client.spawn_upload_artifact(artifact, herd_id, None);
            upload_handles.push(handle);
            progress_receivers.push(progress_rx);
        }

        // Spawn tasks to monitor progress
        let mut progress_tasks = Vec::new();
        for (i, mut progress_rx) in progress_receivers.into_iter().enumerate() {
            let progress_task = tokio::spawn(async move {
                let mut progress_log: Vec<String> = Vec::new();
                while let Ok(progress) = progress_rx.recv().await {
                    let percent =
                        (progress.bytes_uploaded as f64 / progress.total_bytes as f64) * 100.0;
                    let progress_msg = format!(
                        "File {}: {:.1}% ({}/{} bytes) for {}",
                        i + 1,
                        percent,
                        progress.bytes_uploaded,
                        progress.total_bytes,
                        progress.file_name
                    );
                    println!("{}", progress_msg);
                    progress_log.push(progress_msg);
                }
                progress_log
            });
            progress_tasks.push(progress_task);
        }

        // Wait for all uploads
        let mut results = Vec::new();
        for handle in upload_handles {
            let result = handle
                .await
                .expect("Task join failed")
                .expect("Upload failed");
            results.push(result);
        }

        // Collect progress logs from monitoring tasks
        let mut all_progress_logs = Vec::new();
        for task in progress_tasks {
            if let Ok(progress_log) = task.await {
                all_progress_logs.extend(progress_log);
            }
        }

        // Write results
        let output_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/output/storage")
            .join("multiple_uploads.txt");

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let mut output_content = format!(
            "Multiple File Upload Test\n\
             Test Run: {}\n\
             Files Uploaded: {}\n\
             {}\n\n",
            chrono::Utc::now().to_rfc3339(),
            results.len(),
            "=".repeat(60)
        );

        // Add results
        for (i, (artifact, path)) in results.iter().enumerate() {
            output_content.push_str(&format!(
                "File {}: {} -> {}\n",
                i + 1,
                Path::new(&artifact.file_path)
                    .file_name()
                    .unwrap()
                    .to_string_lossy(),
                path
            ));
        }

        // Add progress logs
        if !all_progress_logs.is_empty() {
            output_content.push_str("\nProgress Updates:\n");
            for progress_log in all_progress_logs {
                output_content.push_str(&format!("{}\n", progress_log));
            }
        }

        std::fs::write(&output_path, &output_content).expect("Failed to write results");
        println!("📝 Upload results written to: {}", output_path.display());
    }

    #[tokio::test]
    async fn test_file_extension_filtering() {
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

        // Create temporary files with different extensions
        use std::io::Write;
        let temp_dir = std::env::temp_dir();

        let temp_files = vec![
            (temp_dir.join("scout_test_video.mp4"), "video"),
            (temp_dir.join("scout_test_image.jpg"), "image"),
            (temp_dir.join("scout_test_audio.wav"), "audio"),
        ];

        // Create the temporary files
        for (path, _) in &temp_files {
            if let Ok(mut file) = std::fs::File::create(path) {
                let _ = file.write_all(b"test data");
            }
        }

        // Create artifacts with different file extensions
        let mut artifacts = vec![
            ArtifactLocal::new(
                temp_files[0].0.to_string_lossy().to_string(),
                None,
                device_id,
                Some("video".to_string()),
                None,
            ),
            ArtifactLocal::new(
                temp_files[1].0.to_string_lossy().to_string(), // Not allowed
                None,
                device_id,
                Some("image".to_string()),
                None,
            ),
            ArtifactLocal::new(
                temp_files[2].0.to_string_lossy().to_string(), // Not allowed
                None,
                device_id,
                Some("audio".to_string()),
                None,
            ),
        ];

        artifacts[0].set_id_local("test_mp4".to_string());
        artifacts[1].set_id_local("test_jpg".to_string());
        artifacts[2].set_id_local("test_wav".to_string());

        println!("🔧 Testing file extension filtering...");

        // Generate URLs - should only process .mp4 file
        let url_result = client.generate_upload_urls(&mut artifacts, herd_id).await;

        match url_result {
            Ok(_) => {
                println!("✅ Extension filtering completed");
                // Only the .mp4 file should have a URL
                assert!(artifacts[0].upload_url.is_some()); // Should have URL because .mp4 is allowed
                assert!(artifacts[1].upload_url.is_none()); // No URL because .jpg not allowed
                assert!(artifacts[2].upload_url.is_none()); // No URL because .wav not allowed

                println!("   MP4 file: {:?}", artifacts[0].upload_url.is_some());
                println!("   JPG file: {:?}", artifacts[1].upload_url.is_some());
                println!("   WAV file: {:?}", artifacts[2].upload_url.is_some());
            }
            Err(e) => {
                panic!("Extension filtering test failed: {}", e);
            }
        }

        // Write test results to file
        let output_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/output/storage")
            .join("extension_filtering_test.txt");

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let test_results = format!(
            "File Extension Filtering Test\n\
             Test Run: {}\n\
             Allowed Extensions: {:?}\n\
             {}\n\n\
             Results:\n\
             - test_video.mp4: URL={}\n\
             - test_image.jpg: URL={}\n\
             - test_audio.wav: URL={}\n\n\
             Expected: Only .mp4 files should be processed\n",
            chrono::Utc::now().to_rfc3339(),
            client.config.allowed_extensions,
            "=".repeat(60),
            if artifacts[0].upload_url.is_some() {
                "Generated"
            } else {
                "None"
            },
            if artifacts[1].upload_url.is_some() {
                "Generated"
            } else {
                "None"
            },
            if artifacts[2].upload_url.is_some() {
                "Generated"
            } else {
                "None"
            }
        );

        // Clean up temporary files
        for (path, _) in &temp_files {
            let _ = std::fs::remove_file(path);
        }

        match std::fs::write(&output_path, &test_results) {
            Ok(_) => println!(
                "📝 Extension filtering test results written to: {}",
                output_path.display()
            ),
            Err(e) => println!("⚠️  Failed to write test results: {}", e),
        }
    }
}
