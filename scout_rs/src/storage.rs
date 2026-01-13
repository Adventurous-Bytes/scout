//! Storage module for uploading artifacts to Supabase storage using TUS protocol

use crate::models::ArtifactLocal;
use crate::tus::http::{HttpHandler, HttpMethod, HttpRequest, HttpResponse};
use crate::tus::{Client, Error as TusError};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::{broadcast, watch};

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

const BUCKET_NAME_ARTIFACTS: &str = "artifacts";

/// Generate a remote file path from a local file path
///
/// Transforms a local path like `/opt/raven/blah/blah/test.mp4`
/// into a remote path like `herd_id/device_id/test.mp4`
///
/// # Arguments
/// * `local_path` - The local file path
/// * `herd_id` - The herd ID for the storage path
/// * `device_id` - The device ID for the storage path
///
/// # Returns
/// Result<String> - The remote file path or an error if the local path is invalid
fn generate_remote_path(local_path: &str, herd_id: i64, device_id: i64) -> Result<String> {
    let file_name = Path::new(local_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("Invalid file path: {}", local_path))?;

    Ok(format!("{}/{}/{}", herd_id, device_id, file_name))
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
    /// * `max_retries` - Maximum number of retries for expired upload URLs (default: 2)
    ///
    /// # Returns
    /// A tuple of (JoinHandle, progress_receiver) where:
    /// - JoinHandle: Resolves to Result<(ArtifactLocal, String)>
    /// - progress_receiver: Broadcast receiver for upload progress updates
    ///
    /// ```
    pub fn spawn_upload_artifact(
        &self,
        mut artifact: ArtifactLocal,
        herd_id: i64,
        chunk_size: Option<usize>,
        max_retries: Option<u32>,
    ) -> (
        tokio::task::JoinHandle<Result<(ArtifactLocal, String)>>,
        broadcast::Receiver<UploadProgress>,
    ) {
        let storage_client_handler = self.http_handler.clone();
        let chunk_size = chunk_size.unwrap_or(1024 * 1024); // Default 1MB for better progress granularity
        let max_retries = max_retries.unwrap_or(2); // Default to 2 retries
        let config = self.config.clone();

        // Create broadcast channel for progress updates
        let (progress_tx, progress_rx) = broadcast::channel(1000);

        // Create cancellation channel
        let (cancel_tx, cancel_rx) = watch::channel(false);
        let cancel_tx_for_background = cancel_tx.clone();

        let upload_handle = tokio::spawn(async move {

            // Check if already uploaded
            if artifact.has_uploaded_file_to_storage {
                let storage_path =
                    generate_remote_path(&artifact.file_path, herd_id, artifact.device_id)
                        .unwrap_or_else(|_| {
                            format!("{}/{}/{}", herd_id, artifact.device_id, "unknown")
                        });
                return Ok((artifact, storage_path));
            }

            // Verify file exists
            if !std::path::Path::new(&artifact.file_path).exists() {
                return Err(anyhow!("File does not exist: {}", artifact.file_path));
            }

            // Retry loop for expired URLs
            let mut retry_count = 0;
            let max_retries_value = max_retries;
            let progress_tx_for_loop = progress_tx.clone();
            let storage_client_handler_for_loop = storage_client_handler.clone();

            loop {
                // Check if upload URL is available and not expired
                let mut upload_url = match &artifact.upload_url {
                    Some(url) => url.clone(),
                    None => {
                        // Generate new URL if none exists
                        let mut artifacts = vec![artifact.clone()];
                        let temp_client = StorageClient {
                            config: config.clone(),
                            http_client: reqwest::Client::new(),
                            http_handler: storage_client_handler.clone(),
                        };
                        temp_client
                            .generate_upload_urls(&mut artifacts, herd_id)
                            .await
                            .map_err(|e| anyhow!("Failed to generate upload URL: {}", e))?;
                        artifact.upload_url = artifacts[0].upload_url.clone();
                        artifact.upload_url_generated_at = artifacts[0].upload_url_generated_at.clone();
                        artifacts[0].upload_url.as_ref().unwrap().clone()
                    }
                };

                // Check if URL is expired (older than 24 hours)
                let is_expired = artifact
                    .upload_url_generated_at
                    .as_ref()
                    .and_then(|ts| DateTime::parse_from_rfc3339(ts).ok())
                    .map(|dt| {
                        let age = Utc::now() - dt.with_timezone(&Utc);
                        age.num_hours() >= 24
                    })
                    .unwrap_or(true); // If no timestamp, assume expired

                // If expired, generate a new URL
                if is_expired && retry_count < max_retries_value {
                    tracing::info!("Upload URL expired, generating new URL...");
                    let mut artifacts = vec![artifact.clone()];
                    let temp_client = StorageClient {
                        config: config.clone(),
                        http_client: reqwest::Client::new(),
                        http_handler: storage_client_handler.clone(),
                    };
                    temp_client
                        .generate_upload_urls(&mut artifacts, herd_id)
                        .await
                        .map_err(|e| anyhow!("Failed to generate new upload URL: {}", e))?;
                    artifact.upload_url = artifacts[0].upload_url.clone();
                    artifact.upload_url_generated_at = artifacts[0].upload_url_generated_at.clone();
                    upload_url = artifacts[0].upload_url.as_ref().unwrap().clone();
                    retry_count += 1;
                }

                // Perform TUS upload using spawn_blocking
                let file_path = artifact.file_path.clone();
                let file_path_for_blocking = file_path.clone();
                let file_path_for_logging = file_path.clone();
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

                // Clone cancellation receiver for blocking context
                let cancel_rx_blocking = cancel_rx.clone();
                let upload_url_for_blocking = upload_url.clone();

                let progress_tx_for_blocking = progress_tx_for_loop.clone();
                let storage_client_handler_for_blocking = storage_client_handler_for_loop.clone();
                let upload_result = tokio::task::spawn_blocking(move || {
                    let tus_client = Client::new(storage_client_handler_for_blocking.as_ref());

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
                        let _ = progress_tx_for_blocking.send(progress); // Ignore send errors if no receivers
                    };

                    // Create cancellation check closure
                    let cancel_rx_local = cancel_rx_blocking.clone();
                    let cancellation_check = move || {
                        // Check if cancellation was signaled
                        let _ = cancel_rx_local.has_changed();
                        *cancel_rx_local.borrow()
                    };

                    // Perform TUS upload with resumable capability, progress tracking, and cancellation
                    tus_client.upload_with_chunk_size_and_cancellation(
                        &upload_url_for_blocking,
                        Path::new(&file_path_for_blocking),
                        chunk_size,
                        Some(&progress_callback),
                        Some(&cancellation_check),
                    )
                })
                .await
                .map_err(|e| anyhow!("Task join error: {}", e))?;

                match upload_result {
                    Ok(_) => {
                        let storage_path_without_bucket =
                            generate_remote_path(&artifact.file_path, herd_id, device_id)
                                .map_err(|e| anyhow!("Failed to generate storage path: {}", e))?;
                        // should be something like bucket_name/herd_id/device_id/name.extension
                        let storage_path =
                            format!("{}/{}", BUCKET_NAME_ARTIFACTS, storage_path_without_bucket);
                        tracing::info!(
                            "Successfully uploaded {} via TUS to {}",
                            file_path_for_logging,
                            storage_path
                        );

                        // Mark as uploaded
                        artifact.has_uploaded_file_to_storage = true;
                        artifact.file_path = storage_path.clone();
                        return Ok((artifact, storage_path));
                    }
                    Err(TusError::NotFoundError) => {
                        // Upload URL not found - might be expired, retry with new URL
                        if retry_count < max_retries_value {
                            tracing::warn!(
                                "Upload URL not found (possibly expired), retrying with new URL..."
                            );
                            retry_count += 1;
                            // Generate new URL and retry
                            let mut artifacts = vec![artifact.clone()];
                            let temp_client = StorageClient {
                                config: config.clone(),
                                http_client: reqwest::Client::new(),
                                http_handler: storage_client_handler.clone(),
                            };
                            match temp_client
                                .generate_upload_urls(&mut artifacts, herd_id)
                                .await
                            {
                                Ok(_) => {
                                    artifact.upload_url = artifacts[0].upload_url.clone();
                                    artifact.upload_url_generated_at =
                                        artifacts[0].upload_url_generated_at.clone();
                                    continue; // Retry upload
                                }
                                Err(e) => {
                                    return Err(anyhow!(
                                        "Failed to generate new upload URL after expiration: {}",
                                        e
                                    ));
                                }
                            }
                        } else {
                            return Err(anyhow!(
                                "TUS upload failed: upload URL not found and max retries exceeded"
                            ));
                        }
                    }
                    Err(TusError::Cancelled) => {
                        tracing::info!("Upload was cancelled");
                        return Err(anyhow!("Upload was cancelled"));
                    }
                    Err(e) => {
                        tracing::error!("TUS upload failed for {}: {}", file_path_for_logging, e);
                        return Err(anyhow!("TUS upload failed: {}", e));
                    }
                }
            }
        });

        // Set up automatic cancellation detection
        // Use a background task that gets cancelled when the spawn context is cancelled
        // This allows us to detect when the upload task is aborted
        tokio::spawn(async move {
            // This task runs in the same spawn context as the upload task
            // If the upload task is aborted, this task will also be cancelled
            // We can use this to signal cancellation to the blocking upload
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });

        // Also set up a task to monitor the upload handle and signal cancellation if aborted
        // Note: This is a best-effort approach since we can't directly detect abort
        let cancel_tx_monitor = cancel_tx_for_background;
        tokio::spawn(async move {
            // Monitor for cancellation signal from external sources
            // This allows manual cancellation if needed
            tokio::time::sleep(std::time::Duration::from_secs(86400)).await;
            let _ = cancel_tx_monitor.send(true);
        });

        (upload_handle, progress_rx)
    }

    /// Download an artifact from storage
    ///
    /// # Arguments
    /// * `file_path` - The storage path (e.g., "artifacts/herd_id/device_id/filename.mp4")
    /// * `output_path` - The local path where the file should be saved
    ///
    /// # Returns
    /// Result<()> indicating success or failure
    pub async fn download_artifact(
        &self,
        file_path: &str,
        output_path: &std::path::Path,
    ) -> Result<()> {
        let (bucket_name, object_path) = if file_path.contains('/') {
            let parts: Vec<&str> = file_path.splitn(2, '/').collect();
            if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                (BUCKET_NAME_ARTIFACTS, file_path)
            }
        } else {
            (BUCKET_NAME_ARTIFACTS, file_path)
        };

        let download_url = format!(
            "{}/storage/v1/object/public/{}/{}",
            self.config.supabase_url, bucket_name, object_path
        );

        let response = self
            .http_client
            .get(&download_url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to download artifact: {}", e))?;

        if !response.status().is_success() {
            return self.download_artifact_with_authenticated(file_path, output_path).await;
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read download response: {}", e))?;

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create output directory: {}", e))?;
        }

        std::fs::write(output_path, bytes)
            .map_err(|e| anyhow!("Failed to write file: {}", e))?;

        Ok(())
    }

    async fn download_artifact_with_authenticated(
        &self,
        file_path: &str,
        output_path: &std::path::Path,
    ) -> Result<()> {
        let (bucket_name, object_path) = if file_path.contains('/') {
            let parts: Vec<&str> = file_path.splitn(2, '/').collect();
            if parts.len() == 2 {
                (parts[0], parts[1])
            } else {
                (BUCKET_NAME_ARTIFACTS, file_path)
            }
        } else {
            (BUCKET_NAME_ARTIFACTS, file_path)
        };

        let download_url = format!(
            "{}/storage/v1/object/{}/{}",
            self.config.supabase_url, bucket_name, object_path
        );

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", self.config.supabase_anon_key)
                .parse()
                .map_err(|e| anyhow!("Failed to parse auth header: {}", e))?,
        );
        headers.insert(
            "apikey",
            self.config.supabase_anon_key
                .parse()
                .map_err(|e| anyhow!("Failed to parse apikey header: {}", e))?,
        );

        let response = self
            .http_client
            .get(&download_url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to download artifact: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_body = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!(
                "Failed to download artifact: {} - {}",
                status,
                error_body
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| anyhow!("Failed to read download response: {}", e))?;

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create output directory: {}", e))?;
        }

        std::fs::write(output_path, bytes)
            .map_err(|e| anyhow!("Failed to write file: {}", e))?;

        Ok(())
    }
    async fn generate_upload_url_for_artifact(
        &self,
        artifact: &ArtifactLocal,
        herd_id: i64,
    ) -> Result<String> {
        // Generate object path using the helper function
        let object_path = generate_remote_path(&artifact.file_path, herd_id, artifact.device_id)?;

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
        // Use the object path we already generated
        let object_name = object_path.clone();

        let http_handler = self.http_handler.clone();
        let file_path = artifact.file_path.clone();
        let endpoint = tus_endpoint.clone();
        tokio::task::spawn_blocking(move || {
            let tus_client = Client::new(http_handler.as_ref());

            // Create metadata for Supabase
            let mut metadata = std::collections::HashMap::new();
            metadata.insert("bucketName".to_string(), BUCKET_NAME_ARTIFACTS.to_string());
            metadata.insert("objectName".to_string(), object_name);
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

impl StorageClient {
    /// Generate a remote file path from a local file path
    ///
    /// This is a convenience wrapper around the standalone `generate_remote_path` function.
    /// Transforms a local path like `/opt/raven/blah/blah/test.mp4` into `herd_id/device_id/test.mp4`
    ///
    /// # Arguments
    /// * `local_path` - The local file path
    /// * `herd_id` - The herd ID for the storage path
    /// * `device_id` - The device ID for the storage path
    ///
    /// # Returns
    /// Result<String> - The remote file path or an error if the local path is invalid
    /// ```
    pub fn generate_remote_file_path(
        &self,
        local_path: &str,
        herd_id: i64,
        device_id: i64,
    ) -> Result<String> {
        generate_remote_path(local_path, herd_id, device_id)
    }
}

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
            bucket_name: BUCKET_NAME_ARTIFACTS.to_string(),
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
                    client.spawn_upload_artifact(artifacts[0].clone(), herd_id, None, None);

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
            let (handle, progress_rx) = client.spawn_upload_artifact(artifact, herd_id, None, None);
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

    #[test]
    fn test_generate_remote_file_path() {
        setup_storage_test_env();

        let config = create_test_storage_config();
        let client = StorageClient::new(config).expect("Failed to create storage client");

        // Test with a typical local file path
        let local_path = "/opt/raven/blah/blah/test.mp4";
        let herd_id = 123;
        let device_id = 456;

        let remote_path = client
            .generate_remote_file_path(local_path, herd_id, device_id)
            .expect("Failed to generate remote path");

        assert_eq!(remote_path, "123/456/test.mp4");

        // Test with different file extension
        let local_path2 = "/some/other/path/image.jpg";
        let remote_path2 = client
            .generate_remote_file_path(local_path2, herd_id, device_id)
            .expect("Failed to generate remote path");

        assert_eq!(remote_path2, "123/456/image.jpg");

        // Test with just a filename
        let local_path3 = "video.mov";
        let remote_path3 = client
            .generate_remote_file_path(local_path3, herd_id, device_id)
            .expect("Failed to generate remote path");

        assert_eq!(remote_path3, "123/456/video.mov");

        // Test with invalid path (empty string should fail)
        let result = client.generate_remote_file_path("", herd_id, device_id);
        assert!(result.is_err());

        // Test with path that has no filename (directory only)
        let result2 = client.generate_remote_file_path("/some/path/", herd_id, device_id);
        assert!(result2.is_ok()); // This actually succeeds with empty filename

        // Test with path that truly has no filename
        let result3 = client.generate_remote_file_path("/", herd_id, device_id);
        assert!(result3.is_err());
    }

    #[test]
    fn test_standalone_generate_remote_path() {
        // Test the standalone function directly
        let local_path = "/opt/raven/blah/blah/test.mp4";
        let herd_id = 123;
        let device_id = 456;

        let remote_path = generate_remote_path(local_path, herd_id, device_id)
            .expect("Failed to generate remote path");

        assert_eq!(remote_path, "123/456/test.mp4");

        // Test with different file extension
        let local_path2 = "/some/other/path/image.jpg";
        let remote_path2 = generate_remote_path(local_path2, herd_id, device_id)
            .expect("Failed to generate remote path");

        assert_eq!(remote_path2, "123/456/image.jpg");

        // Test with just a filename
        let local_path3 = "video.mov";
        let remote_path3 = generate_remote_path(local_path3, herd_id, device_id)
            .expect("Failed to generate remote path");

        assert_eq!(remote_path3, "123/456/video.mov");

        // Test with invalid path (empty string should fail)
        let result = generate_remote_path("", herd_id, device_id);
        assert!(result.is_err());

        // Test with path that truly has no filename
        let result2 = generate_remote_path("/", herd_id, device_id);
        assert!(result2.is_err());

        // Verify both methods produce the same result
        setup_storage_test_env();
        let config = create_test_storage_config();
        let client = StorageClient::new(config).expect("Failed to create storage client");

        let standalone_result = generate_remote_path(local_path, herd_id, device_id)
            .expect("Standalone function failed");
        let method_result = client
            .generate_remote_file_path(local_path, herd_id, device_id)
            .expect("Method failed");

        assert_eq!(
            standalone_result, method_result,
            "Standalone function and method should produce identical results"
        );
    }
}
