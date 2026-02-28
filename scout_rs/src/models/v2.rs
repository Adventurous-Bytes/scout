use chrono::{DateTime, Utc};
use native_db::{native_db, ToKey};
use native_model::{native_model, Model};
use serde::{Deserialize, Serialize};

// Re-export from v1 (Artifact id 4 is legacy; Artifact id 19 v1 is ArtifactLocalV1/ArtifactV1; v2 defines Artifact id 19 v2)
pub use super::v1::{
    Action, AncestorLocal, ArtifactLocalV1, ArtifactV1, Device, DevicePrettyLocation, DeviceType,
    Heartbeat, Herd, Layer, MediaType, Plan, PlanInsert, PlanType, ResponseScout, ResponseScoutStatus,
    Session, SessionLocal, Syncable, Tag, TagLocal, TagObservationType, Zone,
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
    // NEW FIELD IN V2
    pub battery_percentage: Option<f32>,
}

// ===== NEW OPERATOR MODEL =====
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 18, version = 1)]
#[native_db]
pub struct OperatorLocal {
    pub id: Option<i64>,
    #[primary_key]
    pub id_local: Option<String>,
    pub created_at: Option<String>,
    pub timestamp: Option<String>,
    #[secondary_key]
    pub session_id: Option<i64>,
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    pub user_id: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Operator {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub timestamp: Option<String>,
    pub session_id: Option<i64>,
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

impl Default for OperatorLocal {
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

impl Default for Operator {
    fn default() -> Self {
        Self {
            id: None,
            created_at: None,
            timestamp: None,
            session_id: None,
            user_id: String::new(),
            action: String::new(),
        }
    }
}

impl AncestorLocal for OperatorLocal {
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

impl Syncable for OperatorLocal {
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

impl Syncable for Operator {
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

impl From<OperatorLocal> for Operator {
    fn from(local: OperatorLocal) -> Self {
        Operator {
            id: local.id,
            created_at: local.created_at,
            timestamp: local.timestamp,
            session_id: local.session_id,
            user_id: local.user_id,
            action: local.action,
        }
    }
}

impl From<Operator> for OperatorLocal {
    fn from(operator: Operator) -> Self {
        OperatorLocal {
            id: operator.id,
            id_local: None, // API structs don't have id_local
            created_at: operator.created_at,
            timestamp: operator.timestamp,
            session_id: operator.session_id,
            ancestor_id_local: None, // API structs don't have ancestor_id_local
            user_id: operator.user_id,
            action: operator.action,
        }
    }
}

impl OperatorLocal {
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

impl Operator {
    pub fn new(user_id: String, action: String, session_id: Option<i64>) -> Self {
        Self {
            id: None,
            created_at: None,
            timestamp: Some(Utc::now().to_rfc3339()),
            session_id,
            user_id,
            action,
        }
    }
}


// ===== ARTIFACT V2 (id 19, version 2) - with embeddings =====
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 19, version = 2)]
#[native_db]
pub struct ArtifactLocal {
    pub id: Option<i64>,
    #[primary_key]
    pub id_local: Option<String>,
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    pub created_at: Option<String>,
    pub file_path: String,
    #[secondary_key]
    pub session_id: Option<i64>,
    pub timestamp_observation: Option<String>,
    pub modality: Option<String>,
    pub device_id: i64,
    pub updated_at: Option<String>,
    pub timestamp_observation_end: String,
    pub has_uploaded_file_to_storage: bool,
    pub upload_url: Option<String>,
    pub upload_url_generated_at: Option<String>,
    pub embedding_qwen_vl_2b: Option<Vec<f32>>,
    pub embedding_vertex_mm_01: Option<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    pub file_path: String,
    pub session_id: Option<i64>,
    pub timestamp_observation: Option<String>,
    pub modality: Option<String>,
    pub device_id: i64,
    pub updated_at: Option<String>,
    pub timestamp_observation_end: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "super::serde_helpers::deserialize_embedding")]
    pub embedding_qwen_vl_2b: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "super::serde_helpers::deserialize_embedding")]
    pub embedding_vertex_mm_01: Option<Vec<f32>>,
}

impl Default for ArtifactLocal {
    fn default() -> Self {
        use chrono::Utc;
        Self {
            id: None,
            id_local: None,
            ancestor_id_local: None,
            created_at: None,
            file_path: String::new(),
            session_id: None,
            timestamp_observation: None,
            modality: None,
            device_id: 0,
            updated_at: None,
            timestamp_observation_end: Utc::now().to_rfc3339(),
            has_uploaded_file_to_storage: false,
            upload_url: None,
            upload_url_generated_at: None,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }
}

impl super::v1::Syncable for ArtifactLocal {
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

impl super::v1::AncestorLocal for ArtifactLocal {
    fn ancestor_id_local(&self) -> Option<String> {
        self.ancestor_id_local.clone()
    }

    fn set_ancestor_id_local(&mut self, ancestor_id_local: String) {
        self.ancestor_id_local = Some(ancestor_id_local);
    }
}

impl From<ArtifactLocal> for Artifact {
    fn from(local: ArtifactLocal) -> Self {
        Artifact {
            id: local.id,
            created_at: local.created_at,
            file_path: local.file_path,
            session_id: local.session_id,
            timestamp_observation: local.timestamp_observation,
            modality: local.modality,
            device_id: local.device_id,
            updated_at: local.updated_at,
            timestamp_observation_end: local.timestamp_observation_end,
            embedding_qwen_vl_2b: local.embedding_qwen_vl_2b,
            embedding_vertex_mm_01: local.embedding_vertex_mm_01,
        }
    }
}

impl From<Artifact> for ArtifactLocal {
    fn from(artifact: Artifact) -> Self {
        ArtifactLocal {
            id: artifact.id,
            id_local: None,
            ancestor_id_local: None,
            created_at: artifact.created_at,
            file_path: artifact.file_path,
            session_id: artifact.session_id,
            timestamp_observation: artifact.timestamp_observation,
            modality: artifact.modality,
            device_id: artifact.device_id,
            updated_at: artifact.updated_at,
            timestamp_observation_end: artifact.timestamp_observation_end,
            has_uploaded_file_to_storage: false,
            upload_url: None,
            upload_url_generated_at: None,
            embedding_qwen_vl_2b: artifact.embedding_qwen_vl_2b,
            embedding_vertex_mm_01: artifact.embedding_vertex_mm_01,
        }
    }
}

impl Artifact {
    pub fn new(
        file_path: String,
        session_id: Option<i64>,
        device_id: i64,
        modality: Option<String>,
        timestamp_observation: Option<String>,
    ) -> Self {
        use chrono::Utc;
        Self {
            id: None,
            created_at: None,
            file_path,
            session_id,
            timestamp_observation,
            modality,
            device_id,
            updated_at: None,
            timestamp_observation_end: Utc::now().to_rfc3339(),
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }
}

impl ArtifactLocal {
    pub fn new(
        file_path: String,
        session_id: Option<i64>,
        device_id: i64,
        modality: Option<String>,
        timestamp_observation: Option<String>,
    ) -> Self {
        use chrono::Utc;
        Self {
            id: None,
            id_local: None,
            ancestor_id_local: None,
            created_at: None,
            file_path,
            session_id,
            timestamp_observation,
            modality,
            device_id,
            updated_at: None,
            timestamp_observation_end: Utc::now().to_rfc3339(),
            has_uploaded_file_to_storage: false,
            upload_url: None,
            upload_url_generated_at: None,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }

    pub fn mark_file_uploaded(&mut self) {
        self.has_uploaded_file_to_storage = true;
    }

    pub fn mark_file_not_uploaded(&mut self) {
        self.has_uploaded_file_to_storage = false;
    }

    pub fn is_file_uploaded(&self) -> bool {
        self.has_uploaded_file_to_storage
    }

    pub fn needs_file_upload(&self) -> bool {
        !self.has_uploaded_file_to_storage
    }
}

// ===== MIGRATION FROM ARTIFACT V1 TO V2 (id 19) =====
impl From<super::v1::ArtifactLocalV1> for ArtifactLocal {
    fn from(v1: super::v1::ArtifactLocalV1) -> Self {
        Self {
            id: v1.id,
            id_local: v1.id_local,
            ancestor_id_local: v1.ancestor_id_local,
            created_at: v1.created_at,
            file_path: v1.file_path,
            session_id: v1.session_id,
            timestamp_observation: v1.timestamp_observation,
            modality: v1.modality,
            device_id: v1.device_id,
            updated_at: v1.updated_at,
            timestamp_observation_end: v1.timestamp_observation_end,
            has_uploaded_file_to_storage: v1.has_uploaded_file_to_storage,
            upload_url: v1.upload_url,
            upload_url_generated_at: v1.upload_url_generated_at,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }
}

impl From<super::v1::ArtifactV1> for Artifact {
    fn from(v1: super::v1::ArtifactV1) -> Self {
        Self {
            id: v1.id,
            created_at: v1.created_at,
            file_path: v1.file_path,
            session_id: v1.session_id,
            timestamp_observation: v1.timestamp_observation,
            modality: v1.modality,
            device_id: v1.device_id,
            updated_at: v1.updated_at,
            timestamp_observation_end: v1.timestamp_observation_end,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }
}

// ===== EVENT V2 WITH EMBEDDINGS =====
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[native_model(id = 16, version = 2)]
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
    pub media_type: super::v1::MediaType,
    pub device_id: i64,
    pub earthranger_url: Option<String>,
    pub timestamp_observation: String,
    pub is_public: bool,
    #[secondary_key]
    pub session_id: Option<i64>,
    #[secondary_key]
    pub ancestor_id_local: Option<String>,
    /// Qwen VL 2B embedding (2000 dims).
    pub embedding_qwen_vl_2b: Option<Vec<f32>>,
    /// Vertex multimodal embedding (1408 dims).
    pub embedding_vertex_mm_01: Option<Vec<f32>>,
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
    pub media_type: super::v1::MediaType,
    pub device_id: i64,
    pub earthranger_url: Option<String>,
    pub timestamp_observation: String,
    pub is_public: bool,
    pub session_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "super::serde_helpers::deserialize_embedding")]
    pub embedding_qwen_vl_2b: Option<Vec<f32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, deserialize_with = "super::serde_helpers::deserialize_embedding")]
    pub embedding_vertex_mm_01: Option<Vec<f32>>,
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
            media_type: super::v1::MediaType::Image,
            device_id: 0,
            earthranger_url: None,
            timestamp_observation: String::new(),
            is_public: false,
            session_id: None,
            ancestor_id_local: None,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
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
            media_type: super::v1::MediaType::Image,
            device_id: 0,
            earthranger_url: None,
            timestamp_observation: String::new(),
            is_public: false,
            session_id: None,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
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
        None
    }

    fn set_id_local(&mut self, _id_local: String) {}
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
            embedding_qwen_vl_2b: local.embedding_qwen_vl_2b,
            embedding_vertex_mm_01: local.embedding_vertex_mm_01,
        }
    }
}

impl From<Event> for EventLocal {
    fn from(event: Event) -> Self {
        EventLocal {
            id: event.id,
            id_local: None,
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
            ancestor_id_local: None,
            embedding_qwen_vl_2b: event.embedding_qwen_vl_2b,
            embedding_vertex_mm_01: event.embedding_vertex_mm_01,
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
        media_type: super::v1::MediaType,
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
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
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
        media_type: super::v1::MediaType,
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
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }

    pub fn format_location(latitude: f64, longitude: f64) -> String {
        format!("POINT({} {})", longitude, latitude)
    }
}

// ===== MIGRATION FROM V1 EVENT TO V2 =====
impl From<super::v1::EventLocal> for EventLocal {
    fn from(v1: super::v1::EventLocal) -> Self {
        Self {
            id: v1.id,
            id_local: v1.id_local,
            message: v1.message,
            media_url: v1.media_url,
            file_path: v1.file_path,
            location: v1.location,
            altitude: v1.altitude,
            heading: v1.heading,
            media_type: v1.media_type,
            device_id: v1.device_id,
            earthranger_url: v1.earthranger_url,
            timestamp_observation: v1.timestamp_observation,
            is_public: v1.is_public,
            session_id: v1.session_id,
            ancestor_id_local: v1.ancestor_id_local,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }
}

impl From<super::v1::Event> for Event {
    fn from(v1: super::v1::Event) -> Self {
        Self {
            id: v1.id,
            message: v1.message,
            media_url: v1.media_url,
            file_path: v1.file_path,
            location: v1.location,
            altitude: v1.altitude,
            heading: v1.heading,
            media_type: v1.media_type,
            device_id: v1.device_id,
            earthranger_url: v1.earthranger_url,
            timestamp_observation: v1.timestamp_observation,
            is_public: v1.is_public,
            session_id: v1.session_id,
            embedding_qwen_vl_2b: None,
            embedding_vertex_mm_01: None,
        }
    }
}
