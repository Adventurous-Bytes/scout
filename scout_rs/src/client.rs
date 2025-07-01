use serde::{ Deserialize, Serialize };
use chrono::{ DateTime, Utc };
use reqwest;
use anyhow::{ Result, anyhow };
use std::path::Path;
use std::fs;
use tracing::{ info, warn, error, debug };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseScoutStatus {
    Success,
    NotAuthorized,
    InvalidEvent,
    InvalidFile,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseScout<T> {
    pub status: ResponseScoutStatus,
    pub data: Option<T>,
}

impl<T> ResponseScout<T> {
    pub fn new(status: ResponseScoutStatus, data: Option<T>) -> Self {
        Self { status, data }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub id: u32,
    pub inserted_at: String,
    pub created_by: String,
    pub herd_id: u32,
    pub device_type: String,
    pub name: String,
    pub description: Option<String>,
    pub domain_name: Option<String>,
    pub altitude: Option<f32>,
    pub heading: Option<f32>,
    pub location: Option<String>,
    pub video_publisher_token: Option<String>,
    pub video_subscriber_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Herd {
    pub id: u32,
    pub inserted_at: String,
    pub created_by: String,
    pub is_public: bool,
    pub slug: String,
    pub description: String,
    pub earthranger_domain: Option<String>,
    pub earthranger_token: Option<String>,
    pub video_publisher_token: Option<String>,
    pub video_subscriber_token: Option<String>,
    pub video_server_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub message: Option<String>,
    pub media_url: Option<String>,
    pub file_path: Option<String>,
    pub location: String,
    pub altitude: f64,
    pub heading: f64,
    pub media_type: String,
    pub device_id: String,
    pub earthranger_url: Option<String>,
    pub timestamp_observation: String,
    pub is_public: bool,
}

impl Event {
    pub fn new(
        message: Option<String>,
        media_url: Option<String>,
        file_path: Option<String>,
        earthranger_url: Option<String>,
        latitude: f64,
        longitude: f64,
        altitude: f64,
        heading: f64,
        media_type: String,
        device_id: u32,
        timestamp_observation: u64,
        is_public: bool
    ) -> Self {
        let location = Self::format_location(latitude, longitude);
        let timestamp_observation = DateTime::from_timestamp(timestamp_observation as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        Self {
            message,
            media_url,
            file_path,
            location,
            altitude,
            heading,
            media_type,
            device_id: device_id.to_string(),
            earthranger_url,
            timestamp_observation,
            is_public,
        }
    }

    pub fn format_location(latitude: f64, longitude: f64) -> String {
        format!("Point({} {})", longitude, latitude)
    }

    pub fn set_observation_time(&mut self, timestamp_observation: u64) {
        self.timestamp_observation = DateTime::from_timestamp(timestamp_observation as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub conf: f64,
    pub observation_type: String,
    pub class_name: String,
    pub event_id: u32,
    pub manual: bool,
}

impl Tag {
    pub fn new(
        _class_id: u32,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        conf: f64,
        observation_type: String,
        class_name: String
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            conf,
            observation_type,
            class_name,
            event_id: 0,
            manual: false,
        }
    }

    pub fn update_event_id(&mut self, event_id: u32) {
        self.event_id = event_id;
    }
}

pub struct ScoutClient {
    pub scout_url: String,
    pub api_key: String,
    pub device: Option<Device>,
    pub herd: Option<Herd>,
    client: reqwest::Client,
}

impl ScoutClient {
    pub fn new(scout_url: String, api_key: String) -> Result<Self> {
        let client = reqwest::Client
            ::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            scout_url,
            api_key,
            device: None,
            herd: None,
            client,
        })
    }

    /// Identify and load device and herd information into client state
    /// This method fetches the device associated with the API key and its corresponding herd
    pub async fn identify(&mut self) -> Result<()> {
        info!("🔍 Identifying device and herd...");

        // Get device information
        let device_response = self.get_device().await?;
        let device = device_response.data.ok_or_else(|| anyhow!("Failed to get device data"))?;

        info!("   Device ID: {}", device.id);
        info!("   Device Name: {}", device.name);
        info!("   Herd ID: {}", device.herd_id);

        // Get herd information using the device's herd_id
        let herd_response = self.get_herd(Some(device.herd_id)).await?;
        let herd = herd_response.data.ok_or_else(|| anyhow!("Failed to get herd data"))?;

        info!("   Herd Slug: {}", herd.slug);
        info!("   Herd Description: {}", herd.description);

        info!("✅ Identification complete");
        Ok(())
    }

    pub async fn get_device(&mut self) -> Result<ResponseScout<Device>> {
        // Return cached device if available
        if let Some(device) = &self.device {
            debug!("Using cached device: {} (ID: {})", device.name, device.id);
            return Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(device.clone())));
        }

        debug!("Fetching device information from API");
        let url = format!("{}/devices", self.scout_url);
        let response = self.client.get(&url).header("Authorization", &self.api_key).send().await?;

        match response.status().as_u16() {
            200 => {
                let data: serde_json::Value = response.json().await?;
                let device_data = if data.is_array() {
                    data.as_array().unwrap()[0].clone()
                } else {
                    data
                };

                let device: Device = serde_json::from_value(device_data)?;
                debug!("Successfully fetched device: {} (ID: {})", device.name, device.id);
                self.device = Some(device.clone());
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(device)))
            }
            201 => {
                debug!("Device created successfully");
                Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
            }
            _ => {
                error!("Failed to get device: HTTP {}", response.status());
                Err(anyhow!("Failed to get device: HTTP {}", response.status()))
            }
        }
    }

    pub async fn get_herd(&mut self, herd_id: Option<u32>) -> Result<ResponseScout<Herd>> {
        let herd_id = if let Some(id) = herd_id {
            id
        } else if let Some(device) = &self.device {
            device.herd_id
        } else {
            return Err(anyhow!("No herd_id provided and no device data available"));
        };

        // Return cached herd if available
        if let Some(herd) = &self.herd {
            if herd.id == herd_id {
                debug!("Using cached herd: {} (ID: {})", herd.slug, herd.id);
                return Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(herd.clone())));
            }
        }

        debug!("Fetching herd information for herd_id: {}", herd_id);
        let url = format!("{}/herds/{}", self.scout_url, herd_id);
        let response = self.client.get(&url).header("Authorization", &self.api_key).send().await?;

        match response.status().as_u16() {
            200 => {
                let data: serde_json::Value = response.json().await?;
                let herd_data = if data.is_array() {
                    data.as_array().unwrap()[0].clone()
                } else {
                    data
                };

                match serde_json::from_value::<Herd>(herd_data.clone()) {
                    Ok(herd) => {
                        debug!("Successfully fetched herd: {} (ID: {})", herd.slug, herd.id);
                        self.herd = Some(herd.clone());
                        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(herd)))
                    }
                    Err(e) => {
                        error!("Failed to deserialize herd data: {}", e);
                        error!("Raw herd data that failed to deserialize: {}", herd_data);
                        Err(anyhow!("Failed to deserialize herd data: {}", e))
                    }
                }
            }
            201 => {
                debug!("Herd created successfully");
                Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
            }
            _ => {
                let status = response.status();
                let error_text = response.text().await?;
                error!("Failed to get herd: HTTP {} - {}", status, error_text);
                Err(anyhow!("Failed to get herd: HTTP {} - {}", status, error_text))
            }
        }
    }

    pub async fn post_event_with_tags(
        &self,
        event: &Event,
        tags: &[Tag],
        file_path: &str
    ) -> Result<ResponseScout<()>> {
        // Check if file exists
        if !std::path::Path::new(file_path).exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        let tags_json = serde_json::to_string(tags)?;
        let event_json = serde_json::to_string(event)?;

        let file_bytes = std::fs::read(file_path)?;
        let filename = std::path::Path
            ::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("Invalid filename"))?;

        let file_part = reqwest::multipart::Part::bytes(file_bytes).file_name(filename.to_string());

        let form = reqwest::multipart::Form
            ::new()
            .text("event", event_json)
            .text("tags", tags_json)
            .part("file", file_part);

        let response = self.client
            .post(&format!("{}/events", self.scout_url))
            .header("Authorization", &self.api_key)
            .multipart(form)
            .send().await?;

        match response.status().as_u16() {
            200 | 201 => { Ok(ResponseScout::new(ResponseScoutStatus::Success, None)) }
            _ => {
                let status = response.status();
                let error_text = response.text().await?;
                Err(anyhow!("Failed to post event: HTTP {} - {}", status, error_text))
            }
        }
    }

    /// Parse filename in the format: "device_id|timestamp|lat_underscore|lon_underscore|altitude|heading"
    /// Example: "29|1733351509|19_754824|-155_15393|10|0.jpg"
    pub fn parse_filename(&self, filename: &str) -> Result<(u32, u64, f64, f64, f64, f64, String)> {
        // Remove file extension
        let filename_raw = filename
            .split('.')
            .next()
            .ok_or_else(|| anyhow!("Invalid filename format: {}", filename))?;

        // Split by pipes
        let parts: Vec<&str> = filename_raw.split('|').collect();
        if parts.len() != 6 {
            return Err(anyhow!("Expected 6 parts in filename, got {}: {}", parts.len(), filename));
        }

        let device_id: u32 = parts[0]
            .parse()
            .map_err(|_| anyhow!("Invalid device_id in filename: {}", parts[0]))?;

        let timestamp: u64 = parts[1]
            .parse()
            .map_err(|_| anyhow!("Invalid timestamp in filename: {}", parts[1]))?;

        let latitude: f64 = parts[2]
            .replace('_', ".")
            .parse()
            .map_err(|_| anyhow!("Invalid latitude in filename: {}", parts[2]))?;

        let longitude: f64 = parts[3]
            .replace('_', ".")
            .parse()
            .map_err(|_| anyhow!("Invalid longitude in filename: {}", parts[3]))?;

        let altitude: f64 = parts[4]
            .parse()
            .map_err(|_| anyhow!("Invalid altitude in filename: {}", parts[4]))?;

        let heading: f64 = parts[5]
            .parse()
            .map_err(|_| anyhow!("Invalid heading in filename: {}", parts[5]))?;

        Ok((device_id, timestamp, latitude, longitude, altitude, heading, filename.to_string()))
    }

    /// Check if a file is an image based on its extension
    pub fn is_image_file(&self, filename: &str) -> bool {
        let ext = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp")
    }

    /// Upload multiple events and files in a batch
    /// This method is more efficient than uploading files one by one
    /// The batch_size parameter limits how many files are uploaded in a single request
    pub async fn post_events_batch(
        &self,
        events_and_files: &[(Event, Vec<Tag>, String)], // (event, tags, file_path)
        batch_size: usize
    ) -> Result<BatchUploadResult> {
        if events_and_files.is_empty() {
            return Ok(BatchUploadResult {
                total_batches: 0,
                successful_batches: 0,
                failed_batches: 0,
                total_files: 0,
                successful_uploads: 0,
                failed_uploads: 0,
                failed_files: Vec::new(),
                batch_errors: Vec::new(),
            });
        }

        let mut results = BatchUploadResult {
            total_batches: 0,
            successful_batches: 0,
            failed_batches: 0,
            total_files: events_and_files.len(),
            successful_uploads: 0,
            failed_uploads: 0,
            failed_files: Vec::new(),
            batch_errors: Vec::new(),
        };

        // Process files in batches
        for (batch_index, chunk) in events_and_files.chunks(batch_size).enumerate() {
            results.total_batches += 1;
            info!(
                "📦 Processing batch {}/{} ({} files)",
                batch_index + 1,
                (events_and_files.len() + batch_size - 1) / batch_size,
                chunk.len()
            );

            match self.post_single_batch(chunk).await {
                Ok(batch_result) => {
                    results.successful_batches += 1;
                    results.successful_uploads += batch_result.successful_uploads;
                    results.failed_uploads += batch_result.failed_uploads;
                    results.failed_files.extend(batch_result.failed_files);
                    info!(
                        "✅ Batch {} completed: {} successful, {} failed",
                        batch_index + 1,
                        batch_result.successful_uploads,
                        batch_result.failed_uploads
                    );
                }
                Err(e) => {
                    results.failed_batches += 1;
                    results.failed_uploads += chunk.len();
                    let error_msg = format!("Batch {} failed: {}", batch_index + 1, e);
                    results.batch_errors.push(error_msg.clone());
                    error!("❌ {}", error_msg);

                    // Add all files in this batch to failed files
                    for (_, _, file_path) in chunk {
                        let filename = std::path::Path
                            ::new(file_path)
                            .file_name()
                            .and_then(|name| name.to_str())
                            .unwrap_or("unknown")
                            .to_string();
                        results.failed_files.push(filename);
                    }
                }
            }
        }

        Ok(results)
    }

    /// Upload a single batch of events and files
    async fn post_single_batch(
        &self,
        events_and_files: &[(Event, Vec<Tag>, String)]
    ) -> Result<BatchResult> {
        let mut form = reqwest::multipart::Form::new();
        let mut batch_result = BatchResult {
            successful_uploads: 0,
            failed_uploads: 0,
            failed_files: Vec::new(),
        };

        for (index, (event, tags, file_path)) in events_and_files.iter().enumerate() {
            // Check if file exists
            if !std::path::Path::new(file_path).exists() {
                batch_result.failed_uploads += 1;
                let filename = std::path::Path
                    ::new(file_path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                batch_result.failed_files.push(filename);
                continue;
            }

            let tags_json = serde_json::to_string(tags)?;
            let event_json = serde_json::to_string(event)?;

            let file_bytes = std::fs::read(file_path)?;
            let filename = std::path::Path
                ::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("Invalid filename"))?;

            let file_part = reqwest::multipart::Part
                ::bytes(file_bytes)
                .file_name(filename.to_string());

            // Add to form with indexed keys
            form = form
                .text(format!("event_{}", index), event_json)
                .text(format!("tags_{}", index), tags_json)
                .part(format!("file_{}", index), file_part);
        }

        // Add batch metadata
        form = form.text("batch_size", events_and_files.len().to_string());

        let response = self.client
            .post(&format!("{}/events/batch", self.scout_url))
            .header("Authorization", &self.api_key)
            .multipart(form)
            .send().await?;

        let status = response.status();
        let response_text = response.text().await?;
        info!("Batch upload response: HTTP {} - {}", status, response_text);

        match status.as_u16() {
            200 | 201 => {
                // Parse the response to get individual results
                match serde_json::from_str::<BatchResponse>(&response_text) {
                    Ok(batch_response) => {
                        batch_result.successful_uploads = batch_response.successful_uploads;
                        batch_result.failed_uploads = batch_response.failed_uploads;
                        batch_result.failed_files = batch_response.failed_files;
                    }
                    Err(_) => {
                        // If we can't parse the response, assume all succeeded
                        batch_result.successful_uploads = events_and_files.len();
                    }
                }
                Ok(batch_result)
            }
            _ => {
                error!("Batch upload failed: HTTP {} - Response: {}", status, response_text);
                Err(anyhow!("Failed to post batch: HTTP {} - {}", status, response_text))
            }
        }
    }

    /// Upload a directory of images to Scout using batch uploads
    /// This is an optimized version of upload_directory that uses batch uploads
    pub async fn upload_directory_batch(
        &mut self,
        directory_path: &str,
        earthranger_url: Option<&str>,
        is_public: bool,
        message: Option<&str>,
        default_latitude: Option<f64>,
        default_longitude: Option<f64>,
        default_altitude: Option<f64>,
        default_heading: Option<f64>,
        batch_size: usize
    ) -> Result<BatchUploadResult> {
        let dir_path = Path::new(directory_path);
        if !dir_path.exists() || !dir_path.is_dir() {
            return Err(anyhow!("Directory does not exist: {}", directory_path));
        }

        // Get device ID from stored state or fetch it
        let device_id = if let Some(device) = &self.device {
            device.id
        } else {
            info!("📡 Getting device information...");
            let device_response = self.get_device().await?;
            let device = device_response.data.ok_or_else(|| anyhow!("Failed to get device data"))?;
            info!("   Device ID: {}", device.id);
            info!("   Device Name: {}", device.name);
            device.id
        };

        let mut events_and_files = Vec::new();
        let entries = fs::read_dir(dir_path)?;

        for entry in entries {
            let entry = entry?;
            let file_path = entry.path();
            let filename = file_path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("Invalid filename"))?;

            // Skip non-image files
            if !self.is_image_file(filename) {
                continue;
            }

            // Parse filename to extract metadata, with fallbacks
            let (final_device_id, timestamp, latitude, longitude, altitude, heading) = match
                self.parse_filename(filename)
            {
                Ok((parsed_device_id, ts, lat, lon, alt, hdg, _)) => {
                    // Use stored device_id if it doesn't match parsed one
                    let device_id_to_use = if parsed_device_id != device_id {
                        warn!(
                            "⚠️  Device ID mismatch: parsed={}, stored={}, using stored",
                            parsed_device_id,
                            device_id
                        );
                        device_id
                    } else {
                        device_id
                    };
                    (device_id_to_use, ts, lat, lon, alt, hdg)
                }
                Err(e) => {
                    debug!(
                        "Filename '{}' doesn't contain metadata: {}. Using defaults.",
                        filename,
                        e
                    );

                    // Use current timestamp as fallback
                    let current_timestamp = std::time::SystemTime
                        ::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    (
                        device_id,
                        current_timestamp,
                        default_latitude.unwrap_or(0.0),
                        default_longitude.unwrap_or(0.0),
                        default_altitude.unwrap_or(0.0),
                        default_heading.unwrap_or(0.0),
                    )
                }
            };

            // Create event
            let event = Event::new(
                message.map(|m| m.to_string()),
                Some("https://www.google.com".to_string()), // Default media URL
                None,
                earthranger_url.map(|url| url.to_string()),
                latitude,
                longitude,
                altitude,
                heading,
                "image".to_string(),
                final_device_id,
                timestamp,
                is_public
            );

            let file_path_str = file_path.to_str().ok_or_else(|| anyhow!("Invalid file path"))?;
            events_and_files.push((event, Vec::new(), file_path_str.to_string()));
        }

        info!(
            "🚀 Starting batch upload of {} files with batch size {}",
            events_and_files.len(),
            batch_size
        );

        self.post_events_batch(&events_and_files, batch_size).await
    }

    /// Upload a directory of images to Scout
    /// The directory should contain image files with filenames in the format:
    /// "device_id|timestamp|lat_underscore|lon_underscore|altitude|heading.ext"
    /// If filename parsing fails, default values will be used instead of skipping the file.
    /// The device ID is automatically retrieved from stored state or fetched from the API.
    /// This method now uses batch uploads internally for better performance.
    pub async fn upload_directory(
        &mut self,
        directory_path: &str,
        earthranger_url: Option<&str>,
        is_public: bool,
        message: Option<&str>,
        default_latitude: Option<f64>,
        default_longitude: Option<f64>,
        default_altitude: Option<f64>,
        default_heading: Option<f64>,
        batch_size: Option<usize>
    ) -> Result<UploadResult> {
        // Use batch upload internally with specified batch size or default of 20
        let batch_result = self.upload_directory_batch(
            directory_path,
            earthranger_url,
            is_public,
            message,
            default_latitude,
            default_longitude,
            default_altitude,
            default_heading,
            batch_size.unwrap_or(20) // Default batch size of 20
        ).await?;

        // Convert BatchUploadResult to UploadResult for backward compatibility
        let result = UploadResult {
            total_files: batch_result.total_files,
            successful_uploads: batch_result.successful_uploads,
            failed_uploads: batch_result.failed_uploads,
            failed_files: batch_result.failed_files,
        };

        Ok(result)
    }
}

#[derive(Debug, Clone)]
pub struct UploadResult {
    pub total_files: usize,
    pub successful_uploads: usize,
    pub failed_uploads: usize,
    pub failed_files: Vec<String>,
}

impl UploadResult {
    pub fn print_summary(&self) {
        info!("📊 Upload Summary:");
        info!("   Total files processed: {}", self.total_files);
        info!("   Successful uploads: {}", self.successful_uploads);
        info!("   Failed uploads: {}", self.failed_uploads);

        if !self.failed_files.is_empty() {
            info!("   Failed files:");
            for file in &self.failed_files {
                info!("     - {}", file);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub successful_uploads: usize,
    pub failed_uploads: usize,
    pub failed_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResponse {
    pub successful_uploads: usize,
    pub failed_uploads: usize,
    pub failed_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BatchUploadResult {
    pub total_batches: usize,
    pub successful_batches: usize,
    pub failed_batches: usize,
    pub total_files: usize,
    pub successful_uploads: usize,
    pub failed_uploads: usize,
    pub failed_files: Vec<String>,
    pub batch_errors: Vec<String>,
}

impl BatchUploadResult {
    pub fn print_summary(&self) {
        info!("📊 Batch Upload Summary:");
        info!("   Total batches processed: {}", self.total_batches);
        info!("   Successful batches: {}", self.successful_batches);
        info!("   Failed batches: {}", self.failed_batches);
        info!("   Total files processed: {}", self.total_files);
        info!("   Successful uploads: {}", self.successful_uploads);
        info!("   Failed uploads: {}", self.failed_uploads);

        if !self.failed_files.is_empty() {
            info!("   Failed files:");
            for file in &self.failed_files {
                info!("     - {}", file);
            }
        }

        if !self.batch_errors.is_empty() {
            info!("   Batch errors:");
            for error in &self.batch_errors {
                info!("     - {}", error);
            }
        }
    }
}

// cargo test --test scout_client
#[cfg(test)]
mod tests {
    use super::*;
    use std::{ time::{ SystemTime, UNIX_EPOCH } };

    #[test]
    fn test_format_location() {
        let location = Event::format_location(40.7128, -74.006);
        assert_eq!(location, "Point(-74.006 40.7128)");
    }

    #[test]
    fn test_event_creation() {
        let event = Event::new(
            Some("Test message".to_string()),
            Some("http://example.com/media".to_string()),
            None,
            Some("http://example.com/earthranger".to_string()),
            40.7128,
            -74.006,
            100.0,
            45.0,
            "image".to_string(),
            1,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            false
        );

        assert_eq!(event.message, Some("Test message".to_string()));
        assert_eq!(event.device_id, "1");
        assert_eq!(event.altitude, 100.0);
        assert_eq!(event.heading, 45.0);
    }

    #[test]
    fn test_tag_creation() {
        let tag = Tag::new(
            1,
            100.0,
            200.0,
            50.0,
            30.0,
            0.95,
            "auto".to_string(),
            "person".to_string()
        );

        assert_eq!(tag.x, 100.0);
        assert_eq!(tag.y, 200.0);
        assert_eq!(tag.conf, 0.95);
        assert_eq!(tag.class_name, "person");
        assert_eq!(tag.event_id, 0); // Should be 0 initially
        assert_eq!(tag.manual, false); // Should be false for auto-generated tags
    }

    #[tokio::test]
    async fn test_scout_client_integration() {
        // Skip this test if no API key is provided
        let api_key =
            "6d11b203de3068673ba0558641ee20c2a78ef8885c50852c5da872cdbb5f144e32edf3c96b8d872959f58aee362623df131caecca853905b34ebfec436b9cf50";

        let mut client = ScoutClient::new(
            "https://www.adventurelabs.earth/api/scout".to_string(),
            api_key.to_string()
        ).expect("Failed to create ScoutClient");

        // Test getting device
        info!("Testing get_device...");
        let device_response = client.get_device().await.expect("Failed to get device response");
        assert_eq!(
            device_response.status,
            ResponseScoutStatus::Success,
            "Expected success status for device request"
        );

        let device = device_response.data.expect("Expected device data in response");
        info!("✅ Successfully got device: {:?}", device);

        // Test getting herd using the device's herd_id
        let herd_id_value = device.herd_id;
        info!("Testing get_herd with herd_id: {}...", herd_id_value);

        let herd_response = client
            .get_herd(Some(herd_id_value)).await
            .expect("Failed to get herd response");
        assert_eq!(
            herd_response.status,
            ResponseScoutStatus::Success,
            "Expected success status for herd request"
        );

        let herd = herd_response.data.expect("Expected herd data in response");
        info!("✅ Successfully got herd: {:?}", herd);

        // Additional assertions to verify the data structure
        assert!(device.id > 0, "Device should have a valid 'id' field");
        assert!(device.name.len() > 0, "Device should have a valid 'name' field");
        assert!(herd.id > 0, "Herd should have a valid 'id' field");
        assert!(herd.slug.len() > 0, "Herd should have a valid 'slug' field");
    }

    #[tokio::test]
    async fn test_scout_client_error_handling() {
        // Test with invalid API key
        let mut client = ScoutClient::new(
            "https://example.com".to_string(),
            "invalid_api_key".to_string()
        ).expect("Failed to create ScoutClient");

        // Test getting device with invalid key
        match client.get_device().await {
            Ok(_response) => {
                assert!(false, "Expected error with invalid API key");
            }
            Err(e) => {
                info!("✅ Correctly returned error with invalid API key: {}", e);
            }
        }
    }

    #[test]
    fn test_parse_filename() {
        let client = ScoutClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        ).expect("Failed to create ScoutClient");

        // Test valid filename
        let filename = "29|1733351509|19_754824|-155_15393|10|0.jpg";
        let result = client.parse_filename(filename).expect("Failed to parse filename");

        assert_eq!(result.0, 29); // device_id
        assert_eq!(result.1, 1733351509); // timestamp
        assert_eq!(result.2, 19.754824); // latitude
        assert_eq!(result.3, -155.15393); // longitude
        assert_eq!(result.4, 10.0); // altitude
        assert_eq!(result.5, 0.0); // heading
        assert_eq!(result.6, filename); // original filename

        // Test invalid filename format
        let invalid_filename = "invalid_format.jpg";
        assert!(client.parse_filename(invalid_filename).is_err());

        // Test filename with wrong number of parts
        let wrong_parts = "29|1733351509|19_754824.jpg";
        assert!(client.parse_filename(wrong_parts).is_err());
    }

    #[test]
    fn test_is_image_file() {
        let client = ScoutClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        ).expect("Failed to create ScoutClient");

        // Test valid image files
        assert!(client.is_image_file("test.jpg"));
        assert!(client.is_image_file("test.jpeg"));
        assert!(client.is_image_file("test.png"));
        assert!(client.is_image_file("test.webp"));
        assert!(client.is_image_file("test.JPG"));
        assert!(client.is_image_file("test.PNG"));

        // Test non-image files
        assert!(!client.is_image_file("test.txt"));
        assert!(!client.is_image_file("test.pdf"));
        assert!(!client.is_image_file("test"));
        assert!(!client.is_image_file("test."));
    }

    #[test]
    fn test_upload_result() {
        let result = UploadResult {
            total_files: 10,
            successful_uploads: 8,
            failed_uploads: 2,
            failed_files: vec!["file1.jpg".to_string(), "file2.jpg".to_string()],
        };

        // Test print_summary doesn't panic
        result.print_summary();

        // Test with no failed files
        let success_result = UploadResult {
            total_files: 5,
            successful_uploads: 5,
            failed_uploads: 0,
            failed_files: Vec::new(),
        };
        success_result.print_summary();
    }

    #[test]
    fn test_filename_parsing_fallback() {
        let client = ScoutClient::new(
            "https://example.com".to_string(),
            "test_key".to_string()
        ).expect("Failed to create ScoutClient");

        // Test that invalid filenames are handled gracefully
        let invalid_filename = "invalid_format.jpg";
        assert!(client.parse_filename(invalid_filename).is_err());

        // Test that the parse_filename method returns the expected error
        let result = client.parse_filename(invalid_filename);
        match result {
            Ok(_) => panic!("Expected error for invalid filename"),
            Err(e) => {
                assert!(e.to_string().contains("Expected 6 parts in filename"));
            }
        }
    }

    #[tokio::test]
    async fn test_identify_method() {
        // Skip this test if no API key is provided
        let api_key =
            "6d11b203de3068673ba0558641ee20c2a78ef8885c50852c5da872cdbb5f144e32edf3c96b8d872959f58aee362623df131caecca853905b34ebfec436b9cf50";
        if api_key.is_empty() {
            info!("Skipping identify test - no API key provided");
            return;
        }

        let mut client = ScoutClient::new(
            "https://www.adventurelabs.earth/api/scout".to_string(),
            api_key.to_string()
        ).expect("Failed to create ScoutClient");

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
                info!("❌ Identify method failed: {}", e);
                // This is expected if no valid API key is provided
            }
        }
    }
}
