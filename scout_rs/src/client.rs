use anyhow::{ Result, anyhow };
use serde::{ Deserialize, Serialize };
use tracing::info;
use chrono::{ DateTime, Utc };

use crate::db_client::{ ScoutDbClient, DatabaseConfig };

// ===== RESPONSE TYPES (for backward compatibility) =====

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

// ===== DATA STRUCTURES =====

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub device_id: i64,
    pub timestamp_start: String,
    pub timestamp_end: String,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_paths: Option<Vec<String>>, // text[] in DB
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earthranger_url: Option<String>, // text in DB
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connectivity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub session_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
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

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: Option<i64>,
    pub inserted_at: Option<String>,
    pub name: String,
    pub instructions: String,
    pub herd_id: i64,
    pub plan_type: String,
}

// ===== CLIENT IMPLEMENTATION =====

#[derive(Debug)]
pub struct ScoutClient {
    pub scout_url: String,
    pub api_key: String,
    pub device: Option<Device>,
    pub herd: Option<Herd>,
    db_client: Option<ScoutDbClient>,
}

impl ScoutClient {
    /// Creates a new ScoutClient instance.
    pub fn new(scout_url: String, api_key: String) -> Result<Self> {
        Ok(Self {
            scout_url,
            api_key,
            device: None,
            herd: None,
            db_client: None,
        })
    }

    /// Identifies the device and herd, then establishes direct database connection
    pub async fn identify(&mut self) -> Result<()> {
        info!("🔍 Identifying device and herd from database...");

        // First establish database connection using environment variables
        let db_config = DatabaseConfig::from_env()?;
        let mut db_client = ScoutDbClient::new(db_config);
        db_client.connect()?;

        self.db_client = Some(db_client);
        info!("✅ Database connection established");

        // Get device information directly from database using API key
        let device = self.get_device_from_db().await?;
        info!("   Device ID: {}", device.id);
        info!("   Device Name: {}", device.name);
        info!("   Herd ID: {}", device.herd_id);

        // Get herd information directly from database
        let herd = self.get_herd_from_db(device.herd_id).await?;
        info!("   Herd Slug: {}", herd.slug);
        info!("   Herd Description: {}", herd.description);

        // Store device and herd info
        self.device = Some(device);
        self.herd = Some(herd);

        info!("✅ Identification and connection complete");
        Ok(())
    }

    /// Gets device information directly from database (authorization handled by database)
    async fn get_device_from_db(&mut self) -> Result<Device> {
        let db_client = self.get_db_client()?;

        // Database authorization will ensure we only get the device we're authorized to see
        let results = db_client.query("devices", |client| {
            client.from("devices").select("*").limit(1)
        }).await?;

        if results.is_empty() {
            return Err(anyhow!("No device found - check authorization"));
        }

        Ok(results.into_iter().next().unwrap())
    }

    /// Gets herd information directly from database
    async fn get_herd_from_db(&mut self, herd_id: u32) -> Result<Herd> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("herds", |client| {
            client.from("herds").select("*").eq("id", herd_id.to_string()).limit(1)
        }).await?;

        if results.is_empty() {
            return Err(anyhow!("No herd found for ID: {}", herd_id));
        }

        Ok(results.into_iter().next().unwrap())
    }

    /// Gets the database client, ensuring it's available
    fn get_db_client(&mut self) -> Result<&mut ScoutDbClient> {
        self.db_client
            .as_mut()
            .ok_or_else(|| anyhow!("Database client not initialized. Call identify() first."))
    }

    /// Checks if the client has been identified and has a database connection
    pub fn is_identified(&self) -> bool {
        self.db_client.is_some() && self.device.is_some() && self.herd.is_some()
    }

    // ===== HELPER METHODS =====

    /// Helper to create a success response
    fn success_response<T>(data: T) -> ResponseScout<T> {
        ResponseScout::new(ResponseScoutStatus::Success, Some(data))
    }

    /// Helper to create a failure response
    fn failure_response<T>() -> ResponseScout<T> {
        ResponseScout::new(ResponseScoutStatus::Failure, None)
    }

    /// Helper to handle database insert results
    fn handle_insert_result<T>(result: Vec<T>) -> Result<ResponseScout<T>> {
        if result.is_empty() {
            Ok(Self::failure_response())
        } else {
            Ok(Self::success_response(result.into_iter().next().unwrap()))
        }
    }

    /// Helper to handle database query results
    fn handle_query_result<T>(result: Vec<T>) -> ResponseScout<Vec<T>> {
        Self::success_response(result)
    }

    // ===== BACKWARD COMPATIBILITY METHODS =====

    /// Gets device information (backward compatibility method)
    pub async fn get_device(&mut self) -> Result<ResponseScout<Device>> {
        // Return cached device if available
        if let Some(device) = &self.device {
            return Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(device.clone())));
        }

        // If not identified yet, try to identify
        self.identify().await?;

        if let Some(device) = &self.device {
            Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(device.clone())))
        } else {
            Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
        }
    }

    /// Gets herd information (backward compatibility method)
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
                return Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(herd.clone())));
            }
        }

        // If not identified yet, try to identify
        if self.device.is_none() {
            self.identify().await?;
        }

        if let Some(herd) = &self.herd {
            if herd.id == herd_id {
                Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(herd.clone())))
            } else {
                Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
            }
        } else {
            Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
        }
    }

    // ===== DIRECT DATABASE OPERATIONS =====

    /// Creates an event directly in the database
    pub async fn create_event(&mut self, event: &Event) -> Result<ResponseScout<Event>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("events", event).await?;
        Self::handle_insert_result(result)
    }

    /// Creates tags for an event directly in the database
    pub async fn create_tags(
        &mut self,
        event_id: i64,
        tags: &[Tag]
    ) -> Result<ResponseScout<Vec<Tag>>> {
        let db_client = self.get_db_client()?;

        let mut created_tags = Vec::new();

        for tag in tags {
            let mut tag_with_event_id = tag.clone();
            tag_with_event_id.update_event_id(event_id as u32);

            let result = db_client.insert("tags", &tag_with_event_id).await?;
            if !result.is_empty() {
                created_tags.push(result.into_iter().next().unwrap());
            }
        }

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_tags)))
    }

    /// Creates an event with tags (compatibility method)
    pub async fn create_event_with_tags(
        &mut self,
        event: &Event,
        tags: &[Tag],
        _file_path: Option<&str>
    ) -> Result<ResponseScout<Event>> {
        // Create event first
        let event_response = self.create_event(event).await?;

        if event_response.status != ResponseScoutStatus::Success {
            return Ok(event_response);
        }

        let created_event = event_response.data.unwrap();

        // Then create tags
        if !tags.is_empty() {
            let tags_response = self.create_tags(created_event.id.unwrap(), tags).await?;
            if tags_response.status != ResponseScoutStatus::Success {
                return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
            }
        }

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_event)))
    }

    /// Creates a session directly in the database
    pub async fn create_session(&mut self, session: &Session) -> Result<ResponseScout<Session>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("sessions", session).await?;
        Self::handle_insert_result(result)
    }

    /// Creates connectivity data directly from the database
    pub async fn create_connectivity(
        &mut self,
        connectivity: &Connectivity
    ) -> Result<ResponseScout<Connectivity>> {
        let db_client = self.get_db_client()?;
        let result = db_client.insert("connectivity", connectivity).await?;
        Self::handle_insert_result(result)
    }

    /// Gets sessions for a herd directly from the database
    pub async fn get_sessions_by_herd(
        &mut self,
        herd_id: u32
    ) -> Result<ResponseScout<Vec<Session>>> {
        let db_client = self.get_db_client()?;
        let results = db_client.query("sessions", |client| {
            client
                .from("sessions")
                .select("*, devices!inner(herd_id)")
                .eq("devices.herd_id", herd_id.to_string())
                .order("timestamp_start.desc")
        }).await?;
        Ok(Self::handle_query_result(results))
    }

    /// Gets plans for a herd directly from the database
    pub async fn get_plans_by_herd(&mut self, herd_id: u32) -> Result<ResponseScout<Vec<Plan>>> {
        let db_client = self.get_db_client()?;
        let results = db_client.query("plans", |client| {
            client.from("plans").eq("herd_id", herd_id.to_string()).order("inserted_at.desc")
        }).await?;
        Ok(Self::handle_query_result(results))
    }

    /// Gets events for a session directly from the database
    pub async fn get_session_events(
        &mut self,
        session_id: i64
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;
        let results = db_client.query("events", |client| {
            client
                .from("events")
                .eq("session_id", session_id.to_string())
                .order("timestamp_observation.desc")
        }).await?;
        Ok(Self::handle_query_result(results))
    }

    /// Gets connectivity data for a session directly from the database
    pub async fn get_session_connectivity(
        &mut self,
        session_id: i64
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        let db_client = self.get_db_client()?;
        let results = db_client.query("connectivity", |client| {
            client
                .from("connectivity")
                .eq("session_id", session_id.to_string())
                .order("timestamp_start.asc")
        }).await?;
        Ok(Self::handle_query_result(results))
    }

    /// Updates a session directly in the database
    pub async fn update_session(
        &mut self,
        session_id: i64,
        session: &Session
    ) -> Result<ResponseScout<Session>> {
        let db_client = self.get_db_client()?;

        let result = db_client.update("sessions", session, |client| {
            client.from("sessions").eq("id", session_id.to_string())
        }).await?;

        if result.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let updated_session = result.into_iter().next().unwrap();
        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(updated_session)))
    }

    /// Deletes a session directly from the database
    pub async fn delete_session(&mut self, session_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        // Delete connectivity entries first
        db_client.delete("connectivity", |client| {
            client.from("connectivity").eq("session_id", session_id.to_string())
        }).await?;

        // Delete events and their tags
        let event_ids: Vec<i64> = db_client.query("events", |client| {
            client.from("events").select("id").eq("session_id", session_id.to_string())
        }).await?;

        for event_id in event_ids {
            db_client.delete("tags", |client| {
                client.from("tags").eq("event_id", event_id.to_string())
            }).await?;
        }

        db_client.delete("events", |client| {
            client.from("events").eq("session_id", session_id.to_string())
        }).await?;

        // Finally delete the session
        db_client.delete("sessions", |client| {
            client.from("sessions").eq("id", session_id.to_string())
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    /// Deletes an event directly from the database
    pub async fn delete_event(&mut self, event_id: i64) -> Result<ResponseScout<()>> {
        let db_client = self.get_db_client()?;

        // Delete tags first
        db_client.delete("tags", |client| {
            client.from("tags").eq("event_id", event_id.to_string())
        }).await?;

        // Delete the event
        db_client.delete("events", |client| {
            client.from("events").eq("id", event_id.to_string())
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
    }

    // ===== ADDITIONAL OPERATIONS =====

    /// Gets all devices for a herd directly from the database
    pub async fn get_devices_by_herd(
        &mut self,
        herd_id: u32
    ) -> Result<ResponseScout<Vec<Device>>> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("devices", |client| {
            client.from("devices").eq("herd_id", herd_id.to_string()).order("inserted_at.desc")
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(results)))
    }

    /// Gets a specific device by ID directly from the database
    pub async fn get_device_by_id(&mut self, device_id: u32) -> Result<ResponseScout<Device>> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("devices", |client| {
            client.from("devices").select("*").eq("id", device_id.to_string()).limit(1)
        }).await?;

        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let device = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(device)))
    }

    /// Gets a specific herd by ID directly from the database
    pub async fn get_herd_by_id(&mut self, herd_id: u32) -> Result<ResponseScout<Herd>> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("herds", |client| {
            client.from("herds").select("*").eq("id", herd_id.to_string()).limit(1)
        }).await?;

        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let herd = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(herd)))
    }

    /// Gets all events for a device directly from the database
    pub async fn get_device_events(&mut self, device_id: u32) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("events", |client| {
            client
                .from("events")
                .eq("device_id", device_id.to_string())
                .order("timestamp_observation.desc")
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(results)))
    }

    /// Gets events with tags for a device directly from the database
    pub async fn get_device_events_with_tags(
        &mut self,
        device_id: u32
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("events", |client| {
            client
                .from("events")
                .select("*, tags(*)")
                .eq("device_id", device_id.to_string())
                .order("timestamp_observation.desc")
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(results)))
    }

    /// Gets events within a time range directly from the database
    pub async fn get_events_in_timerange(
        &mut self,
        start_time: &str,
        end_time: &str
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("events", |client| {
            client
                .from("events")
                .gte("timestamp_observation", start_time)
                .lte("timestamp_observation", end_time)
                .order("timestamp_observation.desc")
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(results)))
    }

    /// Gets events within a geographic area directly from the database
    pub async fn get_events_in_area(
        &mut self,
        min_lat: f64,
        max_lat: f64,
        min_lon: f64,
        max_lon: f64
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        // This would require PostGIS functions, but for now we'll use a simple bounding box approach
        // In a real implementation, you'd use PostGIS ST_Contains or similar functions
        let results = db_client.query("events", |client| {
            client
                .from("events")
                .select("*")
                .gte("latitude", min_lat.to_string())
                .lte("latitude", max_lat.to_string())
                .gte("longitude", min_lon.to_string())
                .lte("longitude", max_lon.to_string())
                .order("timestamp_observation.desc")
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(results)))
    }

    /// Creates multiple events in a batch directly in the database
    pub async fn create_events_batch(
        &mut self,
        events: &[Event]
    ) -> Result<ResponseScout<Vec<Event>>> {
        let db_client = self.get_db_client()?;

        let mut created_events = Vec::new();

        for event in events {
            let result = db_client.insert("events", event).await?;
            if !result.is_empty() {
                created_events.push(result.into_iter().next().unwrap());
            }
        }

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_events)))
    }

    /// Creates multiple sessions in a batch directly in the database
    pub async fn create_sessions_batch(
        &mut self,
        sessions: &[Session]
    ) -> Result<ResponseScout<Vec<Session>>> {
        let db_client = self.get_db_client()?;

        let mut created_sessions = Vec::new();

        for session in sessions {
            let result = db_client.insert("sessions", session).await?;
            if !result.is_empty() {
                created_sessions.push(result.into_iter().next().unwrap());
            }
        }

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_sessions)))
    }

    /// Creates multiple connectivity entries in a batch directly in the database
    pub async fn create_connectivity_batch(
        &mut self,
        connectivity_entries: &[Connectivity]
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        let db_client = self.get_db_client()?;

        let mut created_entries = Vec::new();

        for entry in connectivity_entries {
            let result = db_client.insert("connectivity", entry).await?;
            if !result.is_empty() {
                created_entries.push(result.into_iter().next().unwrap());
            }
        }

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_entries)))
    }

    /// Updates an event directly in the database
    pub async fn update_event(
        &mut self,
        event_id: i64,
        event: &Event
    ) -> Result<ResponseScout<Event>> {
        let db_client = self.get_db_client()?;

        let result = db_client.update("events", event, |client| {
            client.from("events").eq("id", event_id.to_string())
        }).await?;

        if result.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let updated_event = result.into_iter().next().unwrap();
        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(updated_event)))
    }

    /// Updates connectivity data directly in the database
    pub async fn update_connectivity(
        &mut self,
        connectivity_id: i64,
        connectivity: &Connectivity
    ) -> Result<ResponseScout<Connectivity>> {
        let db_client = self.get_db_client()?;

        let result = db_client.update("connectivity", connectivity, |client| {
            client.from("connectivity").eq("id", connectivity_id.to_string())
        }).await?;

        if result.is_empty() {
            return Err(anyhow!("Failed to update connectivity entry - no data returned"));
        }

        Ok(
            ResponseScout::new(
                ResponseScoutStatus::Success,
                Some(result.into_iter().next().unwrap())
            )
        )
    }

    /// Gets connectivity data with coordinates directly from the database
    pub async fn get_connectivity_with_coordinates(
        &mut self,
        session_id: i64
    ) -> Result<ResponseScout<Vec<Connectivity>>> {
        let db_client = self.get_db_client()?;

        // This would typically use a database function or view that extracts coordinates
        // For now, we'll return the basic connectivity data
        let results = db_client.query("connectivity", |client| {
            client
                .from("connectivity")
                .eq("session_id", session_id.to_string())
                .order("timestamp_start.asc")
        }).await?;

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(results)))
    }

    /// Ends a session by updating its timestamp_end directly in the database
    pub async fn end_session(
        &mut self,
        session_id: i64,
        timestamp_end: u64
    ) -> Result<ResponseScout<()>> {
        let mut session = Session::new(
            0, // device_id - will be ignored in update
            timestamp_end,
            timestamp_end,
            "".to_string(), // software_version - will be ignored in update
            None, // locations - will be ignored in update
            0.0,
            0.0,
            0.0, // altitude values - will be ignored in update
            0.0,
            0.0,
            0.0, // velocity values - will be ignored in update
            0.0,
            0.0 // distance values - will be ignored in update
        );

        // Convert timestamp to string format
        session.timestamp_end = chrono::DateTime
            ::from_timestamp(timestamp_end as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now())
            .to_rfc3339();

        let response = self.update_session(session_id, &session).await?;
        if response.status == ResponseScoutStatus::Success {
            Ok(ResponseScout::new(ResponseScoutStatus::Success, None))
        } else {
            Ok(ResponseScout::new(ResponseScoutStatus::Failure, None))
        }
    }

    /// Gets statistics for a session directly from the database
    pub async fn get_session_statistics(
        &mut self,
        session_id: i64
    ) -> Result<ResponseScout<Session>> {
        let db_client = self.get_db_client()?;

        let results = db_client.query("sessions", |client| {
            client.from("sessions").select("*").eq("id", session_id.to_string()).limit(1)
        }).await?;

        if results.is_empty() {
            return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
        }

        let session = results.into_iter().next().unwrap();
        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(session)))
    }

    // ===== COMPATIBILITY METHODS =====

    /// Compatibility method for upsert_session
    pub async fn upsert_session(&mut self, session: &Session) -> Result<ResponseScout<Session>> {
        // For now, always create a new session
        // In the future, this could check if a session exists and update it
        self.create_session(session).await
    }

    /// Compatibility method for upsert_connectivity
    pub async fn upsert_connectivity(
        &mut self,
        connectivity: &Connectivity
    ) -> Result<ResponseScout<Connectivity>> {
        // For now, always create a new connectivity entry
        // In the future, this could check if an entry exists and update it
        self.create_connectivity(connectivity).await
    }

    /// Compatibility method for post_events_batch
    pub async fn post_events_batch(
        &mut self,
        events_and_files: &[(Event, Vec<Tag>, String)],
        _batch_size: usize
    ) -> Result<ResponseScout<Vec<Event>>> {
        let mut created_events = Vec::new();

        for (event, tags, _file_path) in events_and_files {
            let event_response = self.create_event(event).await?;
            if event_response.status != ResponseScoutStatus::Success {
                return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
            }

            let created_event = event_response.data.unwrap();
            if !tags.is_empty() {
                let tags_response = self.create_tags(created_event.id.unwrap(), tags).await?;
                if tags_response.status != ResponseScoutStatus::Success {
                    return Ok(ResponseScout::new(ResponseScoutStatus::Failure, None));
                }
            }
            created_events.push(created_event);
        }

        Ok(ResponseScout::new(ResponseScoutStatus::Success, Some(created_events)))
    }
}
