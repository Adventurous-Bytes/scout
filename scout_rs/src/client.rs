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
pub struct Session {
    pub id: Option<i64>,
    pub device_id: i64,
    pub timestamp_start: String,
    pub timestamp_end: String,
    pub inserted_at: Option<String>,
    pub software_version: String,
    pub locations: Option<String>, // WKT format string - optional for deserialization
    pub altitude_max: f64,
    pub altitude_min: f64,
    pub altitude_average: f64,
    pub velocity_max: f64,
    pub velocity_min: f64,
    pub velocity_average: f64,
    pub distance_total: f64,
    pub distance_max_from_start: f64,
    pub file_paths: Option<Vec<String>>, // text[] in DB
    pub earthranger_url: Option<String>, // text in DB
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionInput {
    pub device_id: i64,
    pub timestamp_start: String,
    pub timestamp_end: Option<String>,
    pub software_version: String,
    pub locations: String,
    pub altitude_max: f64,
    pub altitude_min: f64,
    pub altitude_average: f64,
    pub velocity_max: f64,
    pub velocity_min: f64,
    pub velocity_average: f64,
    pub distance_total: f64,
    pub distance_max_from_start: f64,
    pub file_paths: Option<Vec<String>>,
    pub earthranger_url: Option<String>,
}

impl Session {
    pub fn new(
        device_id: i64,
        timestamp_start: u64,
        timestamp_end: u64,
        software_version: String,
        locations_wkt: Option<String>,
        altitude_max: f64,
        altitude_min: f64,
        altitude_average: f64,
        velocity_max: f64,
        velocity_min: f64,
        velocity_average: f64,
        distance_total: f64,
        distance_max_from_start: f64
    ) -> Self {
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        let timestamp_end_str = DateTime::from_timestamp(timestamp_end as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        // Use the provided WKT location or default to a point at origin
        let locations = locations_wkt.unwrap_or_else(|| "Point(0 0)".to_string());

        Self {
            id: None,
            device_id,
            timestamp_start: timestamp_start_str,
            timestamp_end: timestamp_end_str,
            inserted_at: None,
            software_version,
            locations: Some(locations),
            altitude_max,
            altitude_min,
            altitude_average,
            velocity_max,
            velocity_min,
            velocity_average,
            distance_total,
            distance_max_from_start,
            file_paths: None,
            earthranger_url: None,
        }
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn update_timestamp_end(&mut self, timestamp_end: u64) {
        self.timestamp_end = DateTime::from_timestamp(timestamp_end as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();
    }

    /// Creates a new session with WKT location validation
    pub fn new_with_wkt_validation(
        device_id: i64,
        timestamp_start: u64,
        timestamp_end: u64,
        software_version: String,
        locations_wkt: Option<String>,
        altitude_max: f64,
        altitude_min: f64,
        altitude_average: f64,
        velocity_max: f64,
        velocity_min: f64,
        velocity_average: f64,
        distance_total: f64,
        distance_max_from_start: f64
    ) -> Result<Self> {
        // Validate WKT format if provided
        if let Some(ref wkt) = locations_wkt {
            if !Event::validate_wkt_format(wkt) {
                return Err(anyhow!("Invalid WKT format: {}", wkt));
            }
        }

        Ok(
            Self::new(
                device_id,
                timestamp_start,
                timestamp_end,
                software_version,
                locations_wkt,
                altitude_max,
                altitude_min,
                altitude_average,
                velocity_max,
                velocity_min,
                velocity_average,
                distance_total,
                distance_max_from_start
            )
        )
    }

    pub fn to_input(&self) -> SessionInput {
        SessionInput {
            device_id: self.device_id,
            timestamp_start: self.timestamp_start.clone(),
            timestamp_end: Some(self.timestamp_end.clone()),
            software_version: self.software_version.clone(),
            locations: self.locations.clone().unwrap_or_else(|| "Point(0 0)".to_string()),
            altitude_max: self.altitude_max,
            altitude_min: self.altitude_min,
            altitude_average: self.altitude_average,
            velocity_max: self.velocity_max,
            velocity_min: self.velocity_min,
            velocity_average: self.velocity_average,
            distance_total: self.distance_total,
            distance_max_from_start: self.distance_max_from_start,
            file_paths: self.file_paths.clone(),
            earthranger_url: self.earthranger_url.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connectivity {
    pub id: Option<i64>,
    pub session_id: i64,
    pub inserted_at: Option<String>,
    pub timestamp_start: String,
    pub signal: f64,
    pub noise: f64,
    pub altitude: f64,
    pub heading: f64,
    pub location: String,
    pub h14_index: String,
    pub h13_index: String,
    pub h12_index: String,
    pub h11_index: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectivityInput {
    pub session_id: i64,
    pub timestamp_start: String,
    pub signal: f64,
    pub noise: f64,
    pub altitude: f64,
    pub heading: f64,
    pub location: String,
    pub h14_index: String,
    pub h13_index: String,
    pub h12_index: String,
    pub h11_index: String,
}

impl Connectivity {
    pub fn new(
        session_id: i64,
        timestamp_start: u64,
        signal: f64,
        noise: f64,
        altitude: f64,
        heading: f64,
        location: String,
        h14_index: String,
        h13_index: String,
        h12_index: String,
        h11_index: String
    ) -> Self {
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        Self {
            id: None,
            session_id,
            inserted_at: None,
            timestamp_start: timestamp_start_str,
            signal,
            noise,
            altitude,
            heading,
            location,
            h14_index,
            h13_index,
            h12_index,
            h11_index,
        }
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn to_input(&self) -> ConnectivityInput {
        ConnectivityInput {
            session_id: self.session_id,
            timestamp_start: self.timestamp_start.clone(),
            signal: self.signal,
            noise: self.noise,
            altitude: self.altitude,
            heading: self.heading,
            location: self.location.clone(),
            h14_index: self.h14_index.clone(),
            h13_index: self.h13_index.clone(),
            h12_index: self.h12_index.clone(),
            h11_index: self.h11_index.clone(),
        }
    }

    /// Creates a new connectivity record with WKT location validation
    pub fn new_with_wkt_validation(
        session_id: i64,
        timestamp_start: u64,
        signal: f64,
        noise: f64,
        altitude: f64,
        heading: f64,
        location: String,
        h14_index: String,
        h13_index: String,
        h12_index: String,
        h11_index: String
    ) -> Result<Self> {
        // Validate WKT format
        if !Event::validate_wkt_format(&location) {
            return Err(anyhow!("Invalid WKT format: {}", location));
        }

        Ok(
            Self::new(
                session_id,
                timestamp_start,
                signal,
                noise,
                altitude,
                heading,
                location,
                h14_index,
                h13_index,
                h12_index,
                h11_index
            )
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: Option<i64>,
    pub message: Option<String>,
    pub media_url: Option<String>,
    pub file_path: Option<String>,
    pub location: Option<String>, // Can be WKT string or hex WKB format
    pub altitude: f64,
    pub heading: f64,
    pub media_type: String,
    pub device_id: Option<i64>, // Server returns as integer
    pub earthranger_url: Option<String>,
    pub timestamp_observation: String,
    pub is_public: bool,
    pub session_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventInput {
    pub message: Option<String>,
    pub media_url: Option<String>,
    pub file_path: Option<String>,
    pub location: String,
    pub altitude: f64,
    pub heading: f64,
    pub media_type: String,
    pub device_id: i64,
    pub earthranger_url: Option<String>,
    pub timestamp_observation: String,
    pub is_public: bool,
    pub session_id: Option<i64>,
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
        is_public: bool,
        session_id: Option<i64>
    ) -> Self {
        let location = Self::format_location(latitude, longitude);
        let timestamp_observation = DateTime::from_timestamp(timestamp_observation as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        Self {
            id: None,
            message,
            media_url,
            file_path,
            location: Some(location),
            altitude,
            heading,
            media_type,
            device_id: Some(device_id as i64),
            earthranger_url,
            timestamp_observation,
            is_public,
            session_id,
        }
    }

    pub fn format_location(latitude: f64, longitude: f64) -> String {
        format!("Point({} {})", longitude, latitude)
    }

    /// Validates that a location string follows acceptable WKT format
    pub fn validate_wkt_format(location: &str) -> bool {
        // Check for common WKT geometry types
        if location.starts_with("POINT(") || location.starts_with("Point(") {
            // Point format: Point(longitude latitude)
            let coords = location
                .trim_start_matches("POINT(")
                .trim_start_matches("Point(")
                .trim_end_matches(")");
            return Self::validate_coordinate_pair(coords);
        } else if location.starts_with("LINESTRING(") || location.starts_with("LineString(") {
            // LineString format: LineString(longitude1 latitude1, longitude2 latitude2, ...)
            let coords = location
                .trim_start_matches("LINESTRING(")
                .trim_start_matches("LineString(")
                .trim_end_matches(")");
            return coords.split(',').all(|pair| Self::validate_coordinate_pair(pair.trim()));
        } else if location.starts_with("POLYGON(") || location.starts_with("Polygon(") {
            // Polygon format: Polygon((longitude1 latitude1, longitude2 latitude2, ...))
            let coords = location
                .trim_start_matches("POLYGON(")
                .trim_start_matches("Polygon(")
                .trim_end_matches(")");
            // Remove outer parentheses for polygon
            let coords = coords.trim_start_matches("(").trim_end_matches(")");
            return coords.split(',').all(|pair| Self::validate_coordinate_pair(pair.trim()));
        }
        false
    }

    /// Validates a single coordinate pair (longitude latitude)
    fn validate_coordinate_pair(coords: &str) -> bool {
        let parts: Vec<&str> = coords.split_whitespace().collect();
        if parts.len() != 2 {
            return false;
        }

        // Check if both parts can be parsed as f64
        parts[0].parse::<f64>().is_ok() && parts[1].parse::<f64>().is_ok()
    }

    pub fn set_observation_time(&mut self, timestamp_observation: u64) {
        self.timestamp_observation = DateTime::from_timestamp(timestamp_observation as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn to_input(&self) -> EventInput {
        EventInput {
            message: self.message.clone(),
            media_url: self.media_url.clone(),
            file_path: self.file_path.clone(),
            location: self.location.clone().unwrap_or_else(|| "Point(0 0)".to_string()),
            altitude: self.altitude,
            heading: self.heading,
            media_type: self.media_type.clone(),
            device_id: self.device_id.unwrap_or(0),
            earthranger_url: self.earthranger_url.clone(),
            timestamp_observation: self.timestamp_observation.clone(),
            is_public: self.is_public,
            session_id: self.session_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: Option<i64>,
    pub inserted_at: Option<String>,
    pub name: String,
    pub instructions: String,
    pub herd_id: i64,
    pub plan_type: String,
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
    /// Creates a new ScoutClient instance.
    ///
    /// # Arguments
    ///
    /// * `scout_url` - The base URL of the Scout API (e.g., "https://api.example.com/api/scout")
    /// * `api_key` - The API key for authentication
    ///
    /// # Returns
    ///
    /// A `Result<ScoutClient>` containing the initialized client or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// use scout_rs::client::ScoutClient;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = ScoutClient::new(
    ///         "https://api.example.com/api/scout".to_string(),
    ///         "your_api_key_here".to_string()
    ///     )?;
    ///     Ok(())
    /// }
    /// ```
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

    /// Identifies and loads device and herd information into the client state.
    ///
    /// This method fetches the device associated with the API key and its corresponding herd,
    /// storing the information in the client for future use. This is useful for reducing
    /// API calls by caching device and herd data.
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// client.identify().await?;
    /// // Device and herd information is now cached in the client
    /// # Ok(())
    /// # }
    /// ```
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

    /// Retrieves the device information associated with the current API key.
    ///
    /// This method fetches the device details from the API and caches the result
    /// in the client state for future use. If the device is already cached, it
    /// returns the cached version to avoid unnecessary API calls.
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Device>>` containing the device information or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let device_response = client.get_device().await?;
    /// if let Some(device) = device_response.data {
    ///     println!("Device ID: {}", device.id);
    ///     println!("Device Name: {}", device.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    /// Helper function to handle HTTP response status codes and return appropriate ResponseScout status
    fn handle_response_status<T>(status: u16, response_text: String) -> Result<ResponseScout<T>> {
        match status {
            200 | 201 => {
                // Success cases - let the caller handle the data parsing
                Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
            }
            401 => {
                error!("Unauthorized (401): {}", response_text);
                Ok(ResponseScout::new(ResponseScoutStatus::NotAuthorized, None))
            }
            403 => {
                error!("Forbidden (403): {}", response_text);
                Ok(ResponseScout::new(ResponseScoutStatus::NotAuthorized, None))
            }
            404 => {
                error!("Not Found (404): {}", response_text);
                Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
            }
            _ => {
                error!("HTTP {}: {}", status, response_text);
                Err(anyhow!("HTTP {}: {}", status, response_text))
            }
        }
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
        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let data: serde_json::Value = serde_json::from_str(&response_text)?;
                let device_data = if data.is_array() {
                    let array = data.as_array().unwrap();
                    if array.is_empty() {
                        return Err(anyhow!("Device response is an empty array"));
                    }
                    array[0].clone()
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
            _ => { Self::handle_response_status::<Device>(status, response_text) }
        }
    }

    /// Retrieves herd information by herd ID.
    ///
    /// This method fetches herd details from the API and caches the result
    /// in the client state. If no herd_id is provided, it uses the herd_id
    /// from the cached device information.
    ///
    /// # Arguments
    ///
    /// * `herd_id` - Optional herd ID. If None, uses the herd_id from the cached device
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Herd>>` containing the herd information or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let herd_response = client.get_herd(Some(123)).await?;
    /// if let Some(herd) = herd_response.data {
    ///     println!("Herd Slug: {}", herd.slug);
    ///     println!("Herd Description: {}", herd.description);
    /// }
    /// # Ok(())
    /// # }
    /// ```
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
        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let data: serde_json::Value = serde_json::from_str(&response_text)?;
                let herd_data = if data.is_array() {
                    let array = data.as_array().unwrap();
                    if array.is_empty() {
                        return Err(anyhow!("Herd response is an empty array"));
                    }
                    array[0].clone()
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
            _ => { Self::handle_response_status::<Herd>(status, response_text) }
        }
    }

    /// Posts an event with associated tags and media file to the Scout API.
    ///
    /// This method uploads an event along with its tags and a media file (image/video)
    /// to the Scout API. The file is uploaded as multipart form data.
    ///
    /// # Arguments
    ///
    /// * `event` - The event data to upload
    /// * `tags` - Array of tags associated with the event
    /// * `file_path` - Path to the media file to upload
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<()>>` indicating success or failure
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Event, Tag};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let event = Event::new(
    ///     Some("Trail camera detection".to_string()),
    ///     Some("https://example.com/image.jpg".to_string()),
    ///     None,
    ///     None,
    ///     19.754824,
    ///     -155.15393,
    ///     10.0,
    ///     0.0,
    ///     "image".to_string(),
    ///     123,
    ///     1733351509,
    ///     false,
    ///     None
    /// );
    /// let tags = vec![Tag::new(
    ///     1,
    ///     100.0,
    ///     200.0,
    ///     50.0,
    ///     30.0,
    ///     0.95,
    ///     "manual".to_string(),
    ///     "animal".to_string()
    /// )];
    /// let response = client.create_event_with_tags(&event, &tags, "path/to/image.jpg").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_event_with_tags(
        &self,
        event: &Event,
        tags: &[Tag],
        file_path: &str
    ) -> Result<ResponseScout<Event>> {
        // Check if file exists
        if !std::path::Path::new(file_path).exists() {
            return Err(anyhow!("File does not exist: {}", file_path));
        }

        let tags_json = serde_json::to_string(tags)?;
        let event_input = event.to_input();
        let event_json = serde_json::to_string(&event_input)?;

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

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 | 201 => {
                // Try to parse the response to get the full Event object
                let created_event = if response_text.trim().is_empty() {
                    // If response is empty, we can't parse the event
                    None
                } else {
                    // Try to parse JSON response to get the full Event object
                    match serde_json::from_str::<Event>(&response_text) {
                        Ok(event) => Some(event),
                        Err(_) => {
                            // If full Event parsing fails, try to create Event with ID from response
                            match serde_json::from_str::<serde_json::Value>(&response_text) {
                                Ok(json) => {
                                    // Look for id field in the response
                                    let event_id = json
                                        .get("id")
                                        .and_then(|v| v.as_i64())
                                        .or_else(|| json.get("event_id").and_then(|v| v.as_i64()));

                                    if let Some(id) = event_id {
                                        // Create a new Event with the ID from the response
                                        let mut event_with_id = event.clone();
                                        event_with_id.id = Some(id);
                                        Some(event_with_id)
                                    } else {
                                        None
                                    }
                                }
                                Err(_) => {
                                    // If JSON parsing fails, try to parse as plain number for ID
                                    if let Ok(id) = response_text.trim().parse::<i64>() {
                                        let mut event_with_id = event.clone();
                                        event_with_id.id = Some(id);
                                        Some(event_with_id)
                                    } else {
                                        None
                                    }
                                }
                            }
                        }
                    }
                };

                Ok(ResponseScout::new(ResponseScoutStatus::Success, created_event))
            }
            _ => { Self::handle_response_status::<Event>(status, response_text) }
        }
    }

    /// Parses a filename in the Scout format to extract metadata.
    ///
    /// The expected filename format is: "device_id|timestamp|lat_underscore|lon_underscore|altitude|heading"
    /// where underscores in lat/lon represent decimal points.
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to parse (e.g., "29|1733351509|19_754824|-155_15393|10|0.jpg")
    ///
    /// # Returns
    ///
    /// A `Result<(u32, u64, f64, f64, f64, f64, String)>` containing:
    /// - device_id (u32)
    /// - timestamp (u64)
    /// - latitude (f64)
    /// - longitude (f64)
    /// - altitude (f64)
    /// - heading (f64)
    /// - original filename (String)
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    ///     let (device_id, timestamp, lat, lon, alt, heading, filename) =
    ///         client.parse_filename("29|1733351509|19_754824|-155_15393|10|0.jpg")?;
    ///     println!("Device: {}, Lat: {}, Lon: {}", device_id, lat, lon);
    ///     Ok(())
    /// }
    /// ```
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

    /// Checks if a file is an image based on its file extension.
    ///
    /// Supports common image formats: jpg, jpeg, png, webp (case insensitive).
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to check
    ///
    /// # Returns
    ///
    /// `true` if the file is an image, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    ///     assert!(client.is_image_file("photo.jpg"));
    ///     assert!(client.is_image_file("image.PNG"));
    ///     assert!(!client.is_image_file("document.pdf"));
    ///     Ok(())
    /// }
    /// ```
    pub fn is_image_file(&self, filename: &str) -> bool {
        let ext = Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("")
            .to_lowercase();

        matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "webp")
    }

    /// Uploads multiple events and files in batches for improved efficiency.
    ///
    /// This method is more efficient than uploading files one by one as it groups
    /// multiple files into a single HTTP request. The batch_size parameter controls
    /// how many files are uploaded in each batch.
    ///
    /// # Arguments
    ///
    /// * `events_and_files` - Array of tuples containing (event, tags, file_path)
    /// * `batch_size` - Maximum number of files to upload in a single batch
    ///
    /// # Returns
    ///
    /// A `Result<BatchUploadResult>` containing detailed upload statistics
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Event, Tag};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let events_and_files = vec![
    ///     (Event::new(
    ///         Some("Detection 1".to_string()),
    ///         Some("https://example.com/file1.jpg".to_string()),
    ///         None,
    ///         None,
    ///         19.754824,
    ///         -155.15393,
    ///         10.0,
    ///         0.0,
    ///         "image".to_string(),
    ///         123,
    ///         1733351509,
    ///         false,
    ///         None
    ///     ), vec![Tag::new(
    ///         1,
    ///         100.0,
    ///         200.0,
    ///         50.0,
    ///         30.0,
    ///         0.95,
    ///         "manual".to_string(),
    ///         "animal".to_string()
    ///     )], "file1.jpg".to_string()),
    ///     (Event::new(
    ///         Some("Detection 2".to_string()),
    ///         Some("https://example.com/file2.jpg".to_string()),
    ///         None,
    ///         None,
    ///         19.754825,
    ///         -155.15394,
    ///         11.0,
    ///         5.0,
    ///         "image".to_string(),
    ///         123,
    ///         1733351510,
    ///         false,
    ///         None
    ///     ), vec![Tag::new(
    ///         1,
    ///         150.0,
    ///         250.0,
    ///         60.0,
    ///         40.0,
    ///         0.92,
    ///         "manual".to_string(),
    ///         "animal".to_string()
    ///     )], "file2.jpg".to_string()),
    /// ];
    /// let result = client.post_events_batch(&events_and_files, 10).await?;
    /// println!("Uploaded {} files successfully", result.successful_uploads);
    /// # Ok(())
    /// # }
    /// ```
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

    /// Uploads a single batch of events and files.
    ///
    /// This is an internal method used by `post_events_batch` to handle
    /// the actual HTTP request for a batch of files.
    ///
    /// # Arguments
    ///
    /// * `events_and_files` - Array of tuples containing (event, tags, file_path)
    ///
    /// # Returns
    ///
    /// A `Result<BatchResult>` containing the upload results for this batch
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
            let event_input = event.to_input();
            let event_json = serde_json::to_string(&event_input)?;

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

    /// Uploads a directory of images to Scout using optimized batch uploads.
    ///
    /// This method scans a directory for image files, parses their metadata from
    /// filenames (if available), and uploads them in batches for maximum efficiency.
    /// It's an optimized version of `upload_directory` that uses batch uploads.
    ///
    /// # Arguments
    ///
    /// * `directory_path` - Path to the directory containing images
    /// * `earthranger_url` - Optional EarthRanger URL for the events
    /// * `is_public` - Whether the events should be public
    /// * `message` - Optional message to include with all events
    /// * `default_latitude` - Default latitude if not found in filename
    /// * `default_longitude` - Default longitude if not found in filename
    /// * `default_altitude` - Default altitude if not found in filename
    /// * `default_heading` - Default heading if not found in filename
    /// * `batch_size` - Number of files to upload in each batch
    ///
    /// # Returns
    ///
    /// A `Result<BatchUploadResult>` containing detailed upload statistics
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let result = client.upload_directory_batch(
    ///     "/path/to/images",
    ///     Some("https://earthranger.example.com"),
    ///     true,
    ///     Some("Trail camera images"),
    ///     Some(19.754824),
    ///     Some(-155.15393),
    ///     Some(10.0),
    ///     Some(0.0),
    ///     20
    /// ).await?;
    /// println!("Uploaded {} files", result.successful_uploads);
    /// # Ok(())
    /// # }
    /// ```
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
                is_public,
                None
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

    /// Uploads a directory of images to Scout with automatic metadata parsing.
    ///
    /// This method scans a directory for image files and uploads them to Scout.
    /// It automatically parses metadata from filenames in the format:
    /// "device_id|timestamp|lat_underscore|lon_underscore|altitude|heading.ext"
    ///
    /// If filename parsing fails, default values will be used instead of skipping the file.
    /// The device ID is automatically retrieved from stored state or fetched from the API.
    /// This method uses batch uploads internally for better performance.
    ///
    /// # Arguments
    ///
    /// * `directory_path` - Path to the directory containing images
    /// * `earthranger_url` - Optional EarthRanger URL for the events
    /// * `is_public` - Whether the events should be public
    /// * `message` - Optional message to include with all events
    /// * `default_latitude` - Default latitude if not found in filename
    /// * `default_longitude` - Default longitude if not found in filename
    /// * `default_altitude` - Default altitude if not found in filename
    /// * `default_heading` - Default heading if not found in filename
    /// * `batch_size` - Optional batch size (defaults to 20 if None)
    ///
    /// # Returns
    ///
    /// A `Result<UploadResult>` containing upload statistics
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let result = client.upload_directory(
    ///     "/path/to/images",
    ///     Some("https://earthranger.example.com"),
    ///     true,
    ///     Some("Trail camera images"),
    ///     Some(19.754824),
    ///     Some(-155.15393),
    ///     Some(10.0),
    ///     Some(0.0),
    ///     Some(20)
    /// ).await?;
    /// println!("Uploaded {} files", result.successful_uploads);
    /// # Ok(())
    /// # }
    /// ```
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

    // ===== SESSION API METHODS =====

    /// Retrieves all sessions for a specific herd.
    ///
    /// This method fetches all sessions associated with the given herd ID,
    /// including detailed coordinate and statistics data.
    ///
    /// # Arguments
    ///
    /// * `herd_id` - The ID of the herd to get sessions for
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Vec<Session>>>` containing the sessions or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let response = client.get_sessions_by_herd(123).await?;
    /// if let Some(sessions) = response.data {
    ///     println!("Found {} sessions", sessions.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_sessions_by_herd(&self, herd_id: u32) -> Result<ResponseScout<Vec<Session>>> {
        debug!("Fetching sessions for herd_id: {}", herd_id);
        let url = format!("{}/sessions?herd_id={}", self.scout_url, herd_id);
        let response = self.client.get(&url).header("Authorization", &self.api_key).send().await?;
        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let sessions: Vec<Session> = serde_json::from_str(&response_text)?;
                debug!("Successfully fetched {} sessions for herd {}", sessions.len(), herd_id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(sessions)))
            }
            _ => { Self::handle_response_status::<Vec<Session>>(status, response_text) }
        }
    }

    /// Retrieves all plans for a specific herd.
    ///
    /// This method fetches all plans associated with the given herd ID,
    /// including plan names, instructions, and metadata.
    ///
    /// # Arguments
    ///
    /// * `herd_id` - The ID of the herd to get plans for
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Vec<Plan>>>` containing the plans or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let response = client.get_plans_by_herd(123).await?;
    /// if let Some(plans) = response.data {
    ///     println!("Found {} plans", plans.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_plans_by_herd(&self, herd_id: u32) -> Result<ResponseScout<Vec<Plan>>> {
        debug!("Fetching plans for herd_id: {}", herd_id);
        let url = format!("{}/plans?herd_id={}", self.scout_url, herd_id);
        let response = self.client.get(&url).header("Authorization", &self.api_key).send().await?;
        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let plans: Vec<Plan> = serde_json::from_str(&response_text)?;
                debug!("Successfully fetched {} plans for herd {}", plans.len(), herd_id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(plans)))
            }
            _ => { Self::handle_response_status::<Vec<Plan>>(status, response_text) }
        }
    }

    /// Creates or updates a single session.
    ///
    /// This method can be used to create new sessions or update existing ones.
    /// For new sessions (without ID), it uses `SessionInput` to avoid sending null fields.
    /// For existing sessions (with ID), it sends the full session data.
    ///
    /// # Arguments
    ///
    /// * `session` - The session data to create or update
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Session>>` containing the created/updated session or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Session};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let session = Session::new(
    ///     123,
    ///     1733351509,
    ///     1733351609,
    ///     "v1.0.0".to_string(),
    ///     None,
    ///     100.0,
    ///     50.0,
    ///     75.0,
    ///     25.0,
    ///     5.0,
    ///     15.0,
    ///     5000.0,
    ///     2500.0
    /// );
    /// let response = client.upsert_session(&session).await?;
    /// if let Some(created_session) = response.data {
    ///     println!("Session created with ID: {:?}", created_session.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upsert_session(&self, session: &Session) -> Result<ResponseScout<Session>> {
        debug!("Upserting session for device_id: {}", session.device_id);
        let url = format!("{}/sessions", self.scout_url);

        // For new sessions (without ID), use SessionInput to avoid sending null fields
        let payload = if session.id.is_none() {
            serde_json::to_value(session.to_input()).unwrap()
        } else {
            serde_json::to_value(session).unwrap()
        };

        let response = self.client
            .post(&url)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send().await?;

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            201 => {
                let created_session: Session = serde_json::from_str(&response_text)?;
                debug!("Successfully upserted session with ID: {:?}", created_session.id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_session)))
            }
            _ => { Self::handle_response_status::<Session>(status, response_text) }
        }
    }

    /// Creates or updates multiple sessions in a single batch operation.
    ///
    /// This method is more efficient than creating sessions one by one as it
    /// groups multiple sessions into a single HTTP request. Each session is
    /// converted to the appropriate format (SessionInput for new sessions).
    ///
    /// # Arguments
    ///
    /// * `sessions` - Array of sessions to create or update
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Vec<Session>>>` containing the created/updated sessions or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Session};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let sessions = vec![
    ///     Session::new(
    ///         123,
    ///         1733351509,
    ///         1733351609,
    ///         "v1.0.0".to_string(),
    ///         None,
    ///         100.0,
    ///         50.0,
    ///         75.0,
    ///         25.0,
    ///         5.0,
    ///         15.0,
    ///         5000.0,
    ///         2500.0
    ///     ),
    ///     Session::new(
    ///         124,
    ///         1733351610,
    ///         1733351710,
    ///         "v1.0.0".to_string(),
    ///         None,
    ///         110.0,
    ///         60.0,
    ///         85.0,
    ///         30.0,
    ///         8.0,
    ///         18.0,
    ///         6000.0,
    ///         3000.0
    ///     ),
    /// ];
    /// let response = client.upsert_sessions_batch(&sessions).await?;
    /// if let Some(created_sessions) = response.data {
    ///     println!("Created {} sessions", created_sessions.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upsert_sessions_batch(
        &self,
        sessions: &[Session]
    ) -> Result<ResponseScout<Vec<Session>>> {
        debug!("Upserting {} sessions in batch", sessions.len());
        let url = format!("{}/sessions", self.scout_url);

        // Convert sessions to appropriate format for batch upsert
        let payload: Vec<serde_json::Value> = sessions
            .iter()
            .map(|session| {
                if session.id.is_none() {
                    serde_json::to_value(session.to_input()).unwrap()
                } else {
                    serde_json::to_value(session).unwrap()
                }
            })
            .collect();

        let response = self.client
            .post(&url)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send().await?;

        match response.status().as_u16() {
            201 => {
                let created_sessions: Vec<Session> = response.json().await?;
                debug!("Successfully upserted {} sessions", created_sessions.len());
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_sessions)))
            }
            _ => {
                let status = response.status();
                let error_text = response.text().await?;
                error!("Failed to upsert sessions batch: HTTP {} - {}", status, error_text);
                Err(anyhow!("Failed to upsert sessions batch: HTTP {} - {}", status, error_text))
            }
        }
    }

    /// Updates a specific session by ID.
    ///
    /// This method updates an existing session with new data. The session ID
    /// is specified separately from the session data to ensure the correct
    /// session is updated.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to update
    /// * `session` - The new session data
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Session>>` containing the updated session or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Session};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let updated_session = Session::new(
    ///     123,
    ///     1733351509,
    ///     1733351609,
    ///     "v1.0.0".to_string(),
    ///     None,
    ///     100.0,
    ///     50.0,
    ///     75.0,
    ///     25.0,
    ///     5.0,
    ///     15.0,
    ///     5000.0,
    ///     2500.0
    /// );
    /// let response = client.update_session(123, &updated_session).await?;
    /// if let Some(session) = response.data {
    ///     println!("Session updated: {:?}", session.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_session(
        &self,
        session_id: i64,
        session: &Session
    ) -> Result<ResponseScout<Session>> {
        debug!("Updating session with ID: {}", session_id);
        let url = format!("{}/sessions/{}", self.scout_url, session_id);
        let response = self.client
            .put(&url)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(session)
            .send().await?;

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let updated_session: Session = serde_json::from_str(&response_text)?;
                debug!("Successfully updated session with ID: {}", session_id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(updated_session)))
            }
            _ => { Self::handle_response_status::<Session>(status, response_text) }
        }
    }

    /// Deletes a specific session by ID.
    ///
    /// This method permanently removes a session and all its associated data
    /// (connectivity entries, events, etc.) from the database.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to delete
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<()>>` indicating success or failure
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let response = client.delete_session(123).await?;
    /// println!("Session deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_session(&self, session_id: i64) -> Result<ResponseScout<()>> {
        debug!("Deleting session with ID: {}", session_id);
        let url = format!("{}/sessions/{}", self.scout_url, session_id);
        let response = self.client
            .delete(&url)
            .header("Authorization", &self.api_key)
            .send().await?;

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                debug!("Successfully deleted session with ID: {}", session_id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
            }
            _ => { Self::handle_response_status::<()>(status, response_text) }
        }
    }

    /// Retrieves all events for a specific session.
    ///
    /// This method fetches all events associated with the given session ID,
    /// including media files, tags, and metadata.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to get events for
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Vec<Event>>>` containing the events or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let response = client.get_session_events(123).await?;
    /// if let Some(events) = response.data {
    ///     println!("Found {} events", events.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_session_events(&self, session_id: i64) -> Result<ResponseScout<Vec<Event>>> {
        debug!("Fetching events for session_id: {}", session_id);
        let url = format!("{}/sessions/{}/events", self.scout_url, session_id);
        let response = self.client.get(&url).header("Authorization", &self.api_key).send().await?;

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let events: Vec<Event> = serde_json::from_str(&response_text)?;
                debug!("Successfully fetched {} events for session {}", events.len(), session_id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(events)))
            }
            _ => { Self::handle_response_status::<Vec<Event>>(status, response_text) }
        }
    }

    /// Deletes an event by its ID.
    ///
    /// This method removes an event from the database, which will also cascade delete
    /// any associated tags due to foreign key constraints.
    ///
    /// # Arguments
    ///
    /// * `event_id` - The ID of the event to delete
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<()>>` indicating success or failure
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, ResponseScoutStatus};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let response = client.delete_event(123).await?;
    /// if response.status == ResponseScoutStatus::Success {
    ///     println!("Event deleted successfully");
    /// }
    /// # Ok(())
    /// # }
    /// ```

    /// Updates an existing event by ID.
    ///
    /// This method allows you to modify event properties such as message, location,
    /// altitude, heading, media type, and other metadata. The event must already exist
    /// in the database with the specified ID.
    ///
    /// # Arguments
    ///
    /// * `event_id` - The ID of the event to update
    /// * `event` - The updated event data
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Event>>` containing the updated event or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Event};
    /// # use std::time::{SystemTime, UNIX_EPOCH};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let mut event = Event::new(
    ///     Some("Updated message".to_string()),
    ///     None,
    ///     None,
    ///     None,
    ///     40.7128,
    ///     -74.006,
    ///     100.0,
    ///     45.0,
    ///     "image".to_string(),
    ///     1,
    ///     SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    ///     false,
    ///     None
    /// );
    /// let event_with_id = event.with_id(123);
    /// let response = client.update_event(123, &event_with_id).await?;
    /// if let Some(updated_event) = response.data {
    ///     println!("Event updated successfully with ID: {:?}", updated_event.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_event(&self, event_id: i64, event: &Event) -> Result<ResponseScout<Event>> {
        debug!("Updating event with ID: {}", event_id);
        let url = format!("{}/events/{}", self.scout_url, event_id);
        let event_input = event.to_input();

        let response = self.client
            .put(&url)
            .header("Authorization", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&event_input)
            .send().await?;

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let updated_event: Event = serde_json::from_str(&response_text)?;
                debug!("Successfully updated event with ID: {}", event_id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(updated_event)))
            }
            _ => { Self::handle_response_status::<Event>(status, response_text) }
        }
    }

    pub async fn delete_event(&self, event_id: i64) -> Result<ResponseScout<()>> {
        debug!("Deleting event with ID: {}", event_id);
        let url = format!("{}/events/{}", self.scout_url, event_id);
        let response = self.client
            .delete(&url)
            .header("Authorization", &self.api_key)
            .send().await?;

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 | 204 => {
                debug!("Successfully deleted event with ID: {}", event_id);
                Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
            }
            _ => { Self::handle_response_status::<()>(status, response_text) }
        }
    }

    /// Retrieves all connectivity data for a specific session.
    ///
    /// This method fetches all connectivity entries associated with the given session ID,
    /// including signal strength, noise, altitude, heading, and location data.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to get connectivity data for
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Vec<Connectivity>>>` containing the connectivity data or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let response = client.get_session_connectivity(123).await?;
    /// if let Some(connectivity) = response.data {
    ///     println!("Found {} connectivity entries", connectivity.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_session_connectivity(
        &self,
        session_id: i64
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        debug!("Fetching connectivity for session_id: {}", session_id);
        let url = format!("{}/sessions/{}/connectivity", self.scout_url, session_id);
        let response = self.client.get(&url).header("Authorization", &self.api_key).send().await?;

        let status = response.status().as_u16();
        let response_text = response.text().await?;

        match status {
            200 => {
                let connectivity: Vec<Connectivity> = serde_json::from_str(&response_text)?;
                debug!(
                    "Successfully fetched {} connectivity entries for session {}",
                    connectivity.len(),
                    session_id
                );
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(connectivity)))
            }
            _ => { Self::handle_response_status::<Vec<Connectivity>>(status, response_text) }
        }
    }

    /// Creates a new session with the specified parameters and returns the session ID.
    ///
    /// This is a convenience method that creates a session using the provided parameters
    /// and returns just the session ID for easy reference.
    ///
    /// # Arguments
    ///
    /// * `device_id` - The ID of the device that created this session
    /// * `timestamp_start` - Unix timestamp when the session started
    /// * `timestamp_end` - Unix timestamp when the session ended
    /// * `software_version` - Version of the software that created this session
    /// * `locations_wkt` - Optional WKT (Well-Known Text) location data
    /// * `altitude_max` - Maximum altitude during the session
    /// * `altitude_min` - Minimum altitude during the session
    /// * `altitude_average` - Average altitude during the session
    /// * `velocity_max` - Maximum velocity during the session (m/s)
    /// * `velocity_min` - Minimum velocity during the session (m/s)
    /// * `velocity_average` - Average velocity during the session (m/s)
    /// * `distance_total` - Total distance traveled during the session (m)
    /// * `distance_max_from_start` - Maximum distance from start point (m)
    ///
    /// # Returns
    ///
    /// A `Result<i64>` containing the created session ID or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let session_id = client.create_session(
    ///     123,
    ///     1640995200, // 2022-01-01 00:00:00 UTC
    ///     1640998800, // 2022-01-01 01:00:00 UTC
    ///     "v1.0.0".to_string(),
    ///     None,
    ///     150.0,
    ///     50.0,
    ///     100.0,
    ///     25.0,
    ///     5.0,
    ///     15.0,
    ///     5000.0,
    ///     2500.0
    /// ).await?;
    /// println!("Created session with ID: {}", session_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_session(
        &self,
        device_id: i64,
        timestamp_start: u64,
        timestamp_end: u64,
        software_version: String,
        locations_wkt: Option<String>,
        altitude_max: f64,
        altitude_min: f64,
        altitude_average: f64,
        velocity_max: f64,
        velocity_min: f64,
        velocity_average: f64,
        distance_total: f64,
        distance_max_from_start: f64
    ) -> Result<i64> {
        let session = Session::new(
            device_id,
            timestamp_start,
            timestamp_end,
            software_version,
            locations_wkt,
            altitude_max,
            altitude_min,
            altitude_average,
            velocity_max,
            velocity_min,
            velocity_average,
            distance_total,
            distance_max_from_start
        );
        let response = self.upsert_session(&session).await?;

        match response.data {
            Some(created_session) => {
                created_session.id.ok_or_else(|| anyhow!("Session created but no ID returned"))
            }
            None => Err(anyhow!("Failed to create session: no data returned")),
        }
    }

    /// Creates or updates a single connectivity entry.
    ///
    /// This method can be used to create new connectivity entries or update existing ones.
    /// For new entries (without ID), it uses `ConnectivityInput` to avoid sending null fields.
    /// For existing entries (with ID), it sends the full connectivity data.
    ///
    /// # Arguments
    ///
    /// * `connectivity` - The connectivity data to create or update
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Connectivity>>` containing the created/updated connectivity entry or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Connectivity};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let connectivity = Connectivity::new(
    ///     123,
    ///     1733351509,
    ///     -50.0,
    ///     -60.0,
    ///     100.0,
    ///     45.0,
    ///     "Point(0 0)".to_string(),
    ///     "1".to_string(),
    ///     "2".to_string(),
    ///     "3".to_string(),
    ///     "4".to_string()
    /// );
    /// let response = client.upsert_connectivity(&connectivity).await?;
    /// if let Some(created_connectivity) = response.data {
    ///     println!("Connectivity entry created with ID: {:?}", created_connectivity.id);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upsert_connectivity(
        &self,
        connectivity: &Connectivity
    ) -> Result<ResponseScout<Connectivity>> {
        debug!("Upserting connectivity entry for session_id: {}", connectivity.session_id);
        let url = format!("{}/connectivity", self.scout_url);

        // For new connectivity entries (without ID), use ConnectivityInput to avoid sending null fields
        let payload = if connectivity.id.is_none() {
            serde_json::to_value(connectivity.to_input()).unwrap()
        } else {
            serde_json::to_value(connectivity).unwrap()
        };

        let response = self.client
            .post(&url)
            .header("Authorization", &self.api_key)
            .json(&payload)
            .send().await?;

        match response.status().as_u16() {
            200 | 201 => {
                let created_connectivity: Connectivity = response.json().await?;
                debug!(
                    "Successfully upserted connectivity entry with ID: {:?}",
                    created_connectivity.id
                );
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_connectivity)))
            }
            _ => {
                let status = response.status();
                let error_text = response.text().await?;
                error!("Failed to upsert connectivity: HTTP {} - {}", status, error_text);
                Err(anyhow!("Failed to upsert connectivity: HTTP {} - {}", status, error_text))
            }
        }
    }

    /// Upserts multiple connectivity entries in a single batch request.
    ///
    /// This method sends an array of connectivity entries to the API endpoint,
    /// which will handle batch upserting them. The API automatically detects
    /// whether to create new entries or update existing ones based on the
    /// presence of an ID field.
    ///
    /// # Arguments
    ///
    /// * `connectivities` - A slice of connectivity entries to upsert
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Vec<Connectivity>>>` containing the upserted connectivity entries or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::{ScoutClient, Connectivity};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let connectivities = vec![
    ///     Connectivity::new(123, 1640995200, -50.0, -90.0, 100.0, 45.0, "Point(-74.006 40.7128)".to_string(), "h14".to_string(), "h13".to_string(), "h12".to_string(), "h11".to_string()),
    ///     Connectivity::new(123, 1640995260, -55.0, -85.0, 105.0, 50.0, "Point(-74.007 40.7129)".to_string(), "h14".to_string(), "h13".to_string(), "h12".to_string(), "h11".to_string()),
    /// ];
    /// let response = client.upsert_connectivity_batch(&connectivities).await?;
    /// if let Some(upserted) = response.data {
    ///     println!("Successfully upserted {} connectivity entries", upserted.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upsert_connectivity_batch(
        &self,
        connectivities: &[Connectivity]
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        debug!("Upserting {} connectivity entries in batch", connectivities.len());
        let url = format!("{}/connectivity", self.scout_url);

        // Prepare the payload - use ConnectivityInput for new entries, full Connectivity for existing ones
        let payload: Vec<serde_json::Value> = connectivities
            .iter()
            .map(|conn| {
                if conn.id.is_none() {
                    serde_json::to_value(conn.to_input()).unwrap()
                } else {
                    serde_json::to_value(conn).unwrap()
                }
            })
            .collect();

        let response = self.client
            .post(&url)
            .header("Authorization", &self.api_key)
            .json(&payload)
            .send().await?;

        match response.status().as_u16() {
            200 | 201 => {
                let upserted_connectivities: Vec<Connectivity> = response.json().await?;
                debug!(
                    "Successfully upserted {} connectivity entries in batch",
                    upserted_connectivities.len()
                );
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(upserted_connectivities)))
            }
            _ => {
                let status = response.status();
                let error_text = response.text().await?;
                error!("Failed to upsert connectivity batch: HTTP {} - {}", status, error_text);
                Err(
                    anyhow!("Failed to upsert connectivity batch: HTTP {} - {}", status, error_text)
                )
            }
        }
    }

    /// Ends a session by updating its timestamp_end.
    ///
    /// This method updates an existing session to mark it as completed by
    /// setting the end timestamp. This is useful for sessions that are
    /// created at the start and need to be finalized when they end.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to end
    /// * `timestamp_end` - Unix timestamp when the session ended
    ///
    /// # Returns
    ///
    /// A `Result<()>` indicating success or failure
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let end_time = std::time::SystemTime::now()
    ///     .duration_since(std::time::UNIX_EPOCH)?
    ///     .as_secs();
    /// client.end_session(123, end_time).await?;
    /// println!("Session ended successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn end_session(&self, session_id: i64, timestamp_end: u64) -> Result<()> {
        let session = Session {
            id: Some(session_id),
            device_id: 0, // This will be ignored in the update
            timestamp_start: String::new(), // This will be ignored in the update
            timestamp_end: DateTime::from_timestamp(timestamp_end as i64, 0)
                .unwrap_or_else(|| Utc::now())
                .to_rfc3339(),
            inserted_at: None,
            software_version: String::new(), // This will be ignored in the update
            locations: Some("Point(0 0)".to_string()), // Required field, will be ignored in update
            altitude_max: 0.0, // This will be ignored in the update
            altitude_min: 0.0, // This will be ignored in the update
            altitude_average: 0.0, // This will be ignored in the update
            velocity_max: 0.0, // This will be ignored in the update
            velocity_min: 0.0, // This will be ignored in the update
            velocity_average: 0.0, // This will be ignored in the update
            distance_total: 0.0, // This will be ignored in the update
            distance_max_from_start: 0.0, // This will be ignored in the update
            file_paths: None,
            earthranger_url: None,
        };

        let response = self.update_session(session_id, &session).await?;
        match response.status {
            ResponseScoutStatus::Success => {
                debug!("Successfully ended session with ID: {}", session_id);
                Ok(())
            }
            _ => Err(anyhow!("Failed to end session")),
        }
    }

    /// Retrieves connectivity data for a specific session with coordinate information.
    ///
    /// This method uses a database RPC function to fetch connectivity entries with enhanced
    /// coordinate information, including extracted latitude/longitude coordinates in addition
    /// to the standard connectivity data.
    ///
    /// # Arguments
    ///
    /// * `session_id` - The ID of the session to get connectivity data for
    ///
    /// # Returns
    ///
    /// A `Result<ResponseScout<Vec<ConnectivityWithCoordinates>>>` containing connectivity data with coordinates or an error
    ///
    /// # Example
    ///
    /// ```rust
    /// # use scout_rs::client::ScoutClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ScoutClient::new("https://api.example.com/api/scout".to_string(), "api_key".to_string())?;
    /// let response = client.get_session_connectivity_with_coordinates(123).await?;
    /// if let Some(connectivity) = response.data {
    ///     for entry in connectivity {
    ///         println!("Connectivity at lat: {}, lon: {}", entry.latitude, entry.longitude);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_session_connectivity_with_coordinates(
        &self,
        session_id: i64
    ) -> Result<ResponseScout<Vec<ConnectivityWithCoordinates>>> {
        debug!("Fetching connectivity with coordinates for session_id: {}", session_id);
        let url = format!(
            "{}/sessions/{}/connectivity/with-coordinates",
            self.scout_url,
            session_id
        );
        let response = self.client.get(&url).header("Authorization", &self.api_key).send().await?;

        match response.status().as_u16() {
            200 => {
                let connectivity: Vec<ConnectivityWithCoordinates> = response.json().await?;
                debug!(
                    "Successfully fetched {} connectivity entries with coordinates for session {}",
                    connectivity.len(),
                    session_id
                );
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(connectivity)))
            }
            _ => {
                let status = response.status();
                let error_text = response.text().await?;
                error!(
                    "Failed to get connectivity with coordinates: HTTP {} - {}",
                    status,
                    error_text
                );
                Err(
                    anyhow!(
                        "Failed to get connectivity with coordinates: HTTP {} - {}",
                        status,
                        error_text
                    )
                )
            }
        }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectivityWithCoordinates {
    pub id: Option<i64>,
    pub session_id: i64,
    pub inserted_at: Option<String>,
    pub timestamp_start: String,
    pub signal: f64,
    pub noise: f64,
    pub altitude: f64,
    pub heading: f64,
    pub latitude: f64,
    pub longitude: f64,
    pub h14_index: String,
    pub h13_index: String,
    pub h12_index: String,
    pub h11_index: String,
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
    fn test_validate_wkt_format() {
        // Valid Point formats
        assert!(Event::validate_wkt_format("Point(-74.006 40.7128)"));
        assert!(Event::validate_wkt_format("POINT(-74.006 40.7128)"));

        // Valid LineString formats
        assert!(Event::validate_wkt_format("LineString(-118.4079 33.9434, 2.5559 49.0083)"));
        assert!(Event::validate_wkt_format("LINESTRING(-118.4079 33.9434, 2.5559 49.0083)"));

        // Valid Polygon formats
        assert!(
            Event::validate_wkt_format(
                "Polygon((-74.006 40.7128, -74.007 40.7128, -74.007 40.7129, -74.006 40.7129, -74.006 40.7128))"
            )
        );
        assert!(
            Event::validate_wkt_format(
                "POLYGON((-74.006 40.7128, -74.007 40.7128, -74.007 40.7129, -74.006 40.7129, -74.006 40.7128))"
            )
        );

        // Invalid formats
        assert!(!Event::validate_wkt_format("Invalid"));
        assert!(!Event::validate_wkt_format("Point(invalid coordinates)"));
        assert!(!Event::validate_wkt_format("LineString(-118.4079 33.9434, invalid)"));
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
            false,
            None
        );

        assert_eq!(event.message, Some("Test message".to_string()));
        assert_eq!(event.device_id, Some(1));
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
    }

    #[test]
    fn test_event_with_id() {
        let event = Event::new(
            Some("Test message".to_string()),
            None,
            None,
            None,
            40.7128,
            -74.006,
            100.0,
            45.0,
            "image".to_string(),
            1,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            false,
            None
        );

        let event_with_id = event.with_id(123);
        assert_eq!(event_with_id.id, Some(123));
        assert_eq!(event_with_id.message, Some("Test message".to_string()));
    }

    #[test]
    fn test_handle_response_status_helper() {
        info!("🧪 Testing handle_response_status helper function");

        // Test 401 handling
        let result = ScoutClient::handle_response_status::<String>(401, "Unauthorized".to_string());
        match result {
            Ok(response) => {
                assert_eq!(response.status, ResponseScoutStatus::NotAuthorized);
                info!("✅ 401 correctly mapped to NotAuthorized");
            }
            Err(_) => {
                assert!(false, "401 should return Ok with NotAuthorized status");
            }
        }

        // Test 403 handling
        let result = ScoutClient::handle_response_status::<String>(403, "Forbidden".to_string());
        match result {
            Ok(response) => {
                assert_eq!(response.status, ResponseScoutStatus::NotAuthorized);
                info!("✅ 403 correctly mapped to NotAuthorized");
            }
            Err(_) => {
                assert!(false, "403 should return Ok with NotAuthorized status");
            }
        }

        // Test 404 handling
        let result = ScoutClient::handle_response_status::<String>(404, "Not Found".to_string());
        match result {
            Ok(response) => {
                assert_eq!(response.status, ResponseScoutStatus::Failure);
                info!("✅ 404 correctly mapped to Failure");
            }
            Err(_) => {
                assert!(false, "404 should return Ok with Failure status");
            }
        }

        // Test 500 handling
        let result = ScoutClient::handle_response_status::<String>(
            500,
            "Internal Server Error".to_string()
        );
        match result {
            Ok(_) => {
                assert!(false, "500 should return Err");
            }
            Err(_) => {
                info!("✅ 500 correctly returned Err");
            }
        }

        info!("✅ All handle_response_status tests completed successfully");
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

    // ===== SESSION TESTS =====

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
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

        assert_eq!(session.device_id, 123);
        assert_eq!(session.software_version, "v1.0.0");
        assert!(session.id.is_none());
    }

    #[test]
    fn test_session_creation_with_wkt() {
        let session = Session::new(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "v1.0.0".to_string(),
            Some("Point(-74.006 40.7128)".to_string()),
            100.0,
            50.0,
            75.0,
            10.0,
            5.0,
            7.5,
            1000.0,
            500.0
        );

        assert_eq!(session.device_id, 123);
        assert_eq!(session.software_version, "v1.0.0");
        assert_eq!(session.locations, Some("Point(-74.006 40.7128)".to_string()));
    }

    #[test]
    fn test_session_creation_with_wkt_validation() {
        // Valid WKT
        let session = Session::new_with_wkt_validation(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "v1.0.0".to_string(),
            Some("LineString(-118.4079 33.9434, 2.5559 49.0083)".to_string()),
            100.0,
            50.0,
            75.0,
            10.0,
            5.0,
            7.5,
            1000.0,
            500.0
        ).unwrap();

        assert_eq!(session.device_id, 123);
        assert_eq!(
            session.locations,
            Some("LineString(-118.4079 33.9434, 2.5559 49.0083)".to_string())
        );

        // Invalid WKT should fail
        let result = Session::new_with_wkt_validation(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            "v1.0.0".to_string(),
            Some("Invalid WKT".to_string()),
            100.0,
            50.0,
            75.0,
            10.0,
            5.0,
            7.5,
            1000.0,
            500.0
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_session_with_id() {
        let session = Session::new(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
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
        ).with_id(456);

        assert_eq!(session.id, Some(456));
        assert_eq!(session.device_id, 123);
    }

    #[test]
    fn test_session_update_methods() {
        let mut session = Session::new(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
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

        let new_end_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600;
        session.update_timestamp_end(new_end_time);

        // The timestamp should be updated (we can't easily test the exact string format)
        assert!(!session.timestamp_end.is_empty());
    }

    // ===== CONNECTIVITY TESTS =====

    #[test]
    fn test_connectivity_creation() {
        let connectivity = Connectivity::new(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            -50.0,
            -60.0,
            100.0,
            45.0,
            "Point(-74.006 40.7128)".to_string(),
            "h14".to_string(),
            "h13".to_string(),
            "h12".to_string(),
            "h11".to_string()
        );

        assert_eq!(connectivity.session_id, 123);
        assert_eq!(connectivity.signal, -50.0);
        assert_eq!(connectivity.noise, -60.0);
        assert_eq!(connectivity.altitude, 100.0);
        assert_eq!(connectivity.heading, 45.0);
        assert!(connectivity.id.is_none());
    }

    #[test]
    fn test_connectivity_creation_with_wkt_validation() {
        // Valid WKT
        let connectivity = Connectivity::new_with_wkt_validation(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            -50.0,
            -60.0,
            100.0,
            45.0,
            "Point(-74.006 40.7128)".to_string(),
            "h14".to_string(),
            "h13".to_string(),
            "h12".to_string(),
            "h11".to_string()
        ).unwrap();

        assert_eq!(connectivity.session_id, 123);
        assert_eq!(connectivity.location, "Point(-74.006 40.7128)".to_string());

        // Invalid WKT should fail
        let result = Connectivity::new_with_wkt_validation(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            -50.0,
            -60.0,
            100.0,
            45.0,
            "Invalid WKT".to_string(),
            "h14".to_string(),
            "h13".to_string(),
            "h12".to_string(),
            "h11".to_string()
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_connectivity_with_id() {
        let connectivity = Connectivity::new(
            123,
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            -50.0,
            -60.0,
            100.0,
            45.0,
            "Point(-74.006 40.7128)".to_string(),
            "h14".to_string(),
            "h13".to_string(),
            "h12".to_string(),
            "h11".to_string()
        ).with_id(789);

        assert_eq!(connectivity.id, Some(789));
        assert_eq!(connectivity.session_id, 123);
    }
}
