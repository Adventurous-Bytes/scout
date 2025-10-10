use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};
use serde_json;

use chrono::{DateTime, Utc};

// ===== TRAITS =====
pub trait Syncable {
    fn id(&self) -> Option<i64>;
    fn set_id(&mut self, id: i64);
    fn id_local(&self) -> Option<String>;
    fn set_id_local(&mut self, id_local: String);
}

pub trait AncestorLocal {
    fn ancestor_id_local(&self) -> Option<String>;
    fn set_ancestor_id_local(&mut self, ancestor_id_local: String);
}

// ===== ENUMS =====

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseScoutStatus {
    Success,
    NotAuthorized,
    InvalidEvent,
    InvalidFile,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    TrailCamera,
    DroneFixedWing,
    DroneQuad,
    GpsTracker,
    SentryTower,
    SmartBuoy,
    RadioMeshBaseStation,
    RadioMeshRepeater,
    Unknown,
}

impl From<&str> for DeviceType {
    fn from(s: &str) -> Self {
        match s {
            "trail_camera" => DeviceType::TrailCamera,
            "drone_fixed_wing" => DeviceType::DroneFixedWing,
            "drone_quad" => DeviceType::DroneQuad,
            "gps_tracker" => DeviceType::GpsTracker,
            "sentry_tower" => DeviceType::SentryTower,
            "smart_buoy" => DeviceType::SmartBuoy,
            "radio_mesh_base_station" => DeviceType::RadioMeshBaseStation,
            "radio_mesh_repeater" => DeviceType::RadioMeshRepeater,
            _ => DeviceType::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaType {
    Image,
    Video,
    Audio,
    Text,
}

impl From<&str> for MediaType {
    fn from(s: &str) -> Self {
        match s {
            "image" => MediaType::Image,
            "video" => MediaType::Video,
            "audio" => MediaType::Audio,
            "text" => MediaType::Text,
            _ => MediaType::Image,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TagObservationType {
    Manual,
    Auto,
}

impl From<&str> for TagObservationType {
    fn from(s: &str) -> Self {
        match s {
            "manual" => TagObservationType::Manual,
            "auto" => TagObservationType::Auto,
            _ => TagObservationType::Auto,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanType {
    Mission,
    Fence,
    Rally,
    Markov,
}

impl From<&str> for PlanType {
    fn from(s: &str) -> Self {
        match s {
            "mission" => PlanType::Mission,
            "fence" => PlanType::Fence,
            "rally" => PlanType::Rally,
            "markov" => PlanType::Markov,
            _ => PlanType::Mission,
        }
    }
}

// ===== RESPONSE TYPES =====

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
#[native_model(id = 9, version = 1)]
#[native_db]
pub struct DevicePrettyLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    pub inserted_at: String,
    pub created_by: String,
    pub herd_id: i64,
    pub device_type: String,
    pub domain_name: Option<String>,
    pub location: Option<String>,
    pub altitude: Option<f64>,
    pub heading: Option<f64>,
    pub name: String,
    pub description: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

impl Default for DevicePrettyLocation {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: String::new(),
            created_by: String::new(),
            herd_id: 0,
            device_type: String::new(),
            domain_name: None,
            location: None,
            altitude: None,
            heading: None,
            name: String::new(),
            description: String::new(),
            latitude: None,
            longitude: None,
        }
    }
}

impl Syncable for DevicePrettyLocation {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 2, version = 1)]
#[native_db]
pub struct Device {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    pub inserted_at: String,
    pub created_by: String,
    pub herd_id: i64,
    pub device_type: DeviceType,
    pub name: String,
    pub description: String,
    pub domain_name: Option<String>,
    pub altitude: Option<f64>,
    pub heading: Option<f64>,
    pub location: Option<String>,
    pub video_publisher_token: Option<String>,
    pub video_subscriber_token: Option<String>,
}

impl Default for Device {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: String::new(),
            created_by: String::new(),
            herd_id: 0,
            device_type: DeviceType::Unknown,
            name: String::new(),
            description: String::new(),
            domain_name: None,
            altitude: None,
            heading: None,
            location: None,
            video_publisher_token: None,
            video_subscriber_token: None,
        }
    }
}

impl Syncable for Device {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 1, version = 1)]
#[native_db]
pub struct Herd {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
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

impl Default for Herd {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: String::new(),
            created_by: String::new(),
            is_public: false,
            slug: String::new(),
            description: String::new(),
            earthranger_domain: None,
            earthranger_token: None,
            video_publisher_token: None,
            video_subscriber_token: None,
            video_server_url: None,
        }
    }
}

impl Syncable for Herd {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 14, version = 1)]
#[native_db]
pub struct SessionLocal {
    pub id: Option<i64>,
    #[primary_key]
    pub id_local: Option<String>,
    pub device_id: i64,
    pub timestamp_start: String,
    pub timestamp_end: Option<String>,
    pub inserted_at: Option<String>,
    pub software_version: String,
    pub locations: Option<String>,
    pub altitude_max: f64,
    pub altitude_min: f64,
    pub altitude_average: f64,
    pub velocity_max: f64,
    pub velocity_min: f64,
    pub velocity_average: f64,
    pub distance_total: f64,
    pub distance_max_from_start: f64,
    pub earthranger_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub device_id: i64,
    pub timestamp_start: String,
    pub timestamp_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_at: Option<String>,
    pub software_version: String,
    pub locations: Option<String>,
    pub altitude_max: f64,
    pub altitude_min: f64,
    pub altitude_average: f64,
    pub velocity_max: f64,
    pub velocity_min: f64,
    pub velocity_average: f64,
    pub distance_total: f64,
    pub distance_max_from_start: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earthranger_url: Option<String>,
}

impl Default for SessionLocal {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            device_id: 0,
            timestamp_start: String::new(),
            timestamp_end: None,
            inserted_at: None,
            software_version: String::new(),
            locations: None,
            altitude_max: 0.0,
            altitude_min: 0.0,
            altitude_average: 0.0,
            velocity_max: 0.0,
            velocity_min: 0.0,
            velocity_average: 0.0,
            distance_total: 0.0,
            distance_max_from_start: 0.0,
            earthranger_url: None,
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self {
            id: None,
            device_id: 0,
            timestamp_start: String::new(),
            timestamp_end: None,
            inserted_at: None,
            software_version: String::new(),
            locations: None,
            altitude_max: 0.0,
            altitude_min: 0.0,
            altitude_average: 0.0,
            velocity_max: 0.0,
            velocity_min: 0.0,
            velocity_average: 0.0,
            distance_total: 0.0,
            distance_max_from_start: 0.0,
            earthranger_url: None,
        }
    }
}

impl Syncable for SessionLocal {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

impl Syncable for Session {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        None // API struct doesn't have id_local
    }

    fn set_id_local(&mut self, _id_local: String) {
        // API struct doesn't have id_local, so this is a no-op
    }
}

impl From<SessionLocal> for Session {
    fn from(local: SessionLocal) -> Self {
        Session {
            id: local.id,
            device_id: local.device_id,
            timestamp_start: local.timestamp_start,
            timestamp_end: local.timestamp_end,
            inserted_at: local.inserted_at,
            software_version: local.software_version,
            locations: local.locations,
            altitude_max: local.altitude_max,
            altitude_min: local.altitude_min,
            altitude_average: local.altitude_average,
            velocity_max: local.velocity_max,
            velocity_min: local.velocity_min,
            velocity_average: local.velocity_average,
            distance_total: local.distance_total,
            distance_max_from_start: local.distance_max_from_start,
            earthranger_url: local.earthranger_url,
        }
    }
}

impl From<Session> for SessionLocal {
    fn from(session: Session) -> Self {
        SessionLocal {
            id: session.id,
            id_local: None, // API structs don't have id_local
            device_id: session.device_id,
            timestamp_start: session.timestamp_start,
            timestamp_end: session.timestamp_end,
            inserted_at: session.inserted_at,
            software_version: session.software_version,
            locations: session.locations,
            altitude_max: session.altitude_max,
            altitude_min: session.altitude_min,
            altitude_average: session.altitude_average,
            velocity_max: session.velocity_max,
            velocity_min: session.velocity_min,
            velocity_average: session.velocity_average,
            distance_total: session.distance_total,
            distance_max_from_start: session.distance_max_from_start,
            earthranger_url: session.earthranger_url,
        }
    }
}

impl Session {
    pub fn new(
        device_id: i64,
        timestamp_start: u64,
        timestamp_end: Option<u64>,
        software_version: String,
        location: Option<String>,
        altitude_max: f64,
        altitude_min: f64,
        altitude_average: f64,
        velocity_max: f64,
        velocity_min: f64,
        velocity_average: f64,
        distance_total: f64,
        distance_max_from_start: f64,
    ) -> Self {
        use chrono::{DateTime, Utc};
        // Convert timestamp to string
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
            .to_rfc3339();

        let timestamp_end_str = timestamp_end.map(|t| {
            DateTime::from_timestamp(t as i64, 0)
                .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
                .to_rfc3339()
        });

        Self {
            id: None,
            device_id,
            timestamp_start: timestamp_start_str,
            timestamp_end: timestamp_end_str,
            inserted_at: None,
            software_version,
            locations: location,
            altitude_max,
            altitude_min,
            altitude_average,
            velocity_max,
            velocity_min,
            velocity_average,
            distance_total,
            distance_max_from_start,
            earthranger_url: None,
        }
    }

    pub fn update_timestamp_end(&mut self, timestamp_end: u64) {
        use chrono::{DateTime, Utc};
        self.timestamp_end = Some(
            DateTime::from_timestamp(timestamp_end as i64, 0)
                .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
                .to_rfc3339(),
        );
    }
}

impl SessionLocal {
    pub fn update_timestamp_end(&mut self, timestamp_end: u64) {
        use chrono::{DateTime, Utc};
        self.timestamp_end = Some(
            DateTime::from_timestamp(timestamp_end as i64, 0)
                .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
                .to_rfc3339(),
        );
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 4, version = 1)]
#[native_db]
pub struct Artifact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub file_path: String,
    #[secondary_key]
    pub session_id: Option<i64>,
    #[serde(skip)]
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_observation: Option<String>,
}

impl Default for Artifact {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            created_at: None,
            file_path: String::new(),
            session_id: None,
            ancestor_id_local: None,
            timestamp_observation: None,
        }
    }
}

impl Syncable for Artifact {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

impl AncestorLocal for Artifact {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
    }
}

impl Artifact {
    pub fn new(file_path: String, session_id: Option<i64>) -> Self {
        Self {
            id: None,
            id_local: None,
            created_at: None,
            file_path,
            session_id,
            ancestor_id_local: None,
            timestamp_observation: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 15, version = 1)]
#[native_db]
pub struct ConnectivityLocal {
    pub id: Option<i64>,
    #[primary_key]
    pub id_local: Option<String>,
    #[secondary_key]
    pub session_id: i64,
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    pub inserted_at: Option<String>,
    pub timestamp_start: String,
    pub signal: f64,
    pub noise: f64,
    pub altitude: f64,
    pub heading: f64,
    pub location: Option<String>,
    pub h14_index: String,
    pub h13_index: String,
    pub h12_index: String,
    pub h11_index: String,
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
    pub location: Option<String>,
    pub h14_index: String,
    pub h13_index: String,
    pub h12_index: String,
    pub h11_index: String,
}

impl Default for ConnectivityLocal {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            session_id: 0,
            ancestor_id_local: None,
            inserted_at: None,
            timestamp_start: String::new(),
            signal: 0.0,
            noise: 0.0,
            altitude: 0.0,
            heading: 0.0,
            location: None,
            h14_index: String::new(),
            h13_index: String::new(),
            h12_index: String::new(),
            h11_index: String::new(),
        }
    }
}

impl Default for Connectivity {
    fn default() -> Self {
        Self {
            id: None,
            session_id: 0,
            inserted_at: None,
            timestamp_start: String::new(),
            signal: 0.0,
            noise: 0.0,
            altitude: 0.0,
            heading: 0.0,
            location: None,
            h14_index: String::new(),
            h13_index: String::new(),
            h12_index: String::new(),
            h11_index: String::new(),
        }
    }
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
        h11_index: String,
    ) -> Self {
        use chrono::{DateTime, Utc};
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
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
            location: Some(location),
            h14_index,
            h13_index,
            h12_index,
            h11_index,
        }
    }
}

impl Syncable for ConnectivityLocal {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

impl Syncable for Connectivity {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        None // API struct doesn't have id_local
    }

    fn set_id_local(&mut self, _id_local: String) {
        // API struct doesn't have id_local, so this is a no-op
    }
}

impl AncestorLocal for ConnectivityLocal {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
    }
}

impl From<ConnectivityLocal> for Connectivity {
    fn from(local: ConnectivityLocal) -> Self {
        Connectivity {
            id: local.id,
            session_id: local.session_id,
            inserted_at: local.inserted_at,
            timestamp_start: local.timestamp_start,
            signal: local.signal,
            noise: local.noise,
            altitude: local.altitude,
            heading: local.heading,
            location: local.location,
            h14_index: local.h14_index,
            h13_index: local.h13_index,
            h12_index: local.h12_index,
            h11_index: local.h11_index,
        }
    }
}

impl From<Connectivity> for ConnectivityLocal {
    fn from(connectivity: Connectivity) -> Self {
        ConnectivityLocal {
            id: connectivity.id,
            id_local: None, // API structs don't have id_local
            session_id: connectivity.session_id,
            ancestor_id_local: None, // API structs don't have ancestor_id_local
            inserted_at: connectivity.inserted_at,
            timestamp_start: connectivity.timestamp_start,
            signal: connectivity.signal,
            noise: connectivity.noise,
            altitude: connectivity.altitude,
            heading: connectivity.heading,
            location: connectivity.location,
            h14_index: connectivity.h14_index,
            h13_index: connectivity.h13_index,
            h12_index: connectivity.h12_index,
            h11_index: connectivity.h11_index,
        }
    }
}

impl ConnectivityLocal {
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
        h11_index: String,
    ) -> Self {
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        Self {
            id: None,
            id_local: None,
            session_id,
            ancestor_id_local: None,
            inserted_at: None,
            timestamp_start: timestamp_start_str,
            signal,
            noise,
            altitude,
            heading,
            location: Some(location),
            h14_index,
            h13_index,
            h12_index,
            h11_index,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 16, version = 1)]
#[native_db]
pub struct EventLocal {
    pub id: Option<i64>,
    #[primary_key]
    pub id_local: Option<String>,
    pub message: Option<String>,
    pub media_url: Option<String>,
    pub file_path: Option<String>,
    pub location: Option<String>,
    pub altitude: f64,
    pub heading: f64,
    pub media_type: MediaType,
    pub device_id: i64,
    pub earthranger_url: Option<String>,
    pub timestamp_observation: String,
    pub is_public: bool,
    #[secondary_key]
    pub session_id: Option<i64>,
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub message: Option<String>,
    pub media_url: Option<String>,
    pub file_path: Option<String>,
    pub location: Option<String>,
    pub altitude: f64,
    pub heading: f64,
    pub media_type: MediaType,
    pub device_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earthranger_url: Option<String>,
    pub timestamp_observation: String,
    pub is_public: bool,
    pub session_id: Option<i64>,
}

impl Default for EventLocal {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            message: None,
            media_url: None,
            file_path: None,
            location: None,
            altitude: 0.0,
            heading: 0.0,
            media_type: MediaType::Image,
            device_id: 0,
            earthranger_url: None,
            timestamp_observation: String::new(),
            is_public: false,
            session_id: None,
            ancestor_id_local: None,
        }
    }
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: None,
            message: None,
            media_url: None,
            file_path: None,
            location: None,
            altitude: 0.0,
            heading: 0.0,
            media_type: MediaType::Image,
            device_id: 0,
            earthranger_url: None,
            timestamp_observation: String::new(),
            is_public: false,
            session_id: None,
        }
    }
}

impl Syncable for EventLocal {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

impl Syncable for Event {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        None // API struct doesn't have id_local
    }

    fn set_id_local(&mut self, _id_local: String) {
        // API struct doesn't have id_local, so this is a no-op
    }
}

impl AncestorLocal for EventLocal {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
    }
}

impl From<EventLocal> for Event {
    fn from(local: EventLocal) -> Self {
        Event {
            id: local.id,
            message: local.message,
            media_url: local.media_url,
            file_path: local.file_path,
            location: local.location,
            altitude: local.altitude,
            heading: local.heading,
            media_type: local.media_type,
            device_id: local.device_id,
            earthranger_url: local.earthranger_url,
            timestamp_observation: local.timestamp_observation,
            is_public: local.is_public,
            session_id: local.session_id,
        }
    }
}

impl From<Event> for EventLocal {
    fn from(event: Event) -> Self {
        EventLocal {
            id: event.id,
            id_local: None, // API structs don't have id_local
            message: event.message,
            media_url: event.media_url,
            file_path: event.file_path,
            location: event.location,
            altitude: event.altitude,
            heading: event.heading,
            media_type: event.media_type,
            device_id: event.device_id,
            earthranger_url: event.earthranger_url,
            timestamp_observation: event.timestamp_observation,
            is_public: event.is_public,
            session_id: event.session_id,
            ancestor_id_local: None, // API structs don't have ancestor_id_local
        }
    }
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
        media_type: MediaType,
        device_id: i64,
        timestamp_observation: u64,
        is_public: bool,
        session_id: Option<i64>,
    ) -> Self {
        use chrono::{DateTime, Utc};
        let timestamp_observation_str = DateTime::from_timestamp(timestamp_observation as i64, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
            .to_rfc3339();

        Self {
            id: None,
            message,
            media_url,
            file_path,
            location: Some(Self::format_location(latitude, longitude)),
            altitude,
            heading,
            media_type,
            device_id,
            earthranger_url,
            timestamp_observation: timestamp_observation_str,
            is_public,
            session_id,
        }
    }

    pub fn format_location(latitude: f64, longitude: f64) -> String {
        format!("POINT({} {})", longitude, latitude)
    }
}

impl EventLocal {
    pub fn new(
        message: Option<String>,
        media_url: Option<String>,
        file_path: Option<String>,
        earthranger_url: Option<String>,
        latitude: f64,
        longitude: f64,
        altitude: f64,
        heading: f64,
        media_type: MediaType,
        device_id: i64,
        timestamp_observation: u64,
        is_public: bool,
        session_id: Option<i64>,
    ) -> Self {
        let location = Self::format_location(latitude, longitude);
        let timestamp_observation = DateTime::from_timestamp(timestamp_observation as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        Self {
            id: None,
            id_local: None,
            message,
            media_url,
            file_path,
            location: Some(location),
            altitude,
            heading,
            media_type,
            device_id,
            earthranger_url,
            timestamp_observation,
            is_public,
            session_id,
            ancestor_id_local: None,
        }
    }

    pub fn format_location(latitude: f64, longitude: f64) -> String {
        format!("POINT({} {})", longitude, latitude)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 17, version = 1)]
#[native_db]
pub struct TagLocal {
    pub id: Option<i64>,
    #[primary_key]
    pub id_local: Option<String>,
    pub inserted_at: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub conf: f64,
    pub observation_type: TagObservationType,
    pub class_name: String,
    #[secondary_key]
    pub event_id: i64,
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_at: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub conf: f64,
    pub observation_type: TagObservationType,
    pub class_name: String,
    pub event_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl Default for TagLocal {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            conf: 0.0,
            observation_type: TagObservationType::Auto,
            class_name: String::new(),
            event_id: 0,
            ancestor_id_local: None,
            location: None,
        }
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            id: None,
            inserted_at: None,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            conf: 0.0,
            observation_type: TagObservationType::Manual,
            class_name: String::new(),
            event_id: 0,
            location: None,
        }
    }
}

impl Syncable for TagLocal {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

impl Syncable for Tag {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        None // API struct doesn't have id_local
    }

    fn set_id_local(&mut self, _id_local: String) {
        // API struct doesn't have id_local, so this is a no-op
    }
}

impl AncestorLocal for TagLocal {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
    }
}

impl From<TagLocal> for Tag {
    fn from(local: TagLocal) -> Self {
        Tag {
            id: local.id,
            inserted_at: local.inserted_at,
            x: local.x,
            y: local.y,
            width: local.width,
            height: local.height,
            conf: local.conf,
            observation_type: local.observation_type,
            class_name: local.class_name,
            event_id: local.event_id,
            location: local.location,
        }
    }
}

impl From<Tag> for TagLocal {
    fn from(tag: Tag) -> Self {
        TagLocal {
            id: tag.id,
            id_local: None, // API structs don't have id_local
            inserted_at: tag.inserted_at,
            x: tag.x,
            y: tag.y,
            width: tag.width,
            height: tag.height,
            conf: tag.conf,
            observation_type: tag.observation_type,
            class_name: tag.class_name,
            event_id: tag.event_id,
            ancestor_id_local: None, // API structs don't have ancestor_id_local
            location: tag.location,
        }
    }
}

impl Tag {
    pub fn new(
        _class_id: i64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        conf: f64,
        observation_type: TagObservationType,
        class_name: String,
    ) -> Self {
        Self {
            id: None,
            inserted_at: None,
            x,
            y,
            width,
            height,
            conf,
            observation_type,
            class_name,
            event_id: 0,
            location: None,
        }
    }

    pub fn new_with_location(
        _class_id: i64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        conf: f64,
        observation_type: TagObservationType,
        class_name: String,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        let mut tag = Self::new(
            _class_id,
            x,
            y,
            width,
            height,
            conf,
            observation_type,
            class_name,
        );
        tag.set_location(latitude, longitude);
        tag
    }

    pub fn update_event_id(&mut self, event_id: i64) {
        self.event_id = event_id;
    }

    pub fn set_location(&mut self, latitude: f64, longitude: f64) {
        self.location = Some(Self::format_location(latitude, longitude));
    }

    pub fn clear_location(&mut self) {
        self.location = None;
    }

    pub fn format_location(latitude: f64, longitude: f64) -> String {
        format!("POINT({} {})", longitude, latitude)
    }

    pub fn parse_location(location: &str) -> Option<(f64, f64)> {
        if let Some(coords) = location
            .strip_prefix("POINT(")
            .and_then(|s| s.strip_suffix(")"))
        {
            let parts: Vec<&str> = coords.split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(lon), Ok(lat)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    return Some((lat, lon));
                }
            }
        }
        None
    }

    pub fn get_coordinates(&self) -> Option<(f64, f64)> {
        self.location
            .as_ref()
            .and_then(|loc| Self::parse_location(loc))
    }
}

impl TagLocal {
    pub fn new(
        _class_id: i64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        conf: f64,
        observation_type: TagObservationType,
        class_name: String,
    ) -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: None,
            x,
            y,
            width,
            height,
            conf,
            observation_type,
            class_name,
            event_id: 0,
            ancestor_id_local: None,
            location: None,
        }
    }

    pub fn new_with_location(
        _class_id: i64,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        conf: f64,
        observation_type: TagObservationType,
        class_name: String,
        latitude: f64,
        longitude: f64,
    ) -> Self {
        let mut tag = Self::new(
            _class_id,
            x,
            y,
            width,
            height,
            conf,
            observation_type,
            class_name,
        );
        tag.set_location(latitude, longitude);
        tag
    }

    pub fn update_event_id(&mut self, event_id: i64) {
        self.event_id = event_id;
    }

    pub fn update_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
    }

    pub fn set_location(&mut self, latitude: f64, longitude: f64) {
        self.location = Some(Self::format_location(latitude, longitude));
    }

    pub fn clear_location(&mut self) {
        self.location = None;
    }

    pub fn format_location(latitude: f64, longitude: f64) -> String {
        format!("POINT({} {})", longitude, latitude)
    }

    pub fn parse_location(location: &str) -> Option<(f64, f64)> {
        if let Some(coords) = location
            .strip_prefix("POINT(")
            .and_then(|s| s.strip_suffix(")"))
        {
            let parts: Vec<&str> = coords.split_whitespace().collect();
            if parts.len() == 2 {
                if let (Ok(lon), Ok(lat)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                    return Some((lat, lon));
                }
            }
        }
        None
    }

    pub fn get_coordinates(&self) -> Option<(f64, f64)> {
        self.location
            .as_ref()
            .and_then(|loc| Self::parse_location(loc))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 8, version = 1)]
#[native_db]
pub struct Plan {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    pub inserted_at: Option<String>,
    pub name: String,
    pub instructions: String,
    pub herd_id: i64,
    pub plan_type: PlanType,
}

impl Default for Plan {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: None,
            name: String::new(),
            instructions: String::new(),
            herd_id: 0,
            plan_type: PlanType::Mission,
        }
    }
}

impl Syncable for Plan {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

/// Plan structure for database operations (ID field is optional for insertion)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 10, version = 1)]
#[native_db]
pub struct PlanInsert {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_at: Option<String>,
    pub name: String,
    pub instructions: String,
    pub herd_id: i64,
    pub plan_type: PlanType,
}

impl Default for PlanInsert {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: None,
            name: String::new(),
            instructions: String::new(),
            herd_id: 0,
            plan_type: PlanType::Mission,
        }
    }
}

impl Syncable for PlanInsert {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 11, version = 1)]
#[native_db]
pub struct Layer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub features: serde_json::Value,
    pub herd_id: i64,
}

impl Default for Layer {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            created_at: None,
            features: serde_json::Value::Null,
            herd_id: 0,
        }
    }
}

impl Syncable for Layer {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

impl Layer {
    pub fn new(features: serde_json::Value, herd_id: i64) -> Self {
        Self {
            id: None,
            id_local: None,
            created_at: None,
            features,
            herd_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 12, version = 1)]
#[native_db]
pub struct Zone {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_at: Option<String>,
    pub region: String,
    pub herd_id: i64,
}

impl Default for Zone {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: None,
            region: String::new(),
            herd_id: 0,
        }
    }
}

impl Syncable for Zone {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 13, version = 1)]
#[native_db]
pub struct Action {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_at: Option<String>,
    pub zone_id: i64,
    pub trigger: Vec<String>,
    pub opcode: i32,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            inserted_at: None,
            zone_id: 0,
            trigger: Vec::new(),
            opcode: 0,
        }
    }
}

impl Syncable for Action {
    fn id(&self) -> Option<i64> {
        self.id
    }

    fn set_id(&mut self, id: i64) {
        self.id = Some(id);
    }

    fn id_local(&self) -> Option<String> {
        self.id_local.clone()
    }

    fn set_id_local(&mut self, id_local: String) {
        self.id_local = Some(id_local);
    }
}
