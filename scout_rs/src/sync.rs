use crate::{
    client::ScoutClient,
    models::{
        data, Connectivity, ConnectivityLocal, Event, EventLocal, Session, SessionLocal, Syncable,
        Tag, TagLocal,
    },
};
use anyhow::{Error, Result};
use native_db::{Builder, Database, Models, ToInput};
use once_cell::sync::Lazy;
use tracing::error;

// Static models instance shared across all SyncEngine instances
static MODELS: Lazy<Models> = Lazy::new(|| {
    let mut models = Models::new();
    models
        .define::<SessionLocal>()
        .expect("Failed to define SessionLocal model");
    models
        .define::<EventLocal>()
        .expect("Failed to define EventLocal model");
    models
        .define::<TagLocal>()
        .expect("Failed to define TagLocal model");

    // Define v1 connectivity model (existing data)
    models
        .define::<data::v1::ConnectivityLocal>()
        .expect("Failed to define v1 ConnectivityLocal model");

    // Define v2 connectivity model (new data with battery_percentage)
    models
        .define::<data::v2::ConnectivityLocal>()
        .expect("Failed to define v2 ConnectivityLocal model");

    // Define new Operator model
    models
        .define::<data::v2::OperatorLocal>()
        .expect("Failed to define Operator model");

    models
});

/// SyncEngine handles synchronization between local database and remote Scout server.
///
/// The sync engine maintains a hierarchical sync order:
/// 1. Sessions (parent entities)
/// 2. Connectivity entries (children of sessions)
/// 3. Events (children of sessions)
/// 4. Tags (children of events)
///
/// Features:
/// - Batch operations for efficiency
/// - Automatic ID relationship management
/// - Configurable sync intervals and batch sizes
/// - Auto-cleaning of completed sessions
/// - Resilient error handling with partial failure recovery
pub struct SyncEngine {
    scout_client: ScoutClient,
    db_local_path: String,
    database: Database<'static>,
    interval_flush_sessions_ms: Option<u64>,
    max_num_items_per_sync: Option<u64>,
    auto_clean: bool,
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
}

pub enum EnumSyncAction {
    Upsert,
    Insert,
    Skip,
}

const DEFAULT_INTERVAL_FLUSH_SESSIONS_MS: u64 = 20_000;
const DEFAULT_MAX_NUM_ITEMS_PER_SYNC: u64 = 100;

pub struct BatchSync<T: ToInput + Syncable> {
    upsert: Vec<T>,
    insert: Vec<T>,
}

impl<T: ToInput + Syncable> BatchSync<T> {
    fn new() -> Self {
        Self {
            upsert: Vec::new(),
            insert: Vec::new(),
        }
    }

    fn add_upsert_item(&mut self, item: T) {
        self.upsert.push(item);
    }

    fn add_insert_item(&mut self, item: T) {
        self.insert.push(item);
    }
}

impl SyncEngine {
    /// Creates a new SyncEngine with custom configuration.
    ///
    /// # Arguments
    /// * `scout_client` - Client for communicating with Scout server
    /// * `db_local_path` - Path to local database file
    /// * `interval_flush_sessions_ms` - How often to sync (None = manual only)
    /// * `max_num_items_per_sync` - Maximum items per sync batch (None = unlimited)
    /// * `auto_clean` - Whether to automatically clean completed sessions
    pub fn new(
        scout_client: ScoutClient,
        db_local_path: String,
        interval_flush_sessions_ms: Option<u64>,
        max_num_items_per_sync: Option<u64>,
        auto_clean: bool,
    ) -> Result<Self> {
        // Create database using static models reference
        let database = Builder::new().create(&*MODELS, &db_local_path)?;
        // initialize tracing
        Ok(Self {
            scout_client,
            db_local_path,
            database,
            interval_flush_sessions_ms,
            max_num_items_per_sync,
            auto_clean,
            shutdown_tx: None,
        })
    }

    /// Creates a default SyncEngine with common settings:
    /// - 3 second sync interval
    /// - 100 items per sync batch
    /// - Auto-clean enabled
    pub fn with_defaults(scout_client: ScoutClient, db_local_path: String) -> Result<Self> {
        Self::new(
            scout_client,
            db_local_path,
            Some(DEFAULT_INTERVAL_FLUSH_SESSIONS_MS),
            Some(DEFAULT_MAX_NUM_ITEMS_PER_SYNC),
            true, // Enable auto-clean by default
        )
    }

    fn get_batch<T: Syncable + ToInput>(
        &self,
        action_for_items_with_existing_ids: EnumSyncAction,
        action_for_items_without_existing_ids: EnumSyncAction,
    ) -> Result<BatchSync<T>, Error> {
        let r = self.database.r_transaction()?;
        let mut batch: BatchSync<T> = BatchSync::new();

        for raw_item in r.scan().primary::<T>()?.all()? {
            match raw_item {
                Ok(item) => {
                    // handle action for existing remote ids (on remote)
                    if item.id().is_some() {
                        match action_for_items_with_existing_ids {
                            EnumSyncAction::Insert => {
                                batch.add_insert_item(item);
                            }
                            EnumSyncAction::Upsert => {
                                batch.add_upsert_item(item);
                            }
                            EnumSyncAction::Skip => {
                                // Skip items that already have remote IDs
                            }
                        }
                    }
                    // handle action for no remote id (local only)
                    else {
                        match action_for_items_without_existing_ids {
                            EnumSyncAction::Insert => {
                                batch.add_insert_item(item);
                            }
                            EnumSyncAction::Upsert => {
                                batch.add_upsert_item(item);
                            }
                            EnumSyncAction::Skip => {
                                // Skip items without remote IDs (shouldn't happen)
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to process item: {}", e);
                }
            }
        }
        Ok(batch)
    }

    /// Flushes all local data to remote server in proper order: sessions -> connectivity -> events -> operators -> tags
    /// Continues with remaining operations even if one fails, but reports all errors
    pub async fn flush(&mut self) -> Result<(), Error> {
        let mut sync_errors = Vec::new();

        // Sync sessions first (they're the parent of everything)
        if let Err(e) = self.flush_sessions().await {
            sync_errors.push(format!("Sessions sync failed: {}", e));
            tracing::error!(
                "Sessions sync failed, continuing with other operations: {}",
                e
            );
        }

        // Sync connectivity (depends on sessions)
        if let Err(e) = self.flush_connectivity().await {
            sync_errors.push(format!("Connectivity sync failed: {}", e));
            tracing::error!(
                "Connectivity sync failed, continuing with other operations: {}",
                e
            );
        }

        // Sync events (depends on sessions)
        if let Err(e) = self.flush_events().await {
            sync_errors.push(format!("Events sync failed: {}", e));
            tracing::error!(
                "Events sync failed, continuing with other operations: {}",
                e
            );
        }

        // Sync operators (depends on sessions)
        if let Err(e) = self.flush_operators().await {
            sync_errors.push(format!("Operators sync failed: {}", e));
            tracing::error!(
                "Operators sync failed, continuing with other operations: {}",
                e
            );
        }

        // Sync tags (depends on events)
        if let Err(e) = self.flush_tags().await {
            sync_errors.push(format!("Tags sync failed: {}", e));
            tracing::error!("Tags sync failed: {}", e);
        }

        // Auto clean if enabled and no critical errors occurred
        if self.auto_clean && sync_errors.is_empty() {
            if let Err(e) = self.clean().await {
                sync_errors.push(format!("Clean operation failed: {}", e));
                tracing::error!("Clean operation failed: {}", e);
            }
        }

        // Return error if any operations failed
        if !sync_errors.is_empty() {
            return Err(Error::msg(format!(
                "Sync completed with errors: {}",
                sync_errors.join("; ")
            )));
        }

        Ok(())
    }

    /// Syncs sessions to remote server
    async fn flush_sessions(&mut self) -> Result<(), Error> {
        // For sessions, we always upsert because they can be updated (e.g., timestamp_end)
        let sessions_batch: BatchSync<SessionLocal> = self.get_batch::<SessionLocal>(
            EnumSyncAction::Upsert, // Always upsert sessions with remote IDs
            EnumSyncAction::Upsert, // Always upsert sessions without remote IDs (insert)
        )?;

        // Process insert and upsert batches separately to avoid "All object keys must match" errors
        if !sessions_batch.insert.is_empty() {
            self.process_session_batch(sessions_batch.insert).await?;
        }
        if !sessions_batch.upsert.is_empty() {
            self.process_session_batch(sessions_batch.upsert).await?;
        }

        Ok(())
    }

    /// Processes a batch of sessions with fallback to individual processing on bulk failure
    async fn process_session_batch(
        &mut self,
        mut sessions: Vec<SessionLocal>,
    ) -> Result<(), Error> {
        if sessions.is_empty() {
            return Ok(());
        }

        // Apply batch size limit
        if let Some(max_items) = self.max_num_items_per_sync {
            if sessions.len() > max_items as usize {
                sessions.truncate(max_items as usize);
            }
        }

        let sessions_for_upsert: Vec<Session> = sessions
            .iter()
            .map(|local_session| local_session.clone().into())
            .collect();

        // Try bulk upsert first, fallback to individual on key mismatch errors
        let response = match self
            .scout_client
            .upsert_sessions_batch(&sessions_for_upsert)
            .await
        {
            Ok(response) => response,
            Err(e)
                if e.to_string()
                    .to_lowercase()
                    .contains("all object keys must match") =>
            {
                return self.fallback_individual_upserts(sessions).await;
            }
            Err(e) => return Err(e),
        };

        // Process successful bulk response
        if let Some(upserted_sessions) = response.data {
            let updated_locals: Vec<SessionLocal> = upserted_sessions
                .into_iter()
                .zip(sessions.iter())
                .map(|(remote_session, original_local)| {
                    let mut updated_local: SessionLocal = remote_session.into();
                    updated_local.id_local = original_local.id_local.clone();
                    updated_local
                })
                .collect();

            self.upsert_items(updated_locals.clone())?;

            // Update descendants for new sessions - only if parent exists and was newly created
            for (updated, original) in updated_locals.iter().zip(sessions.iter()) {
                if let (Some(new_id), Some(local_id), None) =
                    (updated.id, &original.id_local, original.id)
                {
                    // Validate the session was actually saved before updating descendants
                    if self
                        .validate_session_exists(local_id, new_id)
                        .unwrap_or(false)
                    {
                        if let Err(e) = self.update_session_descendants(local_id, new_id) {
                            tracing::error!(
                                "Failed to update descendants for session {}: {}",
                                local_id,
                                e
                            );
                        }
                    } else {
                        tracing::warn!(
                            "Session {} with remote ID {} not found - skipping descendant updates",
                            local_id,
                            new_id
                        );
                    }
                }
            }
        }
        Ok(())
    }

    /// Fallback to individual session upserts when bulk fails
    async fn fallback_individual_upserts(
        &mut self,
        sessions: Vec<SessionLocal>,
    ) -> Result<(), Error> {
        for session in sessions {
            let session_for_upsert: Session = session.clone().into();

            match self
                .scout_client
                .upsert_sessions_batch(&[session_for_upsert])
                .await
            {
                Ok(response) => {
                    if let Some(mut upserted_sessions) = response.data {
                        if let Some(upserted_session) = upserted_sessions.pop() {
                            let mut updated_local: SessionLocal = upserted_session.into();
                            updated_local.id_local = session.id_local.clone();
                            self.upsert_items(vec![updated_local.clone()])?;

                            // Update descendants for new sessions - validate parent exists first
                            if let (Some(new_id), Some(local_id), None) =
                                (updated_local.id, &session.id_local, session.id)
                            {
                                if self
                                    .validate_session_exists(local_id, new_id)
                                    .unwrap_or(false)
                                {
                                    if let Err(e) =
                                        self.update_session_descendants(local_id, new_id)
                                    {
                                        tracing::error!("Failed to update descendants: {}", e);
                                    }
                                } else {
                                    tracing::warn!(
                                        "Session {} with remote ID {} not validated - skipping descendants",
                                        local_id,
                                        new_id
                                    );
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Individual session upsert failed: {}", e);
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    /// Syncs connectivity entries to remote server
    async fn flush_connectivity(&mut self) -> Result<(), Error> {
        // For connectivity, we only process items without remote IDs (new items to insert)
        let connectivity_batch: BatchSync<ConnectivityLocal> = self
            .get_batch::<ConnectivityLocal>(
                EnumSyncAction::Skip,   // Skip items with remote IDs - they're already synced
                EnumSyncAction::Insert, // Process items without remote IDs
            )?;

        // Only process items without remote IDs (the insert batch)
        let mut all_connectivity = connectivity_batch.insert;

        if let Some(max_items) = self.max_num_items_per_sync {
            if all_connectivity.len() > max_items as usize {
                tracing::info!(
                    "Limiting connectivity sync from {} to {} items",
                    all_connectivity.len(),
                    max_items
                );
                all_connectivity.truncate(max_items as usize);
            }
        }

        if all_connectivity.is_empty() {
            return Ok(());
        }

        // CRITICAL FIX: Update descendants BEFORE sending to remote server
        // Check if any connectivity records have ancestors with remote IDs and update descendants first
        let mut sessions_to_update = std::collections::HashSet::new();
        for connectivity in all_connectivity.iter() {
            if let Some(ancestor_local_id) = &connectivity.ancestor_id_local {
                // Check if the ancestor session has a remote ID
                if let Ok(Some(session)) = self.get_item::<SessionLocal>(ancestor_local_id) {
                    if let Some(_remote_session_id) = session.id {
                        // Session exists and has remote ID, mark for descendant updates
                        sessions_to_update.insert(ancestor_local_id.clone());
                    }
                }
            }
        }

        // Update descendants for all sessions that have remote IDs
        // This ensures connectivity records get their session_id populated BEFORE remote sync
        for session_local_id in sessions_to_update {
            if let Ok(Some(session)) = self.get_item::<SessionLocal>(&session_local_id) {
                if let Some(remote_session_id) = session.id {
                    if let Err(e) =
                        self.update_session_descendants(&session_local_id, remote_session_id)
                    {
                        tracing::error!(
                            "Failed to update descendants for session {} before connectivity sync: {}",
                            session_local_id,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "Updated descendants for session {} before connectivity sync",
                            session_local_id
                        );
                    }
                }
            }
        }

        // NOW re-fetch the connectivity records (they may have been updated with session_id)
        // We need to get the updated versions with populated session_id values
        let mut updated_all_connectivity = Vec::new();
        for conn in all_connectivity.iter() {
            if let Some(local_id) = &conn.id_local {
                if let Ok(Some(updated_conn)) = self.get_item::<ConnectivityLocal>(local_id) {
                    updated_all_connectivity.push(updated_conn);
                } else {
                    // Fallback to original if we can't find the updated version
                    updated_all_connectivity.push(conn.clone());
                }
            } else {
                updated_all_connectivity.push(conn.clone());
            }
        }

        // Now convert the UPDATED connectivity records for remote sync
        let connectivity_for_insert: Vec<Connectivity> = updated_all_connectivity
            .iter()
            .map(|local_connectivity| local_connectivity.clone().into())
            .collect();

        let response = self
            .scout_client
            .upsert_connectivity_batch(&connectivity_for_insert)
            .await?;

        if let Some(inserted_connectivity) = response.data {
            let final_connectivity: Vec<ConnectivityLocal> = inserted_connectivity
                .into_iter()
                .zip(updated_all_connectivity.iter())
                .map(|(remote_connectivity, original_local)| {
                    let mut updated_local: ConnectivityLocal = remote_connectivity.into();
                    updated_local.id_local = original_local.id_local.clone();
                    updated_local.ancestor_id_local = original_local.ancestor_id_local.clone();
                    updated_local
                })
                .collect();

            self.upsert_items(final_connectivity)?;
        }

        Ok(())
    }

    /// Syncs events to remote server
    async fn flush_events(&mut self) -> Result<(), Error> {
        // For events, we only process items without remote IDs (new items to insert)
        let events_batch: BatchSync<EventLocal> = self.get_batch::<EventLocal>(
            EnumSyncAction::Skip,   // Skip items with remote IDs - they're already synced
            EnumSyncAction::Insert, // Process items without remote IDs
        )?;

        // Only process items without remote IDs (the insert batch)
        let mut all_events = events_batch.insert;

        if let Some(max_items) = self.max_num_items_per_sync {
            if all_events.len() > max_items as usize {
                tracing::info!(
                    "Limiting events sync from {} to {} items",
                    all_events.len(),
                    max_items
                );
                all_events.truncate(max_items as usize);
            }
        }

        if all_events.is_empty() {
            return Ok(());
        }

        // CRITICAL FIX: Update descendants BEFORE sending to remote server
        // Check if any events have session ancestors with remote IDs and update descendants first
        let mut sessions_to_update = std::collections::HashSet::new();
        for event in all_events.iter() {
            if let Some(ancestor_local_id) = &event.ancestor_id_local {
                // Check if the ancestor session has a remote ID
                if let Ok(Some(session)) = self.get_item::<SessionLocal>(ancestor_local_id) {
                    if let Some(_remote_session_id) = session.id {
                        // Session exists and has remote ID, mark for descendant updates
                        sessions_to_update.insert(ancestor_local_id.clone());
                    }
                }
            }
        }

        // Update descendants for all sessions that have remote IDs
        // This ensures events get their session_id populated BEFORE remote sync
        for session_local_id in sessions_to_update {
            if let Ok(Some(session)) = self.get_item::<SessionLocal>(&session_local_id) {
                if let Some(remote_session_id) = session.id {
                    if let Err(e) =
                        self.update_session_descendants(&session_local_id, remote_session_id)
                    {
                        tracing::error!(
                            "Failed to update descendants for session {} before event sync: {}",
                            session_local_id,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "Updated descendants for session {} before event sync",
                            session_local_id
                        );
                    }
                }
            }
        }

        // NOW re-fetch the events (they may have been updated with session_id)
        // We need to get the updated versions with populated session_id values
        let mut updated_all_events = Vec::new();
        for event in all_events.iter() {
            if let Some(local_id) = &event.id_local {
                if let Ok(Some(updated_event)) = self.get_item::<EventLocal>(local_id) {
                    updated_all_events.push(updated_event);
                } else {
                    // Fallback to original if we can't find the updated version
                    updated_all_events.push(event.clone());
                }
            } else {
                updated_all_events.push(event.clone());
            }
        }

        // Now convert the UPDATED events for remote sync
        let events_for_insert: Vec<Event> = updated_all_events
            .iter()
            .map(|local_event| local_event.clone().into())
            .collect();

        let response = self
            .scout_client
            .upsert_events_batch(&events_for_insert)
            .await?;

        if let Some(inserted_events) = response.data {
            let final_events: Vec<EventLocal> = inserted_events
                .into_iter()
                .zip(updated_all_events.iter())
                .map(|(remote_event, original_local)| {
                    let mut updated_local: EventLocal = remote_event.into();
                    updated_local.id_local = original_local.id_local.clone();
                    updated_local.ancestor_id_local = original_local.ancestor_id_local.clone();
                    updated_local
                })
                .collect();

            self.upsert_items(final_events.clone())?;

            // Update tag descendants with new remote event IDs - validate parent exists first
            for (updated_event, original_event) in
                final_events.iter().zip(updated_all_events.iter())
            {
                if let (Some(new_remote_id), Some(local_id)) =
                    (updated_event.id, &original_event.id_local)
                {
                    if original_event.id.is_none() {
                        // Validate the event was actually saved before updating descendants
                        if self
                            .validate_event_exists(local_id, new_remote_id)
                            .unwrap_or(false)
                        {
                            if let Err(e) = self.update_event_descendants(local_id, new_remote_id) {
                                tracing::error!(
                                    "Failed to update descendants for event {}: {}",
                                    local_id,
                                    e
                                );
                            }
                        } else {
                            tracing::warn!(
                                "Event {} with remote ID {} not found - skipping descendant updates",
                                local_id,
                                new_remote_id
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Syncs tags to remote server
    async fn flush_tags(&mut self) -> Result<(), Error> {
        // For tags, we only process items without remote IDs (new items to insert)
        let tags_batch: BatchSync<TagLocal> = self.get_batch::<TagLocal>(
            EnumSyncAction::Skip,   // Skip items with remote IDs - they're already synced
            EnumSyncAction::Insert, // Process items without remote IDs
        )?;

        // Only process items without remote IDs (the insert batch)
        let mut all_tags = tags_batch.insert;

        if let Some(max_items) = self.max_num_items_per_sync {
            if all_tags.len() > max_items as usize {
                tracing::info!(
                    "Limiting tags sync from {} to {} items",
                    all_tags.len(),
                    max_items
                );
                all_tags.truncate(max_items as usize);
            }
        }

        if all_tags.is_empty() {
            return Ok(());
        }

        // CRITICAL FIX: Update descendants BEFORE sending to remote server
        // Check if any tags have event ancestors with remote IDs and update descendants first
        let mut events_to_update = std::collections::HashSet::new();
        let mut sessions_to_update = std::collections::HashSet::new();

        for tag in all_tags.iter() {
            if let Some(ancestor_local_id) = &tag.ancestor_id_local {
                // Check if the ancestor event has a remote ID
                if let Ok(Some(event)) = self.get_item::<EventLocal>(ancestor_local_id) {
                    if let Some(_remote_event_id) = event.id {
                        // Event exists and has remote ID, mark for descendant updates
                        events_to_update.insert(ancestor_local_id.clone());

                        // Also check if the event has a session ancestor
                        if let Some(session_ancestor_id) = &event.ancestor_id_local {
                            if let Ok(Some(session)) =
                                self.get_item::<SessionLocal>(session_ancestor_id)
                            {
                                if let Some(_remote_session_id) = session.id {
                                    sessions_to_update.insert(session_ancestor_id.clone());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update event descendants first
        for event_local_id in events_to_update {
            if let Ok(Some(event)) = self.get_item::<EventLocal>(&event_local_id) {
                if let Some(remote_event_id) = event.id {
                    if let Err(e) = self.update_event_descendants(&event_local_id, remote_event_id)
                    {
                        tracing::error!(
                            "Failed to update event descendants for event {} before tag sync: {}",
                            event_local_id,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "Updated event descendants for event {} before tag sync",
                            event_local_id
                        );
                    }
                }
            }
        }

        // Update session descendants
        for session_local_id in sessions_to_update {
            if let Ok(Some(session)) = self.get_item::<SessionLocal>(&session_local_id) {
                if let Some(remote_session_id) = session.id {
                    if let Err(e) =
                        self.update_session_descendants(&session_local_id, remote_session_id)
                    {
                        tracing::error!(
                            "Failed to update session descendants for session {} before tag sync: {}",
                            session_local_id,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "Updated session descendants for session {} before tag sync",
                            session_local_id
                        );
                    }
                }
            }
        }

        // NOW re-fetch the tags (they may have been updated with event_id)
        // We need to get the updated versions with populated event_id values
        let mut updated_all_tags = Vec::new();
        for tag in all_tags.iter() {
            if let Some(local_id) = &tag.id_local {
                if let Ok(Some(updated_tag)) = self.get_item::<TagLocal>(local_id) {
                    updated_all_tags.push(updated_tag);
                } else {
                    // Fallback to original if we can't find the updated version
                    updated_all_tags.push(tag.clone());
                }
            } else {
                updated_all_tags.push(tag.clone());
            }
        }

        // Now convert the UPDATED tags for remote sync
        let tags_for_insert: Vec<Tag> = updated_all_tags
            .iter()
            .map(|local_tag| local_tag.clone().into())
            .collect();

        let response = self
            .scout_client
            .upsert_tags_batch(&tags_for_insert)
            .await?;

        if let Some(inserted_tags) = response.data {
            let final_tags: Vec<TagLocal> = inserted_tags
                .into_iter()
                .zip(updated_all_tags.iter())
                .map(|(remote_tag, original_local)| {
                    let mut updated_local: TagLocal = remote_tag.into();
                    updated_local.id_local = original_local.id_local.clone();
                    updated_local.ancestor_id_local = original_local.ancestor_id_local.clone();
                    updated_local
                })
                .collect();

            self.upsert_items(final_tags)?;
        }

        Ok(())
    }

    /// Syncs operators to remote server
    async fn flush_operators(&mut self) -> Result<(), Error> {
        // For operators, we only process items without remote IDs (new items to insert)
        let operators_batch: BatchSync<data::v2::OperatorLocal> = self
            .get_batch::<data::v2::OperatorLocal>(
                EnumSyncAction::Skip,   // Skip items with remote IDs - they're already synced
                EnumSyncAction::Insert, // Process items without remote IDs
            )?;

        // Only process items without remote IDs (the insert batch)
        let mut all_operators = operators_batch.insert;

        if let Some(max_items) = self.max_num_items_per_sync {
            if all_operators.len() > max_items as usize {
                tracing::info!(
                    "Limiting operators sync from {} to {} items",
                    all_operators.len(),
                    max_items
                );
                all_operators.truncate(max_items as usize);
            }
        }

        if all_operators.is_empty() {
            return Ok(());
        }

        // CRITICAL FIX: Update descendants BEFORE sending to remote server
        // Check if any operators have session ancestors with remote IDs and update descendants first
        let mut sessions_to_update = std::collections::HashSet::new();
        for operator in all_operators.iter() {
            if let Some(ancestor_local_id) = &operator.ancestor_id_local {
                // Check if the ancestor session has a remote ID
                if let Ok(Some(session)) = self.get_item::<SessionLocal>(ancestor_local_id) {
                    if let Some(_remote_session_id) = session.id {
                        // Session exists and has remote ID, mark for descendant updates
                        sessions_to_update.insert(ancestor_local_id.clone());
                    }
                }
            }
        }

        // Update descendants for all sessions that have remote IDs
        // This ensures operators get their session_id populated BEFORE remote sync
        for session_local_id in sessions_to_update {
            if let Ok(Some(session)) = self.get_item::<SessionLocal>(&session_local_id) {
                if let Some(remote_session_id) = session.id {
                    if let Err(e) =
                        self.update_session_descendants(&session_local_id, remote_session_id)
                    {
                        tracing::error!(
                            "Failed to update descendants for session {} before operator sync: {}",
                            session_local_id,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "Updated descendants for session {} before operator sync",
                            session_local_id
                        );
                    }
                }
            }
        }

        // NOW re-fetch the operators (they may have been updated with session_id)
        // We need to get the updated versions with populated session_id values
        let mut updated_all_operators = Vec::new();
        for operator in all_operators.iter() {
            if let Some(local_id) = &operator.id_local {
                if let Ok(Some(updated_operator)) =
                    self.get_item::<data::v2::OperatorLocal>(local_id)
                {
                    updated_all_operators.push(updated_operator);
                } else {
                    // Fallback to original if we can't find the updated version
                    updated_all_operators.push(operator.clone());
                }
            } else {
                updated_all_operators.push(operator.clone());
            }
        }

        // Now convert the UPDATED operators for remote sync
        let operators_for_insert: Vec<data::v2::Operator> = updated_all_operators
            .iter()
            .map(|local_operator| {
                // Convert OperatorLocal to Operator (removes local-only fields)
                data::v2::Operator::from(local_operator.clone())
            })
            .collect();

        let response = self
            .scout_client
            .upsert_operators_batch(&operators_for_insert)
            .await?;

        if let Some(inserted_operators) = response.data {
            let final_operators: Vec<data::v2::OperatorLocal> = inserted_operators
                .into_iter()
                .zip(updated_all_operators.iter())
                .map(|(remote_operator, original_local)| {
                    let mut updated_local = data::v2::OperatorLocal::from(remote_operator);
                    updated_local.id_local = original_local.id_local.clone();
                    updated_local.ancestor_id_local = original_local.ancestor_id_local.clone();
                    updated_local
                })
                .collect();

            self.upsert_items(final_operators)?;
        }

        Ok(())
    }

    /// Starts the sync engine with automatic flushing at specified intervals.
    /// This method runs indefinitely until an error occurs or the task is cancelled.
    /// Use `spawn_background_sync` to run this in a background task.
    pub async fn start(&mut self) -> Result<(), Error> {
        if let Some(interval_ms) = self.interval_flush_sessions_ms {
            tracing::info!(
                "Starting sync engine with flush interval: {}ms, max items per sync: {:?}",
                interval_ms,
                self.max_num_items_per_sync
            );

            let (shutdown_tx, mut shutdown_rx) = tokio::sync::broadcast::channel(1);
            self.shutdown_tx = Some(shutdown_tx);

            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_millis(interval_ms));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        match self.flush().await {
                            Ok(_) => {
                                tracing::debug!("Periodic flush completed successfully");
                            }
                            Err(e) => {
                                tracing::error!("Periodic flush failed: {}", e);
                                // Continue running despite failures
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Sync engine shutting down gracefully");
                        break;
                    }
                }
            }
            Ok(())
        } else {
            tracing::warn!("No flush interval specified, sync engine will not run automatically");
            Ok(())
        }
    }

    /// Stops any active auto-flushing session
    pub fn stop(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            if let Err(_) = shutdown_tx.send(()) {
                tracing::warn!("No active sync session to stop");
            } else {
                tracing::info!("Sync engine stop signal sent");
            }
        } else {
            tracing::warn!("No active sync session to stop");
        }
    }

    /// Gets an item from the database by local ID and returns a clone
    pub fn get_item<T: ToInput + Syncable + Clone>(
        &self,
        local_id: &str,
    ) -> Result<Option<T>, Error> {
        let r = self.database.r_transaction()?;

        for raw_item in r.scan().primary::<T>()?.all()? {
            if let Ok(item) = raw_item {
                if let Some(item_local_id) = item.id_local() {
                    if item_local_id == local_id {
                        return Ok(Some(item));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Cleans completed sessions and their descendants from local database
    /// Only removes sessions where timestamp_end is Some, all entities have remote IDs
    pub async fn clean(&mut self) -> Result<(), Error> {
        tracing::info!("Starting clean operation for completed sessions");

        let r = self.database.r_transaction()?;
        let mut sessions_to_clean = Vec::new();

        // Find completed sessions with remote IDs
        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if let (Some(_end_time_str), Some(_remote_id)) =
                    (&session.timestamp_end, session.id)
                {
                    // Check if all descendants have remote IDs
                    if self.session_descendants_have_remote_ids(&session, &r)? {
                        sessions_to_clean.push(session);
                    }
                }
            }
        }
        drop(r);

        if sessions_to_clean.is_empty() {
            tracing::debug!("No completed sessions found for cleaning");
            return Ok(());
        }

        tracing::info!(
            "Found {} completed sessions to clean",
            sessions_to_clean.len()
        );

        // Clean each session and its descendants
        for session in sessions_to_clean {
            self.clean_session_and_descendants(&session).await?;
        }

        Ok(())
    }

    /// Checks if all descendants of a session have remote IDs
    fn session_descendants_have_remote_ids(
        &self,
        session: &SessionLocal,
        r: &native_db::transaction::RTransaction,
    ) -> Result<bool, Error> {
        let session_local_id = match &session.id_local {
            Some(id) => id,
            None => return Ok(false),
        };

        // Check connectivity entries
        for raw_connectivity in r.scan().primary::<ConnectivityLocal>()?.all()? {
            if let Ok(connectivity) = raw_connectivity {
                if connectivity.ancestor_id_local.as_deref() == Some(session_local_id) {
                    if connectivity.id.is_none() {
                        tracing::debug!(
                            "Session {} has connectivity without remote ID",
                            session_local_id
                        );
                        return Ok(false);
                    }
                }
            }
        }

        // Check operators entries
        for raw_operator in r.scan().primary::<data::v2::OperatorLocal>()?.all()? {
            if let Ok(operator) = raw_operator {
                if operator.ancestor_id_local.as_deref() == Some(session_local_id) {
                    if operator.id.is_none() {
                        tracing::debug!(
                            "Session {} has operator without remote ID",
                            session_local_id
                        );
                        return Ok(false);
                    }
                }
            }
        }

        // Check events and their tags
        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(event) = raw_event {
                if event.ancestor_id_local.as_deref() == Some(session_local_id) {
                    if event.id.is_none() {
                        tracing::debug!("Session {} has event without remote ID", session_local_id);
                        return Ok(false);
                    }

                    // Check tags for this event
                    if let Some(event_local_id) = &event.id_local {
                        for raw_tag in r.scan().primary::<TagLocal>()?.all()? {
                            if let Ok(tag) = raw_tag {
                                if tag.ancestor_id_local.as_deref() == Some(event_local_id) {
                                    if tag.id.is_none() {
                                        tracing::debug!(
                                            "Session {} has tag without remote ID for event {}",
                                            session_local_id,
                                            event_local_id
                                        );
                                        return Ok(false);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(true)
    }

    /// Removes a session and all its descendants from local database
    async fn clean_session_and_descendants(&mut self, session: &SessionLocal) -> Result<(), Error> {
        let session_local_id = match &session.id_local {
            Some(id) => id.clone(),
            None => return Ok(()),
        };

        tracing::info!("Cleaning session {} and descendants", session_local_id);

        // First, collect all items to remove using read transaction
        let r = self.database.r_transaction()?;

        let mut tags_to_remove = Vec::new();
        let mut events_to_remove = Vec::new();
        let mut connectivity_to_remove = Vec::new();
        let mut operators_to_remove = Vec::new();

        // Collect events for this session
        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(event) = raw_event {
                if event.ancestor_id_local.as_deref() == Some(&session_local_id) {
                    events_to_remove.push(event);
                }
            }
        }

        // Collect tags for each event
        for event in &events_to_remove {
            if let Some(event_local_id) = &event.id_local {
                for raw_tag in r.scan().primary::<TagLocal>()?.all()? {
                    if let Ok(tag) = raw_tag {
                        if tag.ancestor_id_local.as_deref() == Some(event_local_id) {
                            tags_to_remove.push(tag);
                        }
                    }
                }
            }
        }

        // Collect connectivity entries
        for raw_connectivity in r.scan().primary::<ConnectivityLocal>()?.all()? {
            if let Ok(connectivity) = raw_connectivity {
                if connectivity.ancestor_id_local.as_deref() == Some(&session_local_id) {
                    connectivity_to_remove.push(connectivity);
                }
            }
        }

        // Collect operators entries
        for raw_operator in r.scan().primary::<data::v2::OperatorLocal>()?.all()? {
            if let Ok(operator) = raw_operator {
                if operator.ancestor_id_local.as_deref() == Some(&session_local_id) {
                    operators_to_remove.push(operator);
                }
            }
        }

        drop(r); // Close read transaction

        // Now remove all items using write transaction
        let rw = self.database.rw_transaction()?;

        // Remove tags
        let tags_count = tags_to_remove.len();
        for tag in tags_to_remove {
            rw.remove(tag)?;
        }

        // Remove events
        let events_count = events_to_remove.len();
        for event in events_to_remove {
            rw.remove(event)?;
        }

        // Remove connectivity entries
        let connectivity_count = connectivity_to_remove.len();
        for connectivity in connectivity_to_remove {
            rw.remove(connectivity)?;
        }

        // Remove operators entries
        let operators_count = operators_to_remove.len();
        for operator in operators_to_remove {
            rw.remove(operator)?;
        }

        // Remove the session itself
        rw.remove(session.clone())?;

        rw.commit()?;

        tracing::info!(
            "Cleaned session {}: removed {} tags, {} events, {} connectivity entries, {} operators, and 1 session",
            session_local_id,
            tags_count,
            events_count,
            connectivity_count,
            operators_count
        );

        Ok(())
    }

    /// Spawns the sync engine in a background task that runs indefinitely.
    /// Returns a JoinHandle that can be used to await completion or cancel the task.
    pub fn spawn_background_sync(mut self) -> tokio::task::JoinHandle<Result<(), Error>> {
        tokio::spawn(async move { self.start().await })
    }
    /// Returns the path to the local database file
    pub fn get_db_path(&self) -> &str {
        &self.db_local_path
    }

    /// Generates a unique ID using timestamp and table count to avoid race conditions
    pub fn generate_unique_id<T: ToInput>(&self) -> Result<u64, Error> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| Error::msg(format!("System time error: {}", e)))?
            .as_millis() as u64;

        // Use timestamp as base with table count as offset to ensure uniqueness
        let count = self.get_table_count::<T>()?;
        Ok(timestamp * 1000 + count)
    }

    /// Gets the number of items in a specific table type
    pub fn get_table_count<T: ToInput>(&self) -> Result<u64, Error> {
        let r = self.database.r_transaction()?;
        let count = r.len().primary::<T>();
        match count {
            Ok(count) => Ok(count),
            Err(e) => Err(e.into()),
        }
    }

    /// Removes multiple items from the local database
    pub fn remove_items<T: ToInput>(&mut self, items: Vec<T>) -> Result<(), Error> {
        let rw = self.database.rw_transaction();
        match rw {
            Ok(rw) => {
                for item in items {
                    rw.remove(item)?;
                }
                match rw.commit() {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Failed to commit items to database: {}", e);
                        Err(e.into())
                    }
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Inserts or updates multiple items in the local database
    pub fn upsert_items<T: ToInput>(&mut self, items: Vec<T>) -> Result<(), Error> {
        let rw = self.database.rw_transaction()?;
        for item in items {
            rw.upsert(item)?;
        }
        rw.commit()?;
        Ok(())
    }

    /// Updates all descendants of a session with the new remote session ID
    fn update_session_descendants(
        &mut self,
        session_local_id: &str,
        new_remote_session_id: i64,
    ) -> Result<(), Error> {
        // Update connectivity entries
        self.update_connectivity_session_id(session_local_id, new_remote_session_id)?;

        // Update events that belong to this session
        self.update_events_session_id(session_local_id, new_remote_session_id)?;

        // Update operators that belong to this session
        self.update_operators_session_id(session_local_id, new_remote_session_id)?;

        tracing::info!(
            "Updated descendants for session {} with remote ID {}",
            session_local_id,
            new_remote_session_id
        );
        Ok(())
    }

    /// Updates connectivity entries to reference the new remote session ID
    fn update_connectivity_session_id(
        &mut self,
        session_local_id: &str,
        new_remote_session_id: i64,
    ) -> Result<(), Error> {
        let r = self.database.r_transaction()?;

        // Find all connectivity entries that reference this session's local ID
        let mut connectivity_to_update = Vec::new();
        for raw_connectivity in r.scan().primary::<ConnectivityLocal>()?.all()? {
            if let Ok(mut connectivity) = raw_connectivity {
                if connectivity.ancestor_id_local.as_deref() == Some(session_local_id) {
                    // Validate: if session_id is already set, ensure it matches
                    if connectivity.session_id.is_some()
                        && connectivity.session_id != Some(new_remote_session_id)
                    {
                        tracing::warn!(
                            "Connectivity {} has conflicting session_id {:?} vs expected {}",
                            connectivity.id_local.as_deref().unwrap_or("unknown"),
                            connectivity.session_id,
                            new_remote_session_id
                        );
                        continue; // Skip this entry to prevent wrong linkage
                    }

                    // Convert to hybrid connectivity: keep device_id and add session_id
                    connectivity.session_id = Some(new_remote_session_id);
                    // Ensure device_id is set if not already present
                    if connectivity.device_id.is_none() {
                        // This should not happen in v2, but handle gracefully
                        tracing::warn!(
                            "Connectivity {} missing device_id, this may cause RLS issues",
                            connectivity.id_local.as_deref().unwrap_or("unknown")
                        );
                    }
                    // Keep ancestor_id_local as metadata showing original relationship
                    connectivity_to_update.push(connectivity);
                }
            }
        }

        drop(r); // Close read transaction before opening write transaction

        if !connectivity_to_update.is_empty() {
            let count = connectivity_to_update.len();
            self.upsert_items(connectivity_to_update)?;
            tracing::debug!(
                "Updated {} connectivity entries for session {}",
                count,
                session_local_id
            );
        }

        Ok(())
    }

    /// Updates events to reference the new remote session ID
    fn update_events_session_id(
        &mut self,
        session_local_id: &str,
        new_remote_session_id: i64,
    ) -> Result<(), Error> {
        let r = self.database.r_transaction()?;

        // Find all events that reference this session's local ID
        let mut events_to_update = Vec::new();
        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(mut event) = raw_event {
                if event.ancestor_id_local.as_deref() == Some(session_local_id) {
                    // Validate: if session_id is already set, ensure it matches
                    if let Some(existing_session_id) = event.session_id {
                        if existing_session_id != new_remote_session_id {
                            tracing::warn!(
                                "Event {} has conflicting session_id {} vs expected {}",
                                event.id_local.as_deref().unwrap_or("unknown"),
                                existing_session_id,
                                new_remote_session_id
                            );
                            continue; // Skip this entry to prevent wrong linkage
                        }
                    }

                    event.session_id = Some(new_remote_session_id);
                    // Keep ancestor_id_local as metadata showing original relationship
                    events_to_update.push(event);
                }
            }
        }

        drop(r); // Close read transaction before opening write transaction

        if !events_to_update.is_empty() {
            let count = events_to_update.len();
            self.upsert_items(events_to_update)?;
            tracing::debug!("Updated {} events for session {}", count, session_local_id);
        }

        Ok(())
    }

    /// Updates all descendants of an event with the new remote event ID
    fn update_event_descendants(
        &mut self,
        event_local_id: &str,
        new_remote_event_id: i64,
    ) -> Result<(), Error> {
        // Update tags that belong to this event
        self.update_tags_event_id(event_local_id, new_remote_event_id)?;

        tracing::info!(
            "Updated descendants for event {} with remote ID {}",
            event_local_id,
            new_remote_event_id
        );
        Ok(())
    }

    /// Updates tags to reference the new remote event ID
    fn update_tags_event_id(
        &mut self,
        event_local_id: &str,
        new_remote_event_id: i64,
    ) -> Result<(), Error> {
        let r = self.database.r_transaction()?;

        // Find all tags that reference this event's local ID
        let mut tags_to_update = Vec::new();
        for raw_tag in r.scan().primary::<TagLocal>()?.all()? {
            if let Ok(mut tag) = raw_tag {
                if tag.ancestor_id_local.as_deref() == Some(event_local_id) {
                    // Validate: if event_id is already set, ensure it matches
                    if tag.event_id != 0 && tag.event_id != new_remote_event_id {
                        tracing::warn!(
                            "Tag {} has conflicting event_id {} vs expected {}",
                            tag.id_local.as_deref().unwrap_or("unknown"),
                            tag.event_id,
                            new_remote_event_id
                        );
                        continue; // Skip this entry to prevent wrong linkage
                    }

                    tag.event_id = new_remote_event_id;
                    // Keep ancestor_id_local as metadata showing original relationship
                    tags_to_update.push(tag);
                }
            }
        }

        drop(r); // Close read transaction before opening write transaction

        if !tags_to_update.is_empty() {
            let count = tags_to_update.len();
            self.upsert_items(tags_to_update)?;
            tracing::debug!("Updated {} tags for event {}", count, event_local_id);
        }

        Ok(())
    }

    /// Updates operators to reference the new remote session ID
    fn update_operators_session_id(
        &mut self,
        session_local_id: &str,
        new_remote_session_id: i64,
    ) -> Result<(), Error> {
        let r = self.database.r_transaction()?;

        // Find all operators that reference this session's local ID
        let mut operators_to_update = Vec::new();
        for raw_operator in r.scan().primary::<data::v2::OperatorLocal>()?.all()? {
            if let Ok(mut operator) = raw_operator {
                if operator.ancestor_id_local.as_deref() == Some(session_local_id) {
                    // Validate: if session_id is already set, ensure it matches
                    if let Some(existing_session_id) = operator.session_id {
                        if existing_session_id != new_remote_session_id {
                            tracing::warn!(
                                "Operator {} has conflicting session_id {} vs expected {}",
                                operator.id_local.as_deref().unwrap_or("unknown"),
                                existing_session_id,
                                new_remote_session_id
                            );
                            continue; // Skip this entry to prevent wrong linkage
                        }
                    }

                    operator.session_id = Some(new_remote_session_id);
                    // Keep ancestor_id_local as metadata showing original relationship
                    operators_to_update.push(operator);
                }
            }
        }

        drop(r); // Close read transaction before opening write transaction

        if !operators_to_update.is_empty() {
            let count = operators_to_update.len();
            self.upsert_items(operators_to_update)?;
            tracing::debug!(
                "Updated {} operators for session {}",
                count,
                session_local_id
            );
        }

        Ok(())
    }

    /// Validates that a session exists in local database with given local_id and remote_id
    fn validate_session_exists(&self, local_id: &str, remote_id: i64) -> Result<bool, Error> {
        let r = self.database.r_transaction()?;

        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id_local.as_deref() == Some(local_id) && session.id == Some(remote_id) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Validates that an event exists in local database with given local_id and remote_id
    fn validate_event_exists(&self, local_id: &str, remote_id: i64) -> Result<bool, Error> {
        let r = self.database.r_transaction()?;

        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(event) = raw_event {
                if event.id_local.as_deref() == Some(local_id) && event.id == Some(remote_id) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Log information about each table in the local database
    /// Displays table name, count, and all rows for each table
    pub fn log(&self) -> Result<(), Error> {
        println!("=== Database Tables Log ===");

        // Log SessionLocal table
        self.log_table::<SessionLocal>("SessionLocal")?;

        // Log EventLocal table
        self.log_table::<EventLocal>("EventLocal")?;

        // Log TagLocal table
        self.log_table::<TagLocal>("TagLocal")?;

        // Log v1 ConnectivityLocal table
        self.log_table::<data::v1::ConnectivityLocal>("ConnectivityLocal (v1)")?;

        // Log v2 ConnectivityLocal table
        self.log_table::<data::v2::ConnectivityLocal>("ConnectivityLocal (v2)")?;

        // Log Operator table
        self.log_table::<data::v2::OperatorLocal>("OperatorLocal")?;

        println!("=== End Database Tables Log ===");
        Ok(())
    }

    /// Helper method to log a specific table
    fn log_table<T: ToInput + std::fmt::Debug>(&self, table_name: &str) -> Result<(), Error> {
        let r = self.database.r_transaction()?;
        let count = r.len().primary::<T>().unwrap_or(0);

        println!("\n--- Table: {} ---", table_name);
        println!("Count: {}", count);

        if count > 0 {
            println!("Rows:");
            let mut row_num = 1;
            for raw_item in r.scan().primary::<T>()?.all()? {
                match raw_item {
                    Ok(item) => {
                        println!("  {}: {:?}", row_num, item);
                        row_num += 1;
                    }
                    Err(e) => {
                        println!("  Error reading row {}: {:?}", row_num, e);
                        row_num += 1;
                    }
                }
            }
        } else {
            println!("No rows found");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        db_client::DatabaseConfig,
        models::{AncestorLocal, MediaType, SessionLocal, TagObservationType},
    };

    use tempfile::tempdir;

    fn setup_test_env() {
        dotenv::dotenv().ok();

        // Check for required environment variables and panic if missing
        let missing_vars = vec![
            (
                "SCOUT_DEVICE_API_KEY",
                std::env::var("SCOUT_DEVICE_API_KEY").is_err(),
            ),
            (
                "SCOUT_DATABASE_REST_URL",
                std::env::var("SCOUT_DATABASE_REST_URL").is_err(),
            ),
            ("SCOUT_DEVICE_ID", std::env::var("SCOUT_DEVICE_ID").is_err()),
            ("SCOUT_HERD_ID", std::env::var("SCOUT_HERD_ID").is_err()),
        ];

        let missing: Vec<&str> = missing_vars
            .into_iter()
            .filter(|(_, is_missing)| *is_missing)
            .map(|(name, _)| name)
            .collect();

        if !missing.is_empty() {
            panic!(
                "❌ Missing required environment variables: {}. Please check your .env file.",
                missing.join(", ")
            );
        }
    }

    fn create_test_sync_engine() -> Result<SyncEngine> {
        setup_test_env();

        let temp_dir = tempdir()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir
            .path()
            .join(format!("test_{}.db", timestamp))
            .to_string_lossy()
            .to_string();
        let database_config = DatabaseConfig::from_env()
            .map_err(|e| Error::msg(format!("System time error: {}", e)))?;
        let scout_client = ScoutClient::new(database_config);
        let sync_engine = SyncEngine::new(scout_client, db_path, None, None, false)?;

        // Initialize database with a simple transaction to ensure it's properly set up
        {
            let rw = sync_engine.database.rw_transaction()?;
            rw.commit()?;
        }

        Ok(sync_engine)
    }

    async fn create_test_sync_engine_with_identification() -> Result<SyncEngine> {
        setup_test_env();

        // Require API key - tests should fail if not provided
        let _api_key = std::env::var("SCOUT_DEVICE_API_KEY")
            .expect("SCOUT_DEVICE_API_KEY environment variable is required for sync tests");

        let temp_dir = tempdir()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir
            .path()
            .join(format!("test_{}.db", timestamp))
            .to_string_lossy()
            .to_string();

        // Create and identify scout client - MUST succeed for test to be valid
        let config_db = DatabaseConfig::from_env()?;
        let mut scout_client = ScoutClient::new(config_db);
        scout_client.identify().await.expect(
            "Client identification failed - check SCOUT_DEVICE_API_KEY and database connection",
        );

        let sync_engine = SyncEngine::new(scout_client, db_path, None, None, false)?;

        // Initialize database with a simple transaction to ensure it's properly set up
        {
            let rw = sync_engine.database.rw_transaction()?;
            rw.commit()?;
        }

        Ok(sync_engine)
    }

    #[tokio::test]
    async fn test_upsert_sessions_and_count() -> Result<()> {
        let mut sync_engine = create_test_sync_engine()?;

        // Check initial count is 0
        let initial_count = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(initial_count, 0);

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Create test sessions with proper data
        let mut session1 = SessionLocal::default();
        session1.set_id_local("test_session_1".to_string());
        session1.device_id = device_id;
        session1.timestamp_start = "2023-01-01T00:00:00Z".to_string();

        let mut session2 = SessionLocal::default();
        session2.set_id_local("test_session_2".to_string());
        session2.device_id = device_id;
        session2.timestamp_start = "2023-01-01T01:00:00Z".to_string();

        let mut session3 = SessionLocal::default();
        session3.set_id_local("test_session_3".to_string());
        session3.device_id = device_id;
        session3.timestamp_start = "2023-01-01T02:00:00Z".to_string();

        let sessions = vec![session1, session2, session3];

        // Upsert the sessions
        sync_engine.upsert_items(sessions)?;

        // Check that count is now 3
        let final_count = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(final_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_batch() -> Result<()> {
        let mut sync_engine = create_test_sync_engine()?;

        // Create a session with no remote ID (should go to insert batch)
        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        let mut session_1 = SessionLocal::default();
        session_1.set_id_local("test_session_1".to_string());
        session_1.device_id = device_id;
        session_1.timestamp_start = "2023-01-01T00:00:00Z".to_string();
        session_1.software_version = "1.0.0".to_string();

        sync_engine.upsert_items::<SessionLocal>(vec![session_1.clone()])?;

        // Verify the session was actually saved
        let count = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(count, 1);

        let batch = sync_engine
            .get_batch::<SessionLocal>(EnumSyncAction::Upsert, EnumSyncAction::Insert)?;

        // The session has no remote ID (id is None), so it should go to insert batch
        assert_eq!(batch.insert.len(), 1);
        assert_eq!(batch.upsert.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_upsert_operations() -> Result<()> {
        let mut sync_engine = create_test_sync_engine()?;
        // Create two different sessions
        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        let mut session_1 = SessionLocal::default();
        session_1.set_id_local("multi_test_session_1".to_string());
        session_1.device_id = device_id;
        session_1.timestamp_start = "2023-01-01T00:00:00Z".to_string();

        let mut session_2 = SessionLocal::default();
        session_2.set_id_local("multi_test_session_2".to_string());
        session_2.device_id = device_id;
        session_2.timestamp_start = "2023-01-01T01:00:00Z".to_string();

        sync_engine.upsert_items(vec![session_1])?;
        let count_after_first = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(count_after_first, 1);

        // Upsert second session
        sync_engine.upsert_items(vec![session_2])?;
        let count_after_second = sync_engine.get_table_count::<SessionLocal>()?;
        // Count should be 2 since we have two different sessions
        assert_eq!(count_after_second, 2);
        Ok(())
    }

    #[tokio::test]
    async fn test_flush_sessions_without_remote() -> Result<()> {
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        // Create sessions without remote IDs (they should be inserted to remote)
        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        let mut session_1 = SessionLocal::default();
        session_1.set_id_local("flush_test_session_1".to_string());
        session_1.device_id = device_id;
        session_1.timestamp_start = "2023-01-01T10:00:00Z".to_string();
        session_1.software_version = "sync_unit_test_flush_sessions_without_remote_0".to_string();
        session_1.altitude_max = 100.0;
        session_1.altitude_min = 50.0;
        session_1.altitude_average = 75.0;
        session_1.velocity_max = 25.0;
        session_1.velocity_min = 10.0;
        session_1.velocity_average = 15.0;
        session_1.distance_total = 1000.0;
        session_1.distance_max_from_start = 500.0;

        let mut session_2 = SessionLocal::default();
        session_2.set_id_local("flush_test_session_2".to_string());
        session_2.device_id = device_id;
        session_2.timestamp_start = "2023-01-01T11:00:00Z".to_string();
        session_2.software_version = "sync_unit_test_flush_sessions_without_remote_1".to_string();
        session_2.altitude_max = 120.0;
        session_2.altitude_min = 60.0;
        session_2.altitude_average = 90.0;
        session_2.velocity_max = 30.0;
        session_2.velocity_min = 15.0;
        session_2.velocity_average = 20.0;
        session_2.distance_total = 1200.0;
        session_2.distance_max_from_start = 600.0;

        // Insert sessions locally (no remote ID yet)
        sync_engine.upsert_items(vec![session_1, session_2])?;

        // Verify sessions are in local database
        let count_before = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(count_before, 2);

        // Flush MUST succeed - test should fail if remote sync doesn't work
        println!("🚀 Starting session flush to remote...");
        let flush_result = sync_engine.flush().await;

        match &flush_result {
            Ok(_) => println!("✅ Session flush completed successfully!"),
            Err(e) => {
                println!("❌ Session flush failed: {}", e);
                panic!(
                    "Flush operation must succeed - check database connection and API key: {}",
                    e
                );
            }
        }

        flush_result?;

        // Verify sessions are still in database after successful sync
        let count_after = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(count_after, 2);

        // Verify ALL sessions received remote IDs from server
        let r = sync_engine.database.r_transaction()?;
        let mut sessions_with_remote_ids = 0;
        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id.is_some() {
                    sessions_with_remote_ids += 1;
                }
            }
        }

        // STRICT: All sessions must have remote IDs after successful flush
        assert_eq!(
            sessions_with_remote_ids, 2,
            "All sessions must have remote IDs after successful flush to remote database"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_flush_with_descendant_updates() -> Result<()> {
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Create a session without remote ID (will be inserted to remote)
        let mut session = SessionLocal::default();
        session.set_id_local("test_session_with_descendants".to_string());
        session.device_id = device_id;
        session.timestamp_start = "2023-01-01T10:00:00Z".to_string();
        session.software_version = "sync_unit_test_flush_with_descendant_updates_0".to_string();
        session.altitude_max = 100.0;
        session.altitude_min = 50.0;
        session.altitude_average = 75.0;
        session.velocity_max = 25.0;
        session.velocity_min = 10.0;
        session.velocity_average = 15.0;
        session.distance_total = 1000.0;
        session.distance_max_from_start = 500.0;

        // Create connectivity entry that references this session's local ID
        let mut connectivity = ConnectivityLocal::default();
        connectivity.set_id_local("test_connectivity_1".to_string());
        connectivity.session_id = None; // Use device-based connectivity for initial sync
        connectivity.device_id = Some(device_id); // Reference the actual device ID
        connectivity.set_ancestor_id_local("test_session_with_descendants".to_string());
        connectivity.timestamp_start = "2023-01-01T10:05:00Z".to_string();
        connectivity.signal = -70.0;
        connectivity.noise = -90.0;
        connectivity.altitude = 100.0;
        connectivity.heading = 0.0;
        connectivity.location = Some("POINT(-155.15393 19.754824)".to_string());
        connectivity.h14_index = "h14".to_string();
        connectivity.h13_index = "h13".to_string();
        connectivity.h12_index = "h12".to_string();
        connectivity.h11_index = "h11".to_string();

        // Create event that references this session's local ID
        let mut event = EventLocal::default();
        event.set_id_local("test_event_1".to_string());
        event.device_id = device_id;
        event.session_id = None; // Will be updated after session gets remote ID
        event.set_ancestor_id_local("test_session_with_descendants".to_string());
        event.timestamp_observation = "2023-01-01T10:10:00Z".to_string();
        event.message = Some("Test event".to_string());
        event.altitude = 100.0;
        event.heading = 0.0;
        event.media_type = MediaType::Image;

        // Insert all items locally
        sync_engine.upsert_items(vec![session])?;
        sync_engine.upsert_items(vec![connectivity])?;
        sync_engine.upsert_items(vec![event])?;

        // Verify initial state
        let initial_session_count = sync_engine.get_table_count::<SessionLocal>()?;
        let initial_connectivity_count = sync_engine.get_table_count::<ConnectivityLocal>()?;
        let initial_event_count = sync_engine.get_table_count::<EventLocal>()?;
        assert_eq!(initial_session_count, 1);
        assert_eq!(initial_connectivity_count, 1);
        assert_eq!(initial_event_count, 1);

        // Flush MUST succeed - test should fail if remote sync doesn't work
        println!("🚀 Starting descendant update flush to remote...");
        let flush_result = sync_engine.flush().await;

        match &flush_result {
            Ok(_) => println!("✅ Descendant update flush completed successfully!"),
            Err(e) => {
                println!("❌ Descendant update flush failed: {}", e);
                panic!(
                    "Flush operation must succeed - check database connection and API key: {}",
                    e
                );
            }
        }

        flush_result?;

        // Verify all items are still in database after successful sync
        let final_session_count = sync_engine.get_table_count::<SessionLocal>()?;
        let final_connectivity_count = sync_engine.get_table_count::<ConnectivityLocal>()?;
        let final_event_count = sync_engine.get_table_count::<EventLocal>()?;
        assert_eq!(final_session_count, 1);
        assert_eq!(final_connectivity_count, 1);
        assert_eq!(final_event_count, 1);

        // Verify that items received remote IDs and relationships were updated
        let r = sync_engine.database.r_transaction()?;

        // Session MUST have remote ID after successful flush
        let mut session_remote_id = None;
        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id_local.as_deref() == Some("test_session_with_descendants") {
                    session_remote_id = session.id;
                    break;
                }
            }
        }
        assert!(
            session_remote_id.is_some(),
            "Session must have remote ID after successful flush to remote database"
        );

        let session_id = session_remote_id.unwrap();

        // Verify connectivity entries reference the session's remote ID
        for raw_connectivity in r.scan().primary::<ConnectivityLocal>()?.all()? {
            if let Ok(connectivity) = raw_connectivity {
                if connectivity.ancestor_id_local.as_deref()
                    == Some("test_session_with_descendants")
                {
                    assert_eq!(
                        connectivity.device_id,
                        Some(device_id),
                        "Connectivity must reference the correct device ID"
                    );
                    assert_eq!(
                        connectivity.session_id,
                        Some(session_id),
                        "Connectivity must reference session's remote ID after flush (hybrid mode)"
                    );
                }
            }
        }

        // Verify events reference the session's remote ID
        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(event) = raw_event {
                if event.ancestor_id_local.as_deref() == Some("test_session_with_descendants") {
                    assert_eq!(
                        event.session_id,
                        Some(session_id),
                        "Event must reference session's remote ID after flush"
                    );
                }
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_completed_sessions() -> Result<()> {
        let mut sync_engine = create_test_sync_engine()?;

        // Create a completed session (with timestamp_end)
        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        let mut completed_session = SessionLocal::default();
        completed_session.set_id_local("completed_session".to_string());
        completed_session.id = Some(12345); // Has remote ID
        completed_session.device_id = device_id;
        completed_session.timestamp_start = "2023-01-01T10:00:00Z".to_string();
        completed_session.timestamp_end = Some("2023-01-01T11:00:00Z".to_string()); // Completed
        completed_session.software_version = "1.0.0".to_string();
        completed_session.altitude_max = 100.0;
        completed_session.altitude_min = 50.0;
        completed_session.altitude_average = 75.0;
        completed_session.velocity_max = 25.0;
        completed_session.velocity_min = 10.0;
        completed_session.velocity_average = 15.0;
        completed_session.distance_total = 1000.0;
        completed_session.distance_max_from_start = 500.0;

        // Create an incomplete session (no timestamp_end)
        let mut incomplete_session = SessionLocal::default();
        incomplete_session.set_id_local("incomplete_session".to_string());
        incomplete_session.id = Some(23456); // Has remote ID
        incomplete_session.device_id = device_id;
        incomplete_session.timestamp_start = "2023-01-01T12:00:00Z".to_string();
        // No timestamp_end - should not be cleaned
        incomplete_session.software_version = "1.0.0".to_string();
        incomplete_session.altitude_max = 120.0;
        incomplete_session.altitude_min = 60.0;
        incomplete_session.altitude_average = 90.0;
        incomplete_session.velocity_max = 30.0;
        incomplete_session.velocity_min = 15.0;
        incomplete_session.velocity_average = 22.0;
        incomplete_session.distance_total = 1200.0;
        incomplete_session.distance_max_from_start = 600.0;

        // Create descendants for completed session
        let mut completed_connectivity = ConnectivityLocal::default();
        completed_connectivity.set_id_local("completed_connectivity".to_string());
        completed_connectivity.id = Some(34567); // Has remote ID
        completed_connectivity.session_id = None; // Use device-based connectivity
        completed_connectivity.device_id = Some(device_id);
        completed_connectivity.set_ancestor_id_local("completed_session".to_string());
        completed_connectivity.timestamp_start = "2023-01-01T10:05:00Z".to_string();
        completed_connectivity.signal = -70.0;
        completed_connectivity.noise = -90.0;
        completed_connectivity.altitude = 100.0;
        completed_connectivity.heading = 0.0;
        completed_connectivity.location = Some("POINT(-155.15393 19.754824)".to_string());
        completed_connectivity.h14_index = "h14".to_string();
        completed_connectivity.h13_index = "h13".to_string();
        completed_connectivity.h12_index = "h12".to_string();
        completed_connectivity.h11_index = "h11".to_string();

        let mut completed_event = EventLocal::default();
        completed_event.set_id_local("completed_event".to_string());
        completed_event.id = Some(45678); // Has remote ID
        completed_event.device_id = 1;
        completed_event.session_id = Some(12345);
        completed_event.set_ancestor_id_local("completed_session".to_string());
        completed_event.timestamp_observation = "2023-01-01T10:15:00Z".to_string();
        completed_event.message = Some("Completed event".to_string());
        completed_event.altitude = 100.0;
        completed_event.heading = 0.0;
        completed_event.media_type = MediaType::Image;

        let mut completed_tag = TagLocal::default();
        completed_tag.set_id_local("completed_tag".to_string());
        completed_tag.id = Some(56789); // Has remote ID
        completed_tag.x = 100.0;
        completed_tag.y = 200.0;
        completed_tag.width = 50.0;
        completed_tag.height = 75.0;
        completed_tag.conf = 0.95;
        completed_tag.observation_type = crate::models::TagObservationType::Auto;
        completed_tag.event_id = 45678;
        completed_tag.set_ancestor_id_local("completed_event".to_string());
        completed_tag.class_name = "test_animal".to_string();

        let mut completed_operator = data::v2::OperatorLocal::default();
        completed_operator.set_id_local("completed_operator".to_string());
        completed_operator.id = Some(67890); // Has remote ID
        completed_operator.session_id = Some(12345);
        completed_operator.set_ancestor_id_local("completed_session".to_string());
        completed_operator.user_id = "2205a997-c2b5-469a-8efb-6348f67b86e6".to_string();
        completed_operator.action = "test_clean_action".to_string();
        completed_operator.timestamp = Some("2023-01-01T10:20:00Z".to_string());

        // Insert all entities
        sync_engine.upsert_items(vec![completed_session, incomplete_session])?;
        sync_engine.upsert_items(vec![completed_connectivity])?;
        sync_engine.upsert_items(vec![completed_event])?;
        sync_engine.upsert_items(vec![completed_tag])?;
        sync_engine.upsert_items(vec![completed_operator])?;

        // Verify initial state
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 2);
        assert_eq!(sync_engine.get_table_count::<ConnectivityLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<TagLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<data::v2::OperatorLocal>()?, 1);

        // Run clean operation
        sync_engine.clean().await?;

        // Verify completed session and descendants are removed
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 1); // Only incomplete remains
        assert_eq!(sync_engine.get_table_count::<ConnectivityLocal>()?, 0); // Removed
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 0); // Removed
        assert_eq!(sync_engine.get_table_count::<TagLocal>()?, 0); // Removed
        assert_eq!(sync_engine.get_table_count::<data::v2::OperatorLocal>()?, 0); // Removed

        // Verify the remaining session is the incomplete one
        let r = sync_engine.database.r_transaction()?;
        let remaining_sessions: Vec<SessionLocal> = r
            .scan()
            .primary::<SessionLocal>()?
            .all()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(remaining_sessions.len(), 1);
        assert_eq!(
            remaining_sessions[0].id_local.as_deref(),
            Some("incomplete_session")
        );
        assert!(remaining_sessions[0].timestamp_end.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_flush_database_to_remote() -> Result<()> {
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        // Print diagnostic information
        println!("🔍 Testing full database flush to remote...");
        if let Ok(api_key) = std::env::var("SCOUT_DEVICE_API_KEY") {
            println!(
                "📡 Using API key: {}...",
                &api_key[..std::cmp::min(api_key.len(), 8)]
            );
        }
        if let Ok(db_url) = std::env::var("SCOUT_DATABASE_REST_URL") {
            println!("🗄️ Database URL: {}", db_url);
        }

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Create a complete hierarchy: Session -> Connectivity + Event -> Tag + Operator
        let mut session = SessionLocal::default();
        session.set_id_local("flush_test_session".to_string());
        session.device_id = device_id;
        session.timestamp_start = "2023-01-01T10:00:00Z".to_string();
        session.software_version = "test_flush_database_to_remote".to_string();
        session.altitude_max = 100.0;
        session.altitude_min = 50.0;
        session.altitude_average = 75.0;
        session.velocity_max = 25.0;
        session.velocity_min = 10.0;
        session.velocity_average = 15.0;
        session.distance_total = 1000.0;
        session.distance_max_from_start = 500.0;

        let mut connectivity = ConnectivityLocal::default();
        connectivity.set_id_local("flush_test_connectivity".to_string());
        connectivity.set_ancestor_id_local("flush_test_session".to_string());
        connectivity.session_id = None; // Use device-based connectivity for initial sync
        connectivity.device_id = Some(device_id); // Reference the actual device ID
        connectivity.timestamp_start = "2023-01-01T10:05:00Z".to_string();
        connectivity.signal = -70.0;
        connectivity.noise = -90.0;
        connectivity.altitude = 100.0;
        connectivity.heading = 0.0;
        connectivity.location = Some("POINT(-155.15393 19.754824)".to_string());
        connectivity.h14_index = "h14".to_string();
        connectivity.h13_index = "h13".to_string();
        connectivity.h12_index = "h12".to_string();
        connectivity.h11_index = "h11".to_string();

        let mut event = EventLocal::default();
        event.set_id_local("flush_test_event".to_string());
        event.device_id = device_id;
        event.session_id = None; // Will be updated after session sync
        event.set_ancestor_id_local("flush_test_session".to_string());
        event.timestamp_observation = "2023-01-01T10:10:00Z".to_string();
        event.message = Some("Test flush event".to_string());
        event.altitude = 100.0;
        event.heading = 0.0;
        event.media_type = MediaType::Image;

        let mut tag = TagLocal::default();
        tag.set_id_local("flush_test_tag".to_string());
        tag.event_id = 0; // Will be updated after event sync
        tag.set_ancestor_id_local("flush_test_event".to_string());
        tag.class_name = "test_flush_tag".to_string();
        tag.conf = 0.95;
        tag.observation_type = TagObservationType::Manual;

        let mut operator = data::v2::OperatorLocal::default();
        operator.set_id_local("flush_test_operator".to_string());
        operator.session_id = None; // Will be updated after session sync
        operator.set_ancestor_id_local("flush_test_session".to_string());
        operator.user_id = "2205a997-c2b5-469a-8efb-6348f67b86e6".to_string(); // Real user ID
        operator.action = "test_flush_action".to_string();
        operator.timestamp = Some("2023-01-01T10:15:00Z".to_string());

        // Insert all items locally
        sync_engine.upsert_items(vec![session])?;
        sync_engine.upsert_items(vec![connectivity])?;
        sync_engine.upsert_items(vec![event])?;
        sync_engine.upsert_items(vec![tag])?;
        sync_engine.upsert_items(vec![operator])?;

        // Verify initial counts
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<ConnectivityLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<TagLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<data::v2::OperatorLocal>()?, 1);

        // Perform full database flush to remote - MUST succeed
        println!("🚀 Starting full database flush...");
        let flush_result = sync_engine.flush().await;

        match &flush_result {
            Ok(_) => println!("✅ Flush completed successfully!"),
            Err(e) => {
                println!("❌ Flush failed with error: {}", e);
                println!(
                    "💡 This indicates the test is correctly trying to sync to remote database"
                );
                println!("🔧 Check: 1) Valid SCOUT_DEVICE_API_KEY 2) Database permissions 3) RLS policies");
                panic!(
                    "Full database flush must succeed - check database connection and API key: {}",
                    e
                );
            }
        }

        flush_result?;

        // Verify all items are still in database after successful sync
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<ConnectivityLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<TagLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<data::v2::OperatorLocal>()?, 1);

        // Verify the hierarchical sync worked correctly
        let r = sync_engine.database.r_transaction()?;

        // Session MUST have remote ID after successful flush
        let mut session_remote_id = None;
        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id_local.as_deref() == Some("flush_test_session") {
                    session_remote_id = session.id;
                    break;
                }
            }
        }

        let session_id = session_remote_id
            .expect("Session must have remote ID after successful flush to remote database");

        // Verify connectivity references session remote ID
        // Verify connectivity was properly linked to both device and session (hybrid)
        for raw_connectivity in r.scan().primary::<ConnectivityLocal>()?.all()? {
            if let Ok(connectivity) = raw_connectivity {
                if connectivity.id_local.as_deref() == Some("flush_test_connectivity") {
                    assert_eq!(
                        connectivity.device_id,
                        Some(device_id),
                        "Connectivity must reference the correct device ID"
                    );
                    assert_eq!(
                        connectivity.session_id,
                        Some(session_id),
                        "Connectivity must reference session's remote ID after session sync"
                    );
                }
            }
        }

        // Verify event references session remote ID and has remote ID
        let mut event_remote_id = None;
        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(event) = raw_event {
                if event.id_local.as_deref() == Some("flush_test_event") {
                    assert_eq!(
                        event.session_id,
                        Some(session_id),
                        "Event must reference session's remote ID after flush"
                    );
                    event_remote_id = event.id;
                    break;
                }
            }
        }

        let event_id = event_remote_id
            .expect("Event must have remote ID after successful flush to remote database");

        // Verify tag references event remote ID and has remote ID
        for raw_tag in r.scan().primary::<TagLocal>()?.all()? {
            if let Ok(tag) = raw_tag {
                if tag.id_local.as_deref() == Some("flush_test_tag") {
                    assert_eq!(
                        tag.event_id, event_id,
                        "Tag must reference event's remote ID after flush"
                    );
                    assert!(
                        tag.id.is_some(),
                        "Tag must have remote ID after successful flush"
                    );
                }
            }
        }

        // Verify operator references session remote ID and has remote ID
        for raw_operator in r.scan().primary::<data::v2::OperatorLocal>()?.all()? {
            if let Ok(operator) = raw_operator {
                if operator.id_local.as_deref() == Some("flush_test_operator") {
                    assert_eq!(
                        operator.session_id,
                        Some(session_id),
                        "Operator must reference session's remote ID after flush"
                    );
                    assert!(
                        operator.id.is_some(),
                        "Operator must have remote ID after successful flush"
                    );
                }
            }
        }

        println!("✅ Full database flush to remote completed successfully!");
        println!("✅ Session synced with remote ID: {}", session_id);
        println!("✅ Event synced with remote ID: {}", event_id);
        println!("✅ Operator synced and linked to session!");
        println!("✅ All relationships updated correctly!");

        Ok(())
    }

    async fn create_test_sync_engine_with_invalid_credentials() -> Result<SyncEngine> {
        let temp_dir = tempdir()?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let db_path = temp_dir
            .path()
            .join(format!("test_{}.db", timestamp))
            .to_string_lossy()
            .to_string();

        // Create client with invalid credentials - this should fail
        let invalid_config = DatabaseConfig {
            rest_url: "https://invalid.supabase.co/rest/v1".to_string(),
            scout_api_key: "invalid_api_key_12345".to_string(),
            supabase_api_key: "invalid_supabase_key".to_string(),
        };
        let mut scout_client = ScoutClient::new(invalid_config);
        scout_client.identify().await?; // This should fail

        let sync_engine = SyncEngine::new(scout_client, db_path, None, None, false)?;

        // Initialize database with a simple transaction to ensure it's properly set up
        {
            let rw = sync_engine.database.rw_transaction()?;
            rw.commit()?;
        }

        Ok(sync_engine)
    }

    #[tokio::test]
    async fn test_sync_requires_valid_credentials() -> Result<()> {
        println!("🔐 Testing sync failure with invalid credentials...");

        let result = create_test_sync_engine_with_invalid_credentials().await;

        match result {
            Ok(_) => {
                panic!("Sync engine creation should fail with invalid credentials");
            }
            Err(e) => {
                println!("✅ Correctly failed with invalid credentials: {}", e);
                println!("💡 This confirms the sync engine is properly validating credentials");
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_session_lifecycle_insert_update_flush_sequence() -> Result<()> {
        println!(
            "🔄 Testing session lifecycle: insert -> update -> flush -> record another -> flush"
        );
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // PHASE 1: Insert first session
        let mut session1 = SessionLocal::default();
        session1.set_id_local("lifecycle_session_1".to_string());
        session1.device_id = device_id;
        session1.timestamp_start = "2023-01-01T10:00:00Z".to_string();
        session1.software_version = "test_session_lifecycle_v1".to_string();
        session1.altitude_max = 100.0;
        session1.altitude_min = 50.0;
        session1.altitude_average = 75.0;
        session1.velocity_max = 25.0;
        session1.velocity_min = 10.0;
        session1.velocity_average = 15.0;
        session1.distance_total = 1000.0;
        session1.distance_max_from_start = 500.0;

        sync_engine.upsert_items(vec![session1.clone()])?;
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 1);
        println!("✅ Phase 1: First session inserted locally");

        // PHASE 2: Update the same session with new data (e.g., session in progress)
        session1.altitude_max = 150.0; // Updated max altitude
        session1.distance_total = 1500.0; // Updated distance
        session1.timestamp_end = None; // Still in progress

        sync_engine.upsert_items(vec![session1.clone()])?;
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 1); // Still just 1 session
        println!("✅ Phase 2: Session updated with new data");

        // PHASE 3: Flush the session to remote
        println!("🚀 Phase 3: Flushing first session to remote...");
        sync_engine.flush().await?;

        // Verify session got remote ID
        let r = sync_engine.database.r_transaction()?;
        let mut session1_remote_id = None;
        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id_local.as_deref() == Some("lifecycle_session_1") {
                    session1_remote_id = session.id;
                    break;
                }
            }
        }
        assert!(
            session1_remote_id.is_some(),
            "First session must have remote ID after flush"
        );
        println!(
            "✅ Phase 3: First session flushed with remote ID: {:?}",
            session1_remote_id
        );

        // PHASE 4: Complete the first session
        session1.timestamp_end = Some("2023-01-01T11:30:00Z".to_string());
        session1.altitude_max = 175.0; // Final max altitude
        session1.distance_total = 2000.0; // Final distance

        sync_engine.upsert_items(vec![session1])?;
        println!("✅ Phase 4: First session marked as completed");

        // PHASE 5: Record a completely new session (simulating back-to-back usage)
        let mut session2 = SessionLocal::default();
        session2.set_id_local("lifecycle_session_2".to_string());
        session2.device_id = device_id;
        session2.timestamp_start = "2023-01-01T12:00:00Z".to_string();
        session2.software_version = "test_session_lifecycle_v2".to_string();
        session2.altitude_max = 200.0;
        session2.altitude_min = 80.0;
        session2.altitude_average = 140.0;
        session2.velocity_max = 35.0;
        session2.velocity_min = 20.0;
        session2.velocity_average = 25.0;
        session2.distance_total = 800.0;
        session2.distance_max_from_start = 400.0;

        sync_engine.upsert_items(vec![session2.clone()])?;
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 2); // Now have 2 sessions
        println!("✅ Phase 5: Second session inserted (back-to-back usage)");

        // PHASE 6: Add some events to second session before flushing
        let mut event_for_session2 = EventLocal::default();
        event_for_session2.set_id_local("lifecycle_event_session2".to_string());
        event_for_session2.device_id = device_id;
        event_for_session2.session_id = None; // Will be updated after session sync
        event_for_session2.set_ancestor_id_local("lifecycle_session_2".to_string());
        event_for_session2.timestamp_observation = "2023-01-01T12:15:00Z".to_string();
        event_for_session2.message = Some("Event during second session".to_string());
        event_for_session2.altitude = 150.0;
        event_for_session2.heading = 45.0;
        event_for_session2.media_type = MediaType::Video;

        sync_engine.upsert_items(vec![event_for_session2])?;
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 1);
        println!("✅ Phase 6: Event added to second session");

        // PHASE 7: Final flush of everything (simulating critical sync point)
        println!("🚀 Phase 7: Final flush of all data...");
        sync_engine.flush().await?;

        // Verify final state
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 2);
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 1);

        // Verify both sessions have remote IDs
        let r = sync_engine.database.r_transaction()?;
        let mut sessions_with_remote_ids = 0;
        let mut session2_remote_id = None;

        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id.is_some() {
                    sessions_with_remote_ids += 1;
                    if session.id_local.as_deref() == Some("lifecycle_session_2") {
                        session2_remote_id = session.id;
                    }
                }
            }
        }

        assert_eq!(
            sessions_with_remote_ids, 2,
            "Both sessions must have remote IDs"
        );
        assert!(
            session2_remote_id.is_some(),
            "Second session must have remote ID"
        );

        // Verify event references second session's remote ID
        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(event) = raw_event {
                if event.id_local.as_deref() == Some("lifecycle_event_session2") {
                    assert_eq!(
                        event.session_id, session2_remote_id,
                        "Event must reference second session's remote ID"
                    );
                    assert!(event.id.is_some(), "Event must have remote ID");
                }
            }
        }

        println!("✅ Phase 7: Final state verified - all data synced with relationships intact");
        println!("🎉 Session lifecycle test completed successfully!");

        Ok(())
    }

    #[tokio::test]
    async fn test_session_update_during_recording_with_periodic_flush() -> Result<()> {
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Start a new session
        let mut active_session = SessionLocal::default();
        active_session.set_id_local("live_recording_session".to_string());
        active_session.device_id = device_id;
        active_session.timestamp_start = "2023-01-01T14:00:00Z".to_string();
        active_session.software_version = "live_recording_test".to_string();
        active_session.altitude_max = 100.0;
        active_session.distance_total = 0.0;

        sync_engine.upsert_items(vec![active_session.clone()])?;

        // Update session during recording
        active_session.altitude_max = 120.0;
        active_session.distance_total = 300.0;
        sync_engine.upsert_items(vec![active_session.clone()])?;

        // Add connectivity data
        let mut connectivity = ConnectivityLocal::default();
        connectivity.set_id_local("live_conn_1".to_string());
        connectivity.set_ancestor_id_local("live_recording_session".to_string());
        connectivity.timestamp_start = "2023-01-01T14:10:00Z".to_string();
        connectivity.signal = -68.0;
        connectivity.altitude = 120.0;
        connectivity.location = Some("POINT(-155.15393 19.754824)".to_string());
        connectivity.h14_index = "h14_live1".to_string();
        connectivity.h13_index = "h13_live1".to_string();
        connectivity.h12_index = "h12_live1".to_string();
        connectivity.h11_index = "h11_live1".to_string();

        sync_engine.upsert_items(vec![connectivity])?;

        // Periodic flush during recording
        sync_engine.flush().await?;

        // Get session remote ID after flush
        let r = sync_engine.database.r_transaction()?;
        let mut session_remote_id = None;
        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id_local.as_deref() == Some("live_recording_session") {
                    session_remote_id = session.id;
                    assert!(
                        session.timestamp_end.is_none(),
                        "Session should still be active"
                    );
                    break;
                }
            }
        }
        session_remote_id.expect("Session must have remote ID");
        drop(r);

        // Continue recording and add event
        active_session.altitude_max = 140.0;
        active_session.distance_total = 600.0;
        sync_engine.upsert_items(vec![active_session.clone()])?;

        let mut live_event = EventLocal::default();
        live_event.set_id_local("live_event_1".to_string());
        live_event.device_id = device_id;
        live_event.set_ancestor_id_local("live_recording_session".to_string());
        live_event.timestamp_observation = "2023-01-01T14:20:00Z".to_string();
        live_event.message = Some("Live observation".to_string());
        live_event.altitude = 140.0;
        live_event.media_type = MediaType::Image;

        sync_engine.upsert_items(vec![live_event])?;

        // Final flush
        sync_engine.flush().await?;

        // Verify final state
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<ConnectivityLocal>()?, 1);
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 1);

        // Complete the session
        active_session.timestamp_end = Some("2023-01-01T14:30:00Z".to_string());
        sync_engine.upsert_items(vec![active_session])?;

        Ok(())
    }

    #[tokio::test]
    async fn test_field_workflow_multiple_sessions_with_strategic_flushing() -> Result<()> {
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Pre-work session
        let mut pre_work_session = SessionLocal::default();
        pre_work_session.set_id_local("pre_work_session".to_string());
        pre_work_session.device_id = device_id;
        pre_work_session.timestamp_start = "2023-01-01T06:00:00Z".to_string();
        pre_work_session.software_version = "field_workflow_test".to_string();
        pre_work_session.altitude_max = 50.0;
        pre_work_session.distance_total = 200.0;

        sync_engine.upsert_items(vec![pre_work_session.clone()])?;

        // Morning survey with event and connectivity
        let mut morning_survey = SessionLocal::default();
        morning_survey.set_id_local("morning_survey".to_string());
        morning_survey.device_id = device_id;
        morning_survey.timestamp_start = "2023-01-01T08:00:00Z".to_string();
        morning_survey.software_version = "field_workflow_test".to_string();
        morning_survey.altitude_max = 150.0;
        morning_survey.distance_total = 1200.0;

        sync_engine.upsert_items(vec![morning_survey.clone()])?;

        let mut survey_event = EventLocal::default();
        survey_event.set_id_local("survey_obs_1".to_string());
        survey_event.device_id = device_id;
        survey_event.set_ancestor_id_local("morning_survey".to_string());
        survey_event.timestamp_observation = "2023-01-01T08:30:00Z".to_string();
        survey_event.message = Some("Bird observation".to_string());
        survey_event.altitude = 120.0;
        survey_event.media_type = MediaType::Image;

        let mut connectivity = ConnectivityLocal::default();
        connectivity.set_id_local("survey_conn_1".to_string());
        connectivity.set_ancestor_id_local("morning_survey".to_string());
        connectivity.timestamp_start = "2023-01-01T08:15:00Z".to_string();
        connectivity.signal = -68.0;
        connectivity.altitude = 130.0;
        connectivity.location = Some("POINT(-155.15393 19.754824)".to_string());
        connectivity.h14_index = "h14_survey1".to_string();
        connectivity.h13_index = "h13_survey1".to_string();
        connectivity.h12_index = "h12_survey1".to_string();
        connectivity.h11_index = "h11_survey1".to_string();

        sync_engine.upsert_items(vec![survey_event])?;
        sync_engine.upsert_items(vec![connectivity])?;

        // Strategic flush
        sync_engine.flush().await?;

        // Continue with remote area session
        let mut remote_session = SessionLocal::default();
        remote_session.set_id_local("remote_area_session".to_string());
        remote_session.device_id = device_id;
        remote_session.timestamp_start = "2023-01-01T13:00:00Z".to_string();
        remote_session.software_version = "field_workflow_test".to_string();
        remote_session.altitude_max = 200.0;
        remote_session.distance_total = 2500.0;

        sync_engine.upsert_items(vec![remote_session])?;

        // Add two events to remote session
        let mut remote_event1 = EventLocal::default();
        remote_event1.set_id_local("remote_obs_1".to_string());
        remote_event1.device_id = device_id;
        remote_event1.set_ancestor_id_local("remote_area_session".to_string());
        remote_event1.timestamp_observation = "2023-01-01T13:30:00Z".to_string();
        remote_event1.message = Some("Wildlife in remote area".to_string());
        remote_event1.altitude = 200.0;
        remote_event1.media_type = MediaType::Video;

        let mut remote_event2 = EventLocal::default();
        remote_event2.set_id_local("remote_obs_2".to_string());
        remote_event2.device_id = device_id;
        remote_event2.set_ancestor_id_local("remote_area_session".to_string());
        remote_event2.timestamp_observation = "2023-01-01T14:15:00Z".to_string();
        remote_event2.message = Some("Rare species sighting".to_string());
        remote_event2.altitude = 195.0;
        remote_event2.media_type = MediaType::Image;

        sync_engine.upsert_items(vec![remote_event1, remote_event2])?;

        // End of day flush
        sync_engine.flush().await?;

        // Verify final state
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 3);
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 3);
        assert_eq!(sync_engine.get_table_count::<ConnectivityLocal>()?, 1);

        Ok(())
    }
    #[tokio::test]
    async fn test_upsert_same_session_id_no_duplicates() -> Result<()> {
        setup_test_env();
        let mut sync_engine = create_test_sync_engine()?;

        // Check initial count is 0
        let initial_count = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(initial_count, 0);

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Create a test session
        let mut session = SessionLocal::default();
        session.set_id_local("duplicate_test_session".to_string());
        session.device_id = device_id;
        session.timestamp_start = "2023-01-01T00:00:00Z".to_string();
        session.earthranger_url = Some("https://example.com/session1".to_string());

        // First upsert - should insert the session
        sync_engine.upsert_items(vec![session.clone()])?;
        let count_after_first = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(count_after_first, 1);

        // Create a modified version of the same session (same id_local but different data)
        let mut updated_session = session.clone();
        updated_session.earthranger_url = Some("https://example.com/updated_session".to_string());
        updated_session.timestamp_end = Some("2023-01-01T01:00:00Z".to_string());

        // Second upsert with same id_local - should update, not create duplicate
        sync_engine.upsert_items(vec![updated_session])?;
        let count_after_second = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(
            count_after_second, 1,
            "Session count should remain 1 after upserting same ID"
        );

        // Third upsert with the original session again - should still be 1
        sync_engine.upsert_items(vec![session])?;
        let count_after_third = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(
            count_after_third, 1,
            "Session count should remain 1 after upserting same ID again"
        );

        // Test with multiple sessions including duplicates in the same batch
        let mut session2 = SessionLocal::default();
        session2.set_id_local("batch_duplicate_test_session_2".to_string());
        session2.device_id = device_id;
        session2.timestamp_start = "2023-01-01T02:00:00Z".to_string();

        let mut session3 = SessionLocal::default();
        session3.set_id_local("batch_duplicate_test_session_3".to_string());
        session3.device_id = device_id;
        session3.timestamp_start = "2023-01-01T03:00:00Z".to_string();

        // Create duplicate of session2 with different data
        let mut session2_duplicate = session2.clone();
        session2_duplicate.earthranger_url =
            Some("https://example.com/duplicate_session2".to_string());

        // Upsert batch with original and duplicate
        sync_engine.upsert_items(vec![session2, session3, session2_duplicate])?;
        let final_count = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(
            final_count, 3,
            "Should have 3 unique sessions total (1 original + 2 new)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_clean_safety_mechanisms() -> Result<()> {
        setup_test_env();
        let mut sync_engine = create_test_sync_engine()?;

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Test Case 1: Complete session but no remote ID - should NOT be cleaned
        let mut complete_no_remote = SessionLocal::default();
        complete_no_remote.set_id_local("complete_no_remote".to_string());
        complete_no_remote.id = None; // No remote ID
        complete_no_remote.device_id = device_id;
        complete_no_remote.timestamp_start = "2023-01-01T10:00:00Z".to_string();
        complete_no_remote.timestamp_end = Some("2023-01-01T11:00:00Z".to_string());
        complete_no_remote.software_version = "1.0.0".to_string();
        complete_no_remote.altitude_max = 100.0;
        complete_no_remote.altitude_min = 50.0;
        complete_no_remote.altitude_average = 75.0;
        complete_no_remote.velocity_max = 25.0;
        complete_no_remote.velocity_min = 10.0;
        complete_no_remote.velocity_average = 15.0;
        complete_no_remote.distance_total = 1000.0;
        complete_no_remote.distance_max_from_start = 500.0;

        // Test Case 2: Complete session with remote ID but descendant lacks remote ID
        let mut complete_with_unsynced_descendant = SessionLocal::default();
        complete_with_unsynced_descendant.set_id_local("complete_with_unsynced".to_string());
        complete_with_unsynced_descendant.id = Some(12345); // Has remote ID
        complete_with_unsynced_descendant.device_id = device_id;
        complete_with_unsynced_descendant.timestamp_start = "2023-01-01T12:00:00Z".to_string();
        complete_with_unsynced_descendant.timestamp_end = Some("2023-01-01T13:00:00Z".to_string());
        complete_with_unsynced_descendant.software_version = "1.0.0".to_string();
        complete_with_unsynced_descendant.altitude_max = 120.0;
        complete_with_unsynced_descendant.altitude_min = 60.0;
        complete_with_unsynced_descendant.altitude_average = 90.0;
        complete_with_unsynced_descendant.velocity_max = 30.0;
        complete_with_unsynced_descendant.velocity_min = 15.0;
        complete_with_unsynced_descendant.velocity_average = 22.0;
        complete_with_unsynced_descendant.distance_total = 1200.0;
        complete_with_unsynced_descendant.distance_max_from_start = 600.0;

        // Create event with NO remote ID for the second session
        let mut unsynced_event = EventLocal::default();
        unsynced_event.set_id_local("unsynced_event".to_string());
        unsynced_event.id = None; // No remote ID - this should prevent cleaning
        unsynced_event.device_id = device_id;
        unsynced_event.session_id = Some(12345);
        unsynced_event.set_ancestor_id_local("complete_with_unsynced".to_string());
        unsynced_event.timestamp_observation = "2023-01-01T12:15:00Z".to_string();
        unsynced_event.message = Some("Unsynced event".to_string());
        unsynced_event.altitude = 100.0;
        unsynced_event.heading = 0.0;
        unsynced_event.media_type = MediaType::Image;

        // Test Case 3: Complete session with all descendants synced - SHOULD be cleaned
        let mut complete_fully_synced = SessionLocal::default();
        complete_fully_synced.set_id_local("complete_fully_synced".to_string());
        complete_fully_synced.id = Some(23456); // Has remote ID
        complete_fully_synced.device_id = device_id;
        complete_fully_synced.timestamp_start = "2023-01-01T14:00:00Z".to_string();
        complete_fully_synced.timestamp_end = Some("2023-01-01T15:00:00Z".to_string());
        complete_fully_synced.software_version = "1.0.0".to_string();
        complete_fully_synced.altitude_max = 150.0;
        complete_fully_synced.altitude_min = 80.0;
        complete_fully_synced.altitude_average = 115.0;
        complete_fully_synced.velocity_max = 35.0;
        complete_fully_synced.velocity_min = 20.0;
        complete_fully_synced.velocity_average = 27.0;
        complete_fully_synced.distance_total = 1500.0;
        complete_fully_synced.distance_max_from_start = 750.0;

        // Create fully synced descendants
        let mut synced_connectivity = ConnectivityLocal::default();
        synced_connectivity.set_id_local("synced_connectivity".to_string());
        synced_connectivity.id = Some(34567); // Has remote ID
        synced_connectivity.session_id = None;
        synced_connectivity.device_id = Some(device_id);
        synced_connectivity.set_ancestor_id_local("complete_fully_synced".to_string());
        synced_connectivity.timestamp_start = "2023-01-01T14:05:00Z".to_string();
        synced_connectivity.signal = -70.0;
        synced_connectivity.noise = -90.0;
        synced_connectivity.altitude = 100.0;
        synced_connectivity.heading = 0.0;
        synced_connectivity.location = Some("POINT(-155.15393 19.754824)".to_string());
        synced_connectivity.h14_index = "h14".to_string();
        synced_connectivity.h13_index = "h13".to_string();
        synced_connectivity.h12_index = "h12".to_string();
        synced_connectivity.h11_index = "h11".to_string();

        let mut synced_event = EventLocal::default();
        synced_event.set_id_local("synced_event".to_string());
        synced_event.id = Some(45678); // Has remote ID
        synced_event.device_id = device_id;
        synced_event.session_id = Some(23456);
        synced_event.set_ancestor_id_local("complete_fully_synced".to_string());
        synced_event.timestamp_observation = "2023-01-01T14:15:00Z".to_string();
        synced_event.message = Some("Synced event".to_string());
        synced_event.altitude = 100.0;
        synced_event.heading = 0.0;
        synced_event.media_type = MediaType::Image;

        // Insert all test data
        sync_engine.upsert_items(vec![
            complete_no_remote,
            complete_with_unsynced_descendant,
            complete_fully_synced,
        ])?;
        sync_engine.upsert_items(vec![unsynced_event, synced_event])?;
        sync_engine.upsert_items(vec![synced_connectivity])?;

        // Verify initial state
        assert_eq!(sync_engine.get_table_count::<SessionLocal>()?, 3);
        assert_eq!(sync_engine.get_table_count::<EventLocal>()?, 2);
        assert_eq!(sync_engine.get_table_count::<ConnectivityLocal>()?, 1);

        // Run clean operation
        sync_engine.clean().await?;

        // Verify results:
        // - complete_no_remote should NOT be cleaned (no remote ID)
        // - complete_with_unsynced should NOT be cleaned (descendant lacks remote ID)
        // - complete_fully_synced SHOULD be cleaned (all have remote IDs)
        assert_eq!(
            sync_engine.get_table_count::<SessionLocal>()?,
            2,
            "Should have 2 sessions remaining (2 that couldn't be cleaned)"
        );
        assert_eq!(
            sync_engine.get_table_count::<EventLocal>()?,
            1,
            "Should have 1 event remaining (unsynced_event)"
        );
        assert_eq!(
            sync_engine.get_table_count::<ConnectivityLocal>()?,
            0,
            "Synced connectivity should be cleaned with its session"
        );

        // Verify which sessions remain
        let r = sync_engine.database.r_transaction()?;
        let remaining_sessions: Vec<SessionLocal> = r
            .scan()
            .primary::<SessionLocal>()?
            .all()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let remaining_ids: std::collections::HashSet<&str> = remaining_sessions
            .iter()
            .filter_map(|s| s.id_local.as_deref())
            .collect();

        assert!(
            remaining_ids.contains("complete_no_remote"),
            "Session without remote ID should not be cleaned"
        );
        assert!(
            remaining_ids.contains("complete_with_unsynced"),
            "Session with unsynced descendants should not be cleaned"
        );
        assert!(
            !remaining_ids.contains("complete_fully_synced"),
            "Fully synced session should be cleaned"
        );

        // Verify which events remain
        let remaining_events: Vec<EventLocal> = r
            .scan()
            .primary::<EventLocal>()?
            .all()?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        assert_eq!(remaining_events.len(), 1);
        assert_eq!(
            remaining_events[0].id_local.as_deref(),
            Some("unsynced_event")
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_descendant_updates_for_late_arriving_children() -> Result<()> {
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        let device_id = std::env::var("SCOUT_DEVICE_ID")
            .expect("SCOUT_DEVICE_ID required")
            .parse()
            .expect("SCOUT_DEVICE_ID must be valid integer");

        // Step 1: Create and sync a session first (gets remote ID)
        let mut session = SessionLocal::default();
        session.set_id_local("session_synced_first".to_string());
        session.device_id = device_id;
        session.timestamp_start = "2023-01-01T10:00:00Z".to_string();
        session.software_version = "test_late_children_0".to_string();
        session.altitude_max = 100.0;
        session.altitude_min = 50.0;
        session.altitude_average = 75.0;
        session.velocity_max = 25.0;
        session.velocity_min = 10.0;
        session.velocity_average = 15.0;
        session.distance_total = 1000.0;
        session.distance_max_from_start = 500.0;

        sync_engine.upsert_items(vec![session])?;

        // Flush session first - it should get a remote ID
        sync_engine.flush_sessions().await?;

        // Verify session has remote ID
        let r = sync_engine.database.r_transaction()?;
        let mut session_remote_id = None;
        for raw_session in r.scan().primary::<SessionLocal>()?.all()? {
            if let Ok(session) = raw_session {
                if session.id_local.as_deref() == Some("session_synced_first") {
                    session_remote_id = session.id;
                    assert!(
                        session_remote_id.is_some(),
                        "Session must have remote ID after first flush"
                    );
                    break;
                }
            }
        }
        drop(r);

        let session_id = session_remote_id.unwrap();

        // Step 2: Now create connectivity records AFTER session has remote ID
        // This simulates the problem: connectivity created during flight after session sync
        let mut connectivity1 = ConnectivityLocal::default();
        connectivity1.set_id_local("late_connectivity_1".to_string());
        connectivity1.session_id = None; // This should get populated by our fix
        connectivity1.device_id = Some(device_id);
        connectivity1.set_ancestor_id_local("session_synced_first".to_string());
        connectivity1.timestamp_start = "2023-01-01T10:05:00Z".to_string();
        connectivity1.signal = -70.0;
        connectivity1.noise = -90.0;
        connectivity1.altitude = 100.0;
        connectivity1.heading = 0.0;
        connectivity1.location = Some("POINT(-155.15393 19.754824)".to_string());
        connectivity1.h14_index = "h14".to_string();
        connectivity1.h13_index = "h13".to_string();
        connectivity1.h12_index = "h12".to_string();
        connectivity1.h11_index = "h11".to_string();

        let mut connectivity2 = ConnectivityLocal::default();
        connectivity2.set_id_local("late_connectivity_2".to_string());
        connectivity2.session_id = None; // This should get populated by our fix
        connectivity2.device_id = Some(device_id);
        connectivity2.set_ancestor_id_local("session_synced_first".to_string());
        connectivity2.timestamp_start = "2023-01-01T10:10:00Z".to_string();
        connectivity2.signal = -75.0;
        connectivity2.noise = -95.0;
        connectivity2.altitude = 105.0;
        connectivity2.heading = 45.0;
        connectivity2.location = Some("POINT(-155.15400 19.754830)".to_string());
        connectivity2.h14_index = "h14".to_string();
        connectivity2.h13_index = "h13".to_string();
        connectivity2.h12_index = "h12".to_string();
        connectivity2.h11_index = "h11".to_string();

        // Insert connectivity records locally
        sync_engine.upsert_items(vec![connectivity1, connectivity2])?;

        // Verify they don't have session_id yet
        let r = sync_engine.database.r_transaction()?;
        for raw_connectivity in r.scan().primary::<ConnectivityLocal>()?.all()? {
            if let Ok(connectivity) = raw_connectivity {
                if connectivity.ancestor_id_local.as_deref() == Some("session_synced_first") {
                    assert_eq!(
                        connectivity.session_id, None,
                        "Connectivity should not have session_id before sync (this is the bug we're fixing)"
                    );
                }
            }
        }
        drop(r);

        // Step 3: Flush connectivity - our fix should populate session_id
        sync_engine.flush_connectivity().await?;

        // Step 4: Verify the fix worked - connectivity records should now have session_id
        let r = sync_engine.database.r_transaction()?;
        let mut connectivity_count_with_session_id = 0;
        for raw_connectivity in r.scan().primary::<ConnectivityLocal>()?.all()? {
            if let Ok(connectivity) = raw_connectivity {
                if connectivity.ancestor_id_local.as_deref() == Some("session_synced_first") {
                    assert_eq!(
                        connectivity.session_id,
                        Some(session_id),
                        "Connectivity must have session_id populated after our fix (connectivity: {})",
                        connectivity.id_local.as_deref().unwrap_or("unknown")
                    );
                    connectivity_count_with_session_id += 1;
                }
            }
        }
        drop(r);

        assert_eq!(
            connectivity_count_with_session_id, 2,
            "Both connectivity records should have session_id populated"
        );

        // Step 5: Test the same scenario with events
        let mut event = EventLocal::default();
        event.set_id_local("late_event_1".to_string());
        event.device_id = device_id;
        event.session_id = None; // Should get populated by our fix
        event.set_ancestor_id_local("session_synced_first".to_string());
        event.timestamp_observation = "2023-01-01T10:15:00Z".to_string();
        event.message = Some("Late arriving event".to_string());
        event.altitude = 100.0;
        event.heading = 0.0;
        event.media_type = MediaType::Image;

        sync_engine.upsert_items(vec![event])?;

        // Flush events - should populate session_id due to our fix
        sync_engine.flush_events().await?;

        // Verify event got session_id populated
        let r = sync_engine.database.r_transaction()?;
        for raw_event in r.scan().primary::<EventLocal>()?.all()? {
            if let Ok(event) = raw_event {
                if event.ancestor_id_local.as_deref() == Some("session_synced_first") {
                    assert_eq!(
                        event.session_id,
                        Some(session_id),
                        "Event must have session_id populated after our fix"
                    );
                }
            }
        }
        drop(r);

        // Step 6: Test the same scenario with operators
        let mut operator = data::v2::OperatorLocal::default();
        operator.set_id_local("late_operator_1".to_string());
        operator.session_id = None; // Should get populated by our fix
        operator.set_ancestor_id_local("session_synced_first".to_string());
        operator.user_id = "2205a997-c2b5-469a-8efb-6348f67b86e6".to_string(); // Real user ID
        operator.action = "late_test_action".to_string();
        operator.timestamp = Some("2023-01-01T10:20:00Z".to_string());

        sync_engine.upsert_items(vec![operator])?;

        // Flush operators - should populate session_id due to our fix
        sync_engine.flush_operators().await?;

        // Verify operator got session_id populated
        let r = sync_engine.database.r_transaction()?;
        for raw_operator in r.scan().primary::<data::v2::OperatorLocal>()?.all()? {
            if let Ok(operator) = raw_operator {
                if operator.ancestor_id_local.as_deref() == Some("session_synced_first") {
                    assert_eq!(
                        operator.session_id,
                        Some(session_id),
                        "Operator must have session_id populated after our fix"
                    );
                }
            }
        }
        drop(r);

        println!("✅ Test passed: Late arriving children get proper ancestor IDs populated");
        Ok(())
    }
}
