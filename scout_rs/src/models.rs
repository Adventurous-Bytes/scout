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
#[native_model(id = 3, version = 1)]
#[native_db]
pub struct Session {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
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

impl Syncable for Session {
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

impl Session {
    pub fn new(
        device_id: i64,
        timestamp_start: u64,
        timestamp_end: Option<u64>,
        software_version: String,
        locations_wkt: Option<String>,
        altitude_max: f64,
        altitude_min: f64,
        altitude_average: f64,
        velocity_max: f64,
        velocity_min: f64,
        velocity_average: f64,
        distance_total: f64,
        distance_max_from_start: f64,
    ) -> Self {
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| Utc::now())
            .to_rfc3339();

        let timestamp_end_str = timestamp_end.map(|t| {
            DateTime::from_timestamp(t as i64, 0)
                .unwrap_or_else(|| Utc::now())
                .to_rfc3339()
        });

        let locations = locations_wkt.unwrap_or_else(|| "POINT(0 0)".to_string());

        Self {
            id: None,
            id_local: None,
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
            earthranger_url: None,
        }
    }

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn update_timestamp_end(&mut self, timestamp_end: u64) {
        self.timestamp_end = Some(
            DateTime::from_timestamp(timestamp_end as i64, 0)
                .unwrap_or_else(|| Utc::now())
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

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 5, version = 1)]
#[native_db]
pub struct Connectivity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[secondary_key]
    pub session_id: i64,
    #[serde(skip)]
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
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

impl Syncable for Connectivity {
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

impl AncestorLocal for Connectivity {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
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
#[native_model(id = 6, version = 1)]
#[native_db]
pub struct Event {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
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
    #[serde(skip)]
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
}

impl Syncable for Event {
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

impl AncestorLocal for Event {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
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

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 7, version = 1)]
#[native_db]
pub struct Tag {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    #[serde(skip)]
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

impl Syncable for Tag {
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

impl AncestorLocal for Tag {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
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
            location: Some(Self::format_location(latitude, longitude)),
        }
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
    pub created_at: Option<String>,
    pub features: serde_json::Value,
    pub herd_id: i64,
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

    pub fn with_id(mut self, id: i64) -> Self {
        self.id = Some(id);
        self
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
    pub inserted_at: String,
    pub region: String,
    pub herd_id: i64,
    pub actions: Option<Vec<Action>>,
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
    pub inserted_at: String,
    pub zone_id: i64,
    pub trigger: Vec<String>,
    pub opcode: i32,
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
