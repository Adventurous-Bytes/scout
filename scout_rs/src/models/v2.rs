use chrono::{DateTime, Utc};
use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// Re-export all unchanged models from v1
pub use super::v1::{
    Action, AncestorLocal, Artifact, Device, DevicePrettyLocation, DeviceType, Event, EventLocal,
    Heartbeat, Herd, Layer, MediaType, Plan, PlanInsert, PlanType, ResponseScout,
    ResponseScoutStatus, Session, SessionLocal, Syncable, Tag, TagLocal, TagObservationType, Zone,
};

// ===== CONNECTIVITY V2 WITH BATTERY_PERCENTAGE =====
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 15, version = 2)]
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
    // NEW FIELD IN V2
    pub battery_percentage: Option<f32>,
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
    // NEW FIELD IN V2
    pub battery_percentage: Option<f32>,
}

// ===== NEW OPERATOR MODEL =====
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 18, version = 1)]
#[native_db]
pub struct Operator {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip)]
    #[primary_key]
    pub id_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub timestamp: Option<String>,
    #[secondary_key]
    pub session_id: Option<i64>,
    #[serde(skip)]
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    pub user_id: String,
    pub action: String,
}

// ===== MIGRATION FROM V1 TO V2 =====
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
            battery_percentage: None, // Default for migrated data
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
            battery_percentage: None, // Default for migrated data
        }
    }
}

// ===== IMPLEMENTATIONS FOR V2 MODELS =====
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
        }
    }
}

impl Default for Connectivity {
    fn default() -> Self {
        Self {
            id: None,
            session_id: None,
            device_id: None,
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
        }
    }
}

impl Default for Operator {
    fn default() -> Self {
        Self {
            id: None,
            id_local: None,
            created_at: None,
            timestamp: None,
            session_id: None,
            ancestor_id_local: None,
            user_id: String::new(),
            action: String::new(),
        }
    }
}

impl AncestorLocal for Operator {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
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
        None
    }

    fn set_id_local(&mut self, _id_local: String) {}
}

impl Syncable for Operator {
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
        }
    }
}

impl From<Connectivity> for ConnectivityLocal {
    fn from(connectivity: Connectivity) -> Self {
        ConnectivityLocal {
            id: connectivity.id,
            id_local: None,
            session_id: connectivity.session_id,
            device_id: connectivity.device_id,
            ancestor_id_local: None,
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
    ) -> Self {
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
    ) -> Self {
        let timestamp_start_str = DateTime::from_timestamp(timestamp_start as i64, 0)
            .unwrap_or_else(|| Utc::now())
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
        }
    }
}

impl Operator {
    pub fn new(user_id: String, action: String, session_id: Option<i64>) -> Self {
        Self {
            id: None,
            id_local: None,
            created_at: None,
            timestamp: Some(Utc::now().to_rfc3339()),
            session_id,
            ancestor_id_local: None,
            user_id,
            action,
        }
    }
}
