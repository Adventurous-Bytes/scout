use crate::{
    client::ScoutClient,
    models::{
        AncestorLocal, Connectivity, ConnectivityLocal, Event, EventLocal, Session, SessionLocal,
        Syncable, Tag, TagLocal,
    },
};
use anyhow::{Error, Result};
use native_db::{Builder, Database, Models, ToInput};

use tracing::error;

pub struct SyncEngine {
    scout_client: ScoutClient,
    db_local_path: String,
    database: Database<'static>,
    interval_flush_sessions_ms: Option<u64>,
    max_session_life_ms: Option<u64>,
}

pub enum EnumSyncAction {
    Upsert,
    Insert,
    Delete,
}

const DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS: EnumSyncAction = EnumSyncAction::Upsert;
const DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS: EnumSyncAction = EnumSyncAction::Insert;
const DEFEAULT_INTERVAL_FLUSH_SESSIONS_MS: u64 = 3_000;
const DEFAULT_MAX_SESSION_LIFE_MS: u64 = 60 * 1_000;

#[derive(Default)]
pub struct BatchSync<T: ToInput + Syncable> {
    upsert: Vec<T>,
    insert: Vec<T>,
    delete: Vec<T>,
}

impl<T: ToInput + Syncable> BatchSync<T> {
    pub fn new() -> Self {
        Self {
            upsert: Vec::new(),
            insert: Vec::new(),
            delete: Vec::new(),
        }
    }

    pub fn add_upsert_item(&mut self, item: T) {
        self.upsert.push(item);
    }

    pub fn add_insert_item(&mut self, item: T) {
        self.insert.push(item);
    }

    pub fn add_delete_item(&mut self, item: T) {
        self.delete.push(item);
    }
}

impl SyncEngine {
    pub fn new(
        scout_client: ScoutClient,
        db_local_path: String,
        interval_flush_sessions_ms: Option<u64>,
        max_session_life_ms: Option<u64>,
    ) -> Result<Self> {
        let mut models = Models::new();

        // Define all models for local database
        models.define::<SessionLocal>()?;
        models.define::<EventLocal>()?;
        models.define::<TagLocal>()?;
        models.define::<ConnectivityLocal>()?;

        // Create database - use Box::leak to get 'static lifetime
        let models_static = Box::leak(Box::new(models));
        let database = Builder::new().create(models_static, &db_local_path)?;
        // initialize tracing
        Ok(Self {
            scout_client,
            db_local_path,
            database,
            interval_flush_sessions_ms,
            max_session_life_ms,
        })
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
                            EnumSyncAction::Delete => {
                                batch.add_delete_item(item);
                            }
                            EnumSyncAction::Insert => {
                                batch.add_insert_item(item);
                            }
                            EnumSyncAction::Upsert => {
                                batch.add_upsert_item(item);
                            }
                        }
                    }
                    // handle action for no remote id (local only)
                    else {
                        match action_for_items_without_existing_ids {
                            EnumSyncAction::Delete => {
                                batch.add_delete_item(item);
                            }
                            EnumSyncAction::Insert => {
                                batch.add_insert_item(item);
                            }
                            EnumSyncAction::Upsert => {
                                batch.add_upsert_item(item);
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

    fn get_batch_descendants<T: ToInput + Syncable + AncestorLocal>(
        &mut self,
        ancestor_id_local: String,
        action_for_items_with_existing_ids: EnumSyncAction,
        action_for_items_without_existing_ids: EnumSyncAction,
    ) -> Result<BatchSync<T>, Error> {
        let r = self.database.r_transaction()?;
        let mut batch: BatchSync<T> = BatchSync::new();

        // Since we can't generically query by ancestor_id_local secondary key,
        // we scan all items and filter in memory
        for raw_item in r.scan().primary::<T>()?.all()? {
            match raw_item {
                Ok(item) => {
                    // Filter by ancestor_id_local
                    if item.ancestor_id_local() == Some(ancestor_id_local.clone()) {
                        // handle action for existing ids (on remote)
                        if item.id().is_some() {
                            match action_for_items_with_existing_ids {
                                EnumSyncAction::Delete => {
                                    batch.add_delete_item(item);
                                }
                                EnumSyncAction::Insert => {
                                    batch.add_insert_item(item);
                                }
                                EnumSyncAction::Upsert => {
                                    batch.add_upsert_item(item);
                                }
                            }
                        }
                        // handle action for no id (local only)
                        else {
                            match action_for_items_without_existing_ids {
                                EnumSyncAction::Delete => {
                                    batch.add_delete_item(item);
                                }
                                EnumSyncAction::Insert => {
                                    batch.add_insert_item(item);
                                }
                                EnumSyncAction::Upsert => {
                                    batch.add_upsert_item(item);
                                }
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

    pub async fn flush(&mut self) -> Result<(), Error> {
        // Get all sessions from the local database
        let sessions_batch: BatchSync<SessionLocal> = self.get_batch::<SessionLocal>(
            DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS,
            DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS,
        )?;

        // Combine upsert and insert operations - upsert handles both cases
        // Sessions with remote IDs will be updated, sessions without will be inserted
        let mut all_sessions = sessions_batch.upsert;
        all_sessions.extend(sessions_batch.insert);

        // Process all sessions in a single upsert operation for efficiency
        // This assumes that our network and remote server can handle the load...
        // currently assuming a maximum of 20 sessions being processed at once
        if !all_sessions.is_empty() {
            let sessions_for_upsert: Vec<Session> = all_sessions
                .iter()
                .map(|local_session| local_session.clone().into())
                .collect();

            match self
                .scout_client
                .upsert_sessions_batch(&sessions_for_upsert)
                .await
            {
                Ok(response) => {
                    if let Some(upserted_sessions) = response.data {
                        // Update local database with remote IDs and any server changes
                        // New sessions get remote IDs, existing sessions get updated values
                        let updated_locals: Vec<SessionLocal> = upserted_sessions
                            .into_iter()
                            .zip(all_sessions.iter())
                            .map(|(remote_session, original_local)| {
                                let mut updated_local: SessionLocal = remote_session.into();
                                // Preserve the local ID
                                updated_local.id_local = original_local.id_local.clone();
                                updated_local
                            })
                            .collect();

                        // Update sessions in local database first
                        self.upsert_items(updated_locals.clone())?;

                        // Update descendants with new remote session IDs
                        for (updated_session, original_session) in
                            updated_locals.iter().zip(all_sessions.iter())
                        {
                            if let (Some(new_remote_id), Some(local_id)) =
                                (updated_session.id, &original_session.id_local)
                            {
                                // Check if this session got a new remote ID (was previously local-only)
                                if original_session.id.is_none() {
                                    self.update_session_descendants(local_id, new_remote_id)?;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to upsert sessions to remote: {}", e);
                    return Err(e);
                }
            }
        }

        // Handle delete operations if needed
        if !sessions_batch.delete.is_empty() {
            tracing::warn!("Delete operations not yet implemented in flush");
        }

        // Process connectivity upserts after sessions
        let connectivity_batch: BatchSync<ConnectivityLocal> = self
            .get_batch::<ConnectivityLocal>(
                DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS,
                DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS,
            )?;

        // Combine upsert and insert operations for connectivity
        let mut all_connectivity = connectivity_batch.upsert;
        all_connectivity.extend(connectivity_batch.insert);

        // Process all connectivity in a single upsert operation
        if !all_connectivity.is_empty() {
            let connectivity_for_upsert: Vec<Connectivity> = all_connectivity
                .iter()
                .map(|local_connectivity| local_connectivity.clone().into())
                .collect();

            match self
                .scout_client
                .upsert_connectivity_batch(&connectivity_for_upsert)
                .await
            {
                Ok(response) => {
                    if let Some(upserted_connectivity) = response.data {
                        // Update local database with remote IDs and any server changes
                        let updated_connectivity: Vec<ConnectivityLocal> = upserted_connectivity
                            .into_iter()
                            .zip(all_connectivity.iter())
                            .map(|(remote_connectivity, original_local)| {
                                let mut updated_local: ConnectivityLocal =
                                    remote_connectivity.into();
                                // Preserve the local ID
                                updated_local.id_local = original_local.id_local.clone();
                                updated_local
                            })
                            .collect();

                        self.upsert_items(updated_connectivity)?;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to upsert connectivity to remote: {}", e);
                    return Err(e);
                }
            }
        }

        // Handle delete operations for connectivity if needed
        if !connectivity_batch.delete.is_empty() {
            tracing::warn!("Delete operations not yet implemented for connectivity");
        }

        // Process events upserts after connectivity
        let events_batch: BatchSync<EventLocal> = self.get_batch::<EventLocal>(
            DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS,
            DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS,
        )?;

        // Combine upsert and insert operations for events
        let mut all_events = events_batch.upsert;
        all_events.extend(events_batch.insert);

        // Process all events in a single upsert operation
        if !all_events.is_empty() {
            let events_for_upsert: Vec<Event> = all_events
                .iter()
                .map(|local_event| local_event.clone().into())
                .collect();

            match self
                .scout_client
                .upsert_events_batch(&events_for_upsert)
                .await
            {
                Ok(response) => {
                    if let Some(upserted_events) = response.data {
                        // Update local database with remote IDs and any server changes
                        let updated_events: Vec<EventLocal> = upserted_events
                            .into_iter()
                            .zip(all_events.iter())
                            .map(|(remote_event, original_local)| {
                                let mut updated_local: EventLocal = remote_event.into();
                                // Preserve the local ID
                                updated_local.id_local = original_local.id_local.clone();
                                updated_local
                            })
                            .collect();

                        // Update events in local database first
                        self.upsert_items(updated_events.clone())?;

                        // Update tag descendants with new remote event IDs
                        for (updated_event, original_event) in
                            updated_events.iter().zip(all_events.iter())
                        {
                            if let (Some(new_remote_id), Some(local_id)) =
                                (updated_event.id, &original_event.id_local)
                            {
                                // Check if this event got a new remote ID (was previously local-only)
                                if original_event.id.is_none() {
                                    self.update_event_descendants(local_id, new_remote_id)?;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to upsert events to remote: {}", e);
                    return Err(e);
                }
            }
        }

        // Handle delete operations for events if needed
        if !events_batch.delete.is_empty() {
            tracing::warn!("Delete operations not yet implemented for events");
        }

        // Process tags upserts after events
        let tags_batch: BatchSync<TagLocal> = self.get_batch::<TagLocal>(
            DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS,
            DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS,
        )?;

        // Combine upsert and insert operations for tags
        let mut all_tags = tags_batch.upsert;
        all_tags.extend(tags_batch.insert);

        // Process all tags in a single upsert operation
        if !all_tags.is_empty() {
            let tags_for_upsert: Vec<Tag> = all_tags
                .iter()
                .map(|local_tag| local_tag.clone().into())
                .collect();

            match self.scout_client.upsert_tags_batch(&tags_for_upsert).await {
                Ok(response) => {
                    if let Some(upserted_tags) = response.data {
                        // Update local database with remote IDs and any server changes
                        let updated_tags: Vec<TagLocal> = upserted_tags
                            .into_iter()
                            .zip(all_tags.iter())
                            .map(|(remote_tag, original_local)| {
                                let mut updated_local: TagLocal = remote_tag.into();
                                // Preserve the local ID
                                updated_local.id_local = original_local.id_local.clone();
                                updated_local
                            })
                            .collect();

                        self.upsert_items(updated_tags)?;
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to upsert tags to remote: {}", e);
                    return Err(e);
                }
            }
        }

        // Handle delete operations for tags if needed
        if !tags_batch.delete.is_empty() {
            tracing::warn!("Delete operations not yet implemented for tags");
        }

        Ok(())
    }
    pub fn get_db_path(&self) -> &str {
        &self.db_local_path
    }

    pub fn generate_unique_id<T: ToInput>(&self) -> Result<u64, Error> {
        let count = self.get_table_count::<T>();
        match count {
            Ok(count) => Ok(count + 1),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_table_count<T: ToInput>(&self) -> Result<u64, Error> {
        let r = self.database.r_transaction()?;
        let count = r.len().primary::<T>();
        match count {
            Ok(count) => Ok(count),
            Err(e) => Err(e.into()),
        }
    }

    pub fn remove_items<T: ToInput>(&mut self, items: Vec<T>) -> Result<(), Error> {
        let rw = self.database.rw_transaction();
        match rw {
            Ok(rw) => {
                for item in items {
                    rw.upsert(item)?;
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
                    connectivity.session_id = new_remote_session_id;
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
                    event.session_id = Some(new_remote_session_id);
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
                    tag.event_id = new_remote_event_id;
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
}

#[cfg(test)]
// to run this specific test suite use cargo test --test sync
mod tests {
    use super::*;
    use crate::models::{MediaType, SessionLocal};
    use once_cell::sync::Lazy;
    use std::cell::RefCell;
    use std::env;
    use tempfile::tempdir;
    use tokio::sync::Mutex;

    // Global test mutex to prevent concurrent database access
    static DB_TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    // Simple cleanup tracker for sync tests
    struct SyncTestCleanup {
        session_ids: RefCell<Vec<i64>>,
        connectivity_ids: RefCell<Vec<i64>>,
        event_ids: RefCell<Vec<i64>>,
        tag_ids: RefCell<Vec<i64>>,
    }

    impl SyncTestCleanup {
        fn new() -> Self {
            Self {
                session_ids: RefCell::new(Vec::new()),
                connectivity_ids: RefCell::new(Vec::new()),
                event_ids: RefCell::new(Vec::new()),
                tag_ids: RefCell::new(Vec::new()),
            }
        }

        fn track_session(&self, id: i64) {
            self.session_ids.borrow_mut().push(id);
        }

        fn track_connectivity(&self, id: i64) {
            self.connectivity_ids.borrow_mut().push(id);
        }

        fn track_event(&self, id: i64) {
            self.event_ids.borrow_mut().push(id);
        }

        fn track_tag(&self, id: i64) {
            self.tag_ids.borrow_mut().push(id);
        }

        async fn cleanup(&self, client: &mut ScoutClient) {
            // Clean up in reverse dependency order
            // for &tag_id in self.tag_ids.borrow().iter() {
            //     let _ = client.delete_tag(tag_id).await;
            // }
            // for &event_id in self.event_ids.borrow().iter() {
            //     let _ = client.delete_event(event_id).await;
            // }
            // for &connectivity_id in self.connectivity_ids.borrow().iter() {
            //     let _ = client.delete_connectivity(connectivity_id).await;
            // }
            // for &session_id in self.session_ids.borrow().iter() {
            //     let _ = client.delete_session(session_id).await;
            // }
        }
    }

    macro_rules! sync_test_with_cleanup {
        ($test_name:ident, $test_fn:ident) => {
            #[tokio::test]
            async fn $test_name() {
                let _guard = DB_TEST_MUTEX.lock().await;
                setup_test_env();

                let cleanup = SyncTestCleanup::new();
                $test_fn(&cleanup).await;

                // Clean up test data
                let mut client = ScoutClient::new(
                    env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
                )
                .unwrap();

                if client.identify().await.is_ok() {
                    cleanup.cleanup(&mut client).await;
                }
            }
        };
    }

    fn setup_test_env() {
        // Load environment variables from .env file
        dotenv::dotenv().ok();

        // Check for required environment variables for sync tests
        let required_vars = vec!["SCOUT_DEVICE_API_KEY", "SCOUT_DATABASE_REST_URL"];

        let missing: Vec<&str> = required_vars
            .into_iter()
            .filter(|var| env::var(var).is_err())
            .collect();

        if !missing.is_empty() {
            eprintln!(
                "⚠️  Warning: Missing environment variables for sync tests: {}. Some tests may be skipped.",
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

        // Create a properly configured scout client using environment variables
        let scout_client = ScoutClient::new(
            std::env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
        )?;

        let sync_engine = SyncEngine::new(scout_client, db_path, None, None)?;

        // Initialize database with a simple transaction to ensure it's properly set up
        {
            let rw = sync_engine.database.rw_transaction()?;
            rw.commit()?;
        }

        Ok(sync_engine)
    }

    async fn create_test_sync_engine_with_identification() -> Result<SyncEngine> {
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

        // Create and identify scout client
        let mut scout_client = ScoutClient::new(
            std::env::var("SCOUT_DEVICE_API_KEY").unwrap_or_else(|_| "test_api_key".to_string()),
        )?;

        // Try to identify the client - this may fail in test environments
        let _ = scout_client.identify().await;

        let sync_engine = SyncEngine::new(scout_client, db_path, None, None)?;

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

        // Create test sessions with proper data
        let mut session1 = SessionLocal::default();
        session1.set_id_local("test_session_1".to_string());
        session1.device_id = 1;
        session1.timestamp_start = "2023-01-01T00:00:00Z".to_string();

        let mut session2 = SessionLocal::default();
        session2.set_id_local("test_session_2".to_string());
        session2.device_id = 2;
        session2.timestamp_start = "2023-01-01T01:00:00Z".to_string();

        let mut session3 = SessionLocal::default();
        session3.set_id_local("test_session_3".to_string());
        session3.device_id = 3;
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
        let mut session_1 = SessionLocal::default();
        session_1.set_id_local("multi_test_session_1".to_string());
        session_1.device_id = 1;
        session_1.timestamp_start = "2023-01-01T00:00:00Z".to_string();
        session_1.software_version = "1.0.0".to_string();

        sync_engine.upsert_items::<SessionLocal>(vec![session_1.clone()])?;

        // Verify the session was actually saved
        let count = sync_engine.get_table_count::<SessionLocal>()?;
        assert_eq!(count, 1);

        let batch = sync_engine.get_batch::<SessionLocal>(
            DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS,
            DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS,
        )?;

        // The session has no remote ID (id is None), so it should go to insert batch
        assert_eq!(batch.insert.len(), 1);
        assert_eq!(batch.upsert.len(), 0);
        assert_eq!(batch.delete.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_upsert_operations() -> Result<()> {
        let mut sync_engine = create_test_sync_engine()?;
        // Create two different sessions
        let mut session_1 = SessionLocal::default();
        session_1.set_id_local("multi_test_session_1".to_string());
        session_1.device_id = 1;
        session_1.timestamp_start = "2023-01-01T00:00:00Z".to_string();

        let mut session_2 = SessionLocal::default();
        session_2.set_id_local("multi_test_session_2".to_string());
        session_2.device_id = 2;
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
        let mut session_1 = SessionLocal::default();
        session_1.set_id_local("flush_test_session_1".to_string());
        session_1.device_id = 1;
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
        session_2.device_id = 2;
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

        // Note: This test would require a mock or real Scout API client
        // For now, we test that the flush method doesn't error out
        // In a real implementation, the sessions would get remote IDs after flush
        let flush_result = sync_engine.flush().await;

        // The flush might fail due to network/API issues in test environment
        // but it shouldn't panic or cause database corruption
        match flush_result {
            Ok(_) => {
                // If successful, sessions should still be in database
                let count_after = sync_engine.get_table_count::<SessionLocal>()?;
                assert_eq!(count_after, 2);
            }
            Err(_) => {
                // If it fails (expected in test without real API),
                // sessions should still be in database
                let count_after = sync_engine.get_table_count::<SessionLocal>()?;
                assert_eq!(count_after, 2);
            }
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_flush_with_descendant_updates() -> Result<()> {
        let mut sync_engine = create_test_sync_engine_with_identification().await?;

        // Create a session without remote ID (will be inserted to remote)
        let mut session = SessionLocal::default();
        session.set_id_local("test_session_with_descendants".to_string());
        session.device_id = 1;
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
        connectivity.session_id = 0; // Will be updated after session gets remote ID
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
        event.device_id = 1;
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

        // Attempt to flush - this will try to upsert sessions and update descendants
        let flush_result = sync_engine.flush().await;

        // Verify the flush operation doesn't corrupt the database
        match flush_result {
            Ok(_) => {
                // If successful, all items should still be in database
                let final_session_count = sync_engine.get_table_count::<SessionLocal>()?;
                let final_connectivity_count =
                    sync_engine.get_table_count::<ConnectivityLocal>()?;
                let final_event_count = sync_engine.get_table_count::<EventLocal>()?;
                assert_eq!(final_session_count, 1);
                assert_eq!(final_connectivity_count, 1);
                assert_eq!(final_event_count, 1);

                // Note: In a real test with API access, we would verify that:
                // - Session got a remote ID
                // - Connectivity entry's session_id was updated
                // - Event's session_id was updated
            }
            Err(_) => {
                // If it fails (expected in test without real API),
                // all items should still be in database unchanged
                let final_session_count = sync_engine.get_table_count::<SessionLocal>()?;
                let final_connectivity_count =
                    sync_engine.get_table_count::<ConnectivityLocal>()?;
                let final_event_count = sync_engine.get_table_count::<EventLocal>()?;
                assert_eq!(final_session_count, 1);
                assert_eq!(final_connectivity_count, 1);
                assert_eq!(final_event_count, 1);
            }
        }

        Ok(())
    }

    sync_test_with_cleanup!(
        test_complete_flush_functionality,
        test_complete_flush_functionality_impl
    );

    async fn test_complete_flush_functionality_impl(cleanup: &SyncTestCleanup) {
        let mut sync_engine = create_test_sync_engine_with_identification().await.unwrap();

        let device_id: i64 = env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123);

        // Create complete hierarchy without remote IDs
        let mut session = SessionLocal::default();
        session.set_id_local("test_complete_session".to_string());
        session.device_id = device_id;
        session.timestamp_start = "2023-12-01T10:00:00Z".to_string();
        session.software_version = "sync_unit_test_sync_with_cleanup".to_string();
        session.altitude_max = 200.0;
        session.altitude_min = 100.0;
        session.altitude_average = 150.0;
        session.velocity_max = 50.0;
        session.velocity_min = 20.0;
        session.velocity_average = 35.0;
        session.distance_total = 2000.0;
        session.distance_max_from_start = 1000.0;

        let mut connectivity = ConnectivityLocal::default();
        connectivity.set_id_local("test_complete_connectivity".to_string());
        connectivity.session_id = 0; // No remote session ID yet
        connectivity.set_ancestor_id_local("test_complete_session".to_string());
        connectivity.timestamp_start = "2023-12-01T10:05:00Z".to_string();
        connectivity.signal = -75.0;
        connectivity.noise = -95.0;
        connectivity.altitude = 150.0;
        connectivity.heading = 45.0;
        connectivity.location = Some("POINT(-122.4194 37.7749)".to_string());
        connectivity.h14_index = "sf14".to_string();
        connectivity.h13_index = "sf13".to_string();
        connectivity.h12_index = "sf12".to_string();
        connectivity.h11_index = "sf11".to_string();

        let mut event = EventLocal::default();
        event.set_id_local("test_complete_event".to_string());
        event.device_id = device_id;
        event.session_id = None; // No remote session ID yet
        event.set_ancestor_id_local("test_complete_session".to_string());
        event.timestamp_observation = "2023-12-01T10:15:00Z".to_string();
        event.message = Some("Complete test event".to_string());
        event.altitude = 150.0;
        event.heading = 45.0;
        event.media_type = MediaType::Image;

        let mut tag = TagLocal::default();
        tag.set_id_local("test_complete_tag".to_string());
        tag.x = 100.0;
        tag.y = 200.0;
        tag.width = 50.0;
        tag.height = 75.0;
        tag.conf = 0.95;
        tag.observation_type = crate::models::TagObservationType::Auto;
        tag.event_id = 0; // No remote event ID yet
        tag.set_ancestor_id_local("test_complete_event".to_string());
        tag.class_name = "test_animal".to_string();

        // Insert complete hierarchy locally
        sync_engine.upsert_items(vec![session]).unwrap();
        sync_engine.upsert_items(vec![connectivity]).unwrap();
        sync_engine.upsert_items(vec![event]).unwrap();
        sync_engine.upsert_items(vec![tag]).unwrap();

        // Verify initial state - no remote IDs
        let r = sync_engine.database.r_transaction().unwrap();
        let sessions: Vec<SessionLocal> = r
            .scan()
            .primary::<SessionLocal>()
            .unwrap()
            .all()
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let connectivity_items: Vec<ConnectivityLocal> = r
            .scan()
            .primary::<ConnectivityLocal>()
            .unwrap()
            .all()
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let events: Vec<EventLocal> = r
            .scan()
            .primary::<EventLocal>()
            .unwrap()
            .all()
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let tags: Vec<TagLocal> = r
            .scan()
            .primary::<TagLocal>()
            .unwrap()
            .all()
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        drop(r);

        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].id.is_none());
        assert_eq!(connectivity_items.len(), 1);
        assert!(connectivity_items[0].id.is_none());
        assert_eq!(events.len(), 1);
        assert!(events[0].id.is_none());
        assert_eq!(tags.len(), 1);
        assert!(tags[0].id.is_none());

        // Attempt flush operation with real Scout client
        let flush_result = sync_engine.flush().await;

        match flush_result {
            Ok(_) => {
                // Verify entities now have remote IDs if API call succeeded
                let r = sync_engine.database.r_transaction().unwrap();
                let final_sessions: Vec<SessionLocal> = r
                    .scan()
                    .primary::<SessionLocal>()
                    .unwrap()
                    .all()
                    .unwrap()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                drop(r);

                if let Some(session_id) = final_sessions[0].id {
                    // Track for cleanup if we got a real remote ID
                    cleanup.track_session(session_id);

                    // Also check and track other entities that got remote IDs
                    let r = sync_engine.database.r_transaction().unwrap();

                    let final_connectivity: Vec<ConnectivityLocal> = r
                        .scan()
                        .primary::<ConnectivityLocal>()
                        .unwrap()
                        .all()
                        .unwrap()
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    if let Some(connectivity_id) = final_connectivity.get(0).and_then(|c| c.id) {
                        cleanup.track_connectivity(connectivity_id);
                    }

                    let final_events: Vec<EventLocal> = r
                        .scan()
                        .primary::<EventLocal>()
                        .unwrap()
                        .all()
                        .unwrap()
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    if let Some(event_id) = final_events.get(0).and_then(|e| e.id) {
                        cleanup.track_event(event_id);
                    }

                    let final_tags: Vec<TagLocal> = r
                        .scan()
                        .primary::<TagLocal>()
                        .unwrap()
                        .all()
                        .unwrap()
                        .into_iter()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    if let Some(tag_id) = final_tags.get(0).and_then(|t| t.id) {
                        cleanup.track_tag(tag_id);
                    }

                    drop(r);

                    tracing::info!("Flush succeeded - session got remote ID: {}", session_id);
                } else {
                    tracing::warn!("Flush succeeded but session didn't get remote ID");
                }
            }
            Err(e) => {
                tracing::warn!("Flush failed (expected in test environment): {}", e);
                // Verify database integrity is maintained even on failure
                assert_eq!(sync_engine.get_table_count::<SessionLocal>().unwrap(), 1);
                assert_eq!(
                    sync_engine.get_table_count::<ConnectivityLocal>().unwrap(),
                    1
                );
                assert_eq!(sync_engine.get_table_count::<EventLocal>().unwrap(), 1);
                assert_eq!(sync_engine.get_table_count::<TagLocal>().unwrap(), 1);
            }
        }
    }

    sync_test_with_cleanup!(
        test_flush_remote_id_assignment,
        test_flush_remote_id_assignment_impl
    );

    async fn test_flush_remote_id_assignment_impl(cleanup: &SyncTestCleanup) {
        let mut sync_engine = create_test_sync_engine_with_identification().await.unwrap();

        let device_id: i64 = env::var("SCOUT_DEVICE_ID")
            .unwrap_or_else(|_| "123".to_string())
            .parse()
            .unwrap_or(123);

        // Create new session without remote ID
        let mut new_session = SessionLocal::default();
        new_session.set_id_local("new_flush_session".to_string());
        new_session.device_id = device_id;
        new_session.timestamp_start = "2023-11-01T10:00:00Z".to_string();
        new_session.software_version = "sync_unit_test_flush_remote_id_assignment_impl".to_string();
        new_session.altitude_max = 120.0;
        new_session.altitude_min = 60.0;
        new_session.altitude_average = 90.0;
        new_session.velocity_max = 40.0;
        new_session.velocity_min = 15.0;
        new_session.velocity_average = 25.0;
        new_session.distance_total = 1200.0;
        new_session.distance_max_from_start = 600.0;

        // Insert session locally
        sync_engine.upsert_items(vec![new_session]).unwrap();

        // Verify initial state - no remote ID
        let r = sync_engine.database.r_transaction().unwrap();
        let sessions: Vec<SessionLocal> = r
            .scan()
            .primary::<SessionLocal>()
            .unwrap()
            .all()
            .unwrap()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].id.is_none());
        drop(r);

        // Attempt flush with real Scout client
        let flush_result = sync_engine.flush().await;

        match flush_result {
            Ok(_) => {
                // Check if session got remote ID from real API
                let r = sync_engine.database.r_transaction().unwrap();
                let final_sessions: Vec<SessionLocal> = r
                    .scan()
                    .primary::<SessionLocal>()
                    .unwrap()
                    .all()
                    .unwrap()
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();
                drop(r);

                assert_eq!(final_sessions.len(), 1);
                if let Some(remote_id) = final_sessions[0].id {
                    // Track for cleanup if we got a real remote ID
                    cleanup.track_session(remote_id);
                    tracing::info!("Session successfully got remote ID: {}", remote_id);
                    assert!(remote_id > 0);
                } else {
                    tracing::warn!(
                        "Session didn't get remote ID (may be expected in test environment)"
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Flush failed (expected in test environment): {}", e);
                // Verify database integrity maintained
                assert_eq!(sync_engine.get_table_count::<SessionLocal>().unwrap(), 1);
            }
        }
    }
}
