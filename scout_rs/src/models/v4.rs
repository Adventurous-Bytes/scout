use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// Re-export all unchanged models from v3
pub use super::v3::{Artifact, ArtifactLocal};

// Re-export all unchanged models from v2
pub use super::v2::{Operator, OperatorLocal};

// Re-export all unchanged models from v1
pub use super::v1::{
    Action, AncestorLocal, Device, DevicePrettyLocation, DeviceType, Event, EventLocal, Heartbeat,
    Herd, Layer, MediaType, Plan, PlanInsert, PlanType, ResponseScout, ResponseScoutStatus,
    Session, SessionLocal, Syncable, Tag, TagLocal, TagObservationType, Zone,
};

// ===== CONNECTIVITY V4 WITH OPTIONAL MODE FIELD =====
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 15, version = 4)]
#[native_db]
pub struct ConnectivityLocal {
    pub id: Option<i64>,
    #[primary_key]
    pub id_local: Option<String>,
    #[secondary_key]
    pub session_id: Option<i64>,
    #[secondary_key]
    pub device_id: Option<i64>,
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
    // FIELDS FROM V2
    pub battery_percentage: Option<f32>,
    // FIELDS FROM V3
    pub frequency_hz: Option<f32>,
    pub bandwidth_hz: Option<f32>,
    pub associated_station: Option<String>,
    // NEW FIELD IN V4
    pub mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connectivity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<i64>,
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
    // FIELDS FROM V2
    #[serde(skip_serializing_if = "Option::is_none")]
    pub battery_percentage: Option<f32>,
    // FIELDS FROM V3
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_hz: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_hz: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub associated_station: Option<String>,
    // NEW FIELD IN V4
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

impl Default for ConnectivityLocal {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            session_id: None,
            device_id: None,
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
            battery_percentage: None,
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
            mode: None,
        }
    }
}

impl super::v1::Syncable for ConnectivityLocal {
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

impl super::v1::AncestorLocal for ConnectivityLocal {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
    }
}

impl From<ConnectivityLocal> for Connectivity {
    fn from(local: ConnectivityLocal) -> Self {
        Self {
            id: local.id,
            session_id: local.session_id,
            device_id: local.device_id,
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
            battery_percentage: local.battery_percentage,
            frequency_hz: local.frequency_hz,
            bandwidth_hz: local.bandwidth_hz,
            associated_station: local.associated_station,
            mode: local.mode,
        }
    }
}

impl From<Connectivity> for ConnectivityLocal {
    fn from(remote: Connectivity) -> Self {
        Self {
            id: remote.id,
            id_local: None,
            session_id: remote.session_id,
            device_id: remote.device_id,
            ancestor_id_local: None,
            inserted_at: remote.inserted_at,
            timestamp_start: remote.timestamp_start,
            signal: remote.signal,
            noise: remote.noise,
            altitude: remote.altitude,
            heading: remote.heading,
            location: remote.location,
            h14_index: remote.h14_index,
            h13_index: remote.h13_index,
            h12_index: remote.h12_index,
            h11_index: remote.h11_index,
            battery_percentage: remote.battery_percentage,
            frequency_hz: remote.frequency_hz,
            bandwidth_hz: remote.bandwidth_hz,
            associated_station: remote.associated_station,
            mode: remote.mode,
        }
    }
}

impl Connectivity {
    pub fn new(
        session_id: Option<i64>,
        device_id: Option<i64>,
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
        battery_percentage: Option<f32>,
        frequency_hz: Option<f32>,
        bandwidth_hz: Option<f32>,
        associated_station: Option<String>,
        mode: Option<String>,
    ) -> Self {
        use chrono::{DateTime, Utc};
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
            .to_rfc3339();

        Self {
            id: None,
            session_id,
            device_id,
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
            battery_percentage,
            frequency_hz,
            bandwidth_hz,
            associated_station,
            mode,
        }
    }
}

impl ConnectivityLocal {
    pub fn new(
        session_id: Option<i64>,
        device_id: Option<i64>,
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
        battery_percentage: Option<f32>,
        frequency_hz: Option<f32>,
        bandwidth_hz: Option<f32>,
        associated_station: Option<String>,
        mode: Option<String>,
    ) -> Self {
        use chrono::{DateTime, Utc};
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| DateTime::<Utc>::from_timestamp(0, 0).unwrap())
            .to_rfc3339();

        Self {
            id: None,
            id_local: None,
            session_id,
            device_id,
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
            battery_percentage,
            frequency_hz,
            bandwidth_hz,
            associated_station,
            mode,
        }
    }
}

// ===== MIGRATION FROM V3 TO V4 =====
impl From<super::v3::ConnectivityLocal> for ConnectivityLocal {
    fn from(v3: super::v3::ConnectivityLocal) -> Self {
        Self {
            id: v3.id,
            id_local: v3.id_local,
            session_id: v3.session_id,
            device_id: v3.device_id,
            ancestor_id_local: v3.ancestor_id_local,
            inserted_at: v3.inserted_at,
            timestamp_start: v3.timestamp_start,
            signal: v3.signal,
            noise: v3.noise,
            altitude: v3.altitude,
            heading: v3.heading,
            location: v3.location,
            h14_index: v3.h14_index,
            h13_index: v3.h13_index,
            h12_index: v3.h12_index,
            h11_index: v3.h11_index,
            battery_percentage: v3.battery_percentage,
            frequency_hz: v3.frequency_hz,
            bandwidth_hz: v3.bandwidth_hz,
            associated_station: v3.associated_station,
            // New field in v4 - set to None for migrated data
            mode: None,
        }
    }
}

impl From<super::v3::Connectivity> for Connectivity {
    fn from(v3: super::v3::Connectivity) -> Self {
        Self {
            id: v3.id,
            session_id: v3.session_id,
            device_id: v3.device_id,
            inserted_at: v3.inserted_at,
            timestamp_start: v3.timestamp_start,
            signal: v3.signal,
            noise: v3.noise,
            altitude: v3.altitude,
            heading: v3.heading,
            location: v3.location,
            h14_index: v3.h14_index,
            h13_index: v3.h13_index,
            h12_index: v3.h12_index,
            h11_index: v3.h11_index,
            battery_percentage: v3.battery_percentage,
            frequency_hz: v3.frequency_hz,
            bandwidth_hz: v3.bandwidth_hz,
            associated_station: v3.associated_station,
            // New field in v4 - set to None for migrated data
            mode: None,
        }
    }
}

// ===== MIGRATION FROM V2 TO V4 =====
impl From<super::v2::ConnectivityLocal> for ConnectivityLocal {
    fn from(v2: super::v2::ConnectivityLocal) -> Self {
        Self {
            id: v2.id,
            id_local: v2.id_local,
            session_id: v2.session_id,
            device_id: v2.device_id,
            ancestor_id_local: v2.ancestor_id_local,
            inserted_at: v2.inserted_at,
            timestamp_start: v2.timestamp_start,
            signal: v2.signal,
            noise: v2.noise,
            altitude: v2.altitude,
            heading: v2.heading,
            location: v2.location,
            h14_index: v2.h14_index,
            h13_index: v2.h13_index,
            h12_index: v2.h12_index,
            h11_index: v2.h11_index,
            battery_percentage: v2.battery_percentage,
            // New fields from v3 - use defaults for migrated data
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
            // New field in v4 - set to None for migrated data
            mode: None,
        }
    }
}

impl From<super::v2::Connectivity> for Connectivity {
    fn from(v2: super::v2::Connectivity) -> Self {
        Self {
            id: v2.id,
            session_id: v2.session_id,
            device_id: v2.device_id,
            inserted_at: v2.inserted_at,
            timestamp_start: v2.timestamp_start,
            signal: v2.signal,
            noise: v2.noise,
            altitude: v2.altitude,
            heading: v2.heading,
            location: v2.location,
            h14_index: v2.h14_index,
            h13_index: v2.h13_index,
            h12_index: v2.h12_index,
            h11_index: v2.h11_index,
            battery_percentage: v2.battery_percentage,
            // New fields from v3 - use defaults for migrated data
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
            // New field in v4 - set to None for migrated data
            mode: None,
        }
    }
}

// ===== MIGRATION FROM V1 TO V4 =====
impl From<super::v1::ConnectivityLocal> for ConnectivityLocal {
    fn from(v1: super::v1::ConnectivityLocal) -> Self {
        Self {
            id: v1.id,
            id_local: v1.id_local,
            session_id: Some(v1.session_id),
            device_id: None, // v1 doesn't have device_id field
            ancestor_id_local: v1.ancestor_id_local,
            inserted_at: v1.inserted_at,
            timestamp_start: v1.timestamp_start,
            signal: v1.signal,
            noise: v1.noise,
            altitude: v1.altitude,
            heading: v1.heading,
            location: v1.location,
            h14_index: v1.h14_index,
            h13_index: v1.h13_index,
            h12_index: v1.h12_index,
            h11_index: v1.h11_index,
            // New fields from v2 - use defaults for migrated data
            battery_percentage: None,
            // New fields from v3 - use defaults for migrated data
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
            // New field in v4 - set to None for migrated data
            mode: None,
        }
    }
}

impl From<super::v1::Connectivity> for Connectivity {
    fn from(v1: super::v1::Connectivity) -> Self {
        Self {
            id: v1.id,
            session_id: Some(v1.session_id),
            device_id: None, // v1 doesn't have device_id field
            inserted_at: v1.inserted_at,
            timestamp_start: v1.timestamp_start,
            signal: v1.signal,
            noise: v1.noise,
            altitude: v1.altitude,
            heading: v1.heading,
            location: v1.location,
            h14_index: v1.h14_index,
            h13_index: v1.h13_index,
            h12_index: v1.h12_index,
            h11_index: v1.h11_index,
            // New fields from v2 - use defaults for migrated data
            battery_percentage: None,
            // New fields from v3 - use defaults for migrated data
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
            // New field in v4 - set to None for migrated data
            mode: None,
        }
    }
}
