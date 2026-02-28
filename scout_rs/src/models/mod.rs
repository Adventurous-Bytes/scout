pub mod health_metric;
pub mod serde_helpers;
pub mod v1;
pub mod v2;
pub mod v3;
pub mod v4;

// ===== VERSIONED MODELS FOLLOWING NATIVE_DB PATTERN =====
// Following the pattern from the native_db documentation:
// https://docs.rs/native_db/latest/native_db/

pub mod data {
    // Type aliases pointing to the latest versions
    pub type ConnectivityLocal = super::v4::ConnectivityLocal;
    pub type Connectivity = super::v4::Connectivity;
    pub type OperatorLocal = super::v2::OperatorLocal; // New model in v2
    pub type Operator = super::v2::Operator; // New model in v2
    pub type ArtifactLocal = super::v2::ArtifactLocal; // Artifact v2 (id 19) in v2.rs
    pub type Artifact = super::v2::Artifact;

    // Other models that haven't changed stay at v1
    pub type Device = super::v1::Device;
    pub type DevicePrettyLocation = super::v1::DevicePrettyLocation;
    pub type Herd = super::v1::Herd;
    pub type SessionLocal = super::v1::SessionLocal;
    pub type Session = super::v1::Session;
    pub type EventLocal = super::v2::EventLocal; // Event v2 with embeddings
    pub type Event = super::v2::Event;
    pub type TagLocal = super::v1::TagLocal;
    pub type Tag = super::v1::Tag;
    pub type Plan = super::v1::Plan;
    pub type PlanInsert = super::v1::PlanInsert;
    pub type Layer = super::v1::Layer;
    pub type Zone = super::v1::Zone;
    pub type Action = super::v1::Action;
    pub type Heartbeat = super::v1::Heartbeat;
    pub type HealthMetric = super::health_metric::HealthMetric;

    // Re-export versioned modules for direct access
    pub use super::{v1, v2, v3, v4};
}

// Re-export for backward compatibility at the top level
pub use data::*;

// Re-export common traits and enums that are shared across versions
pub use v1::{
    AncestorLocal, DeviceType, MediaType, PlanType, ResponseScout, ResponseScoutStatus, Syncable,
    TagObservationType,
};
