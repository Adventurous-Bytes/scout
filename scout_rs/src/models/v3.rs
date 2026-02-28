use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// Re-export from v2 (includes Artifact id 19 v2, Event v2)
pub use super::v2::{
    Action, AncestorLocal, Artifact, ArtifactLocal, Device, DevicePrettyLocation, DeviceType, Event,
    EventLocal, Heartbeat, Herd, Layer, MediaType, Operator, OperatorLocal, Plan, PlanInsert,
    PlanType, ResponseScout, ResponseScoutStatus, Session, SessionLocal, Syncable, Tag, TagLocal,
    TagObservationType, Zone,
};

// ===== CONNECTIVITY V3 WITH FREQUENCY, BANDWIDTH, AND ASSOCIATED STATION =====
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 15, version = 3)]
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
    // FIELD FROM V2
    pub battery_percentage: Option<f32>,
    // NEW FIELDS IN V3
    pub frequency_hz: Option<f32>,
    pub bandwidth_hz: Option<f32>,
    pub associated_station: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Connectivity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub session_id: Option<i64>,
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
    // FIELD FROM V2
    pub battery_percentage: Option<f32>,
    // NEW FIELDS IN V3
    pub frequency_hz: Option<f32>,
    pub bandwidth_hz: Option<f32>,
    pub associated_station: Option<String>,
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
        Connectivity {
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
        }
    }
}

impl From<Connectivity> for ConnectivityLocal {
    fn from(connectivity: Connectivity) -> Self {
        ConnectivityLocal {
            id: connectivity.id,
            id_local: None, // API structs don't have id_local
            session_id: connectivity.session_id,
            device_id: connectivity.device_id,
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
            battery_percentage: connectivity.battery_percentage,
            frequency_hz: connectivity.frequency_hz,
            bandwidth_hz: connectivity.bandwidth_hz,
            associated_station: connectivity.associated_station,
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
        }
    }
}

// ===== MIGRATION FROM V2 TO V3 =====
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
            // Default for new fields in v3
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
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
            // Default for new fields in v3
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
        }
    }
}

// ===== MIGRATION FROM V1 TO V3 =====
impl From<super::v1::ConnectivityLocal> for ConnectivityLocal {
    fn from(v1: super::v1::ConnectivityLocal) -> Self {
        Self {
            id: v1.id,
            id_local: v1.id_local,
            session_id: Some(v1.session_id),
            device_id: None, // New field, default to None for migrated data
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
            // Default for v2 and v3 fields
            battery_percentage: None,
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
        }
    }
}

impl From<super::v1::Connectivity> for Connectivity {
    fn from(v1: super::v1::Connectivity) -> Self {
        Self {
            id: v1.id,
            session_id: Some(v1.session_id),
            device_id: None, // New field, default to None for migrated data
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
            // Default for v2 and v3 fields
            battery_percentage: None,
            frequency_hz: None,
            bandwidth_hz: None,
            associated_station: None,
        }
    }
}
