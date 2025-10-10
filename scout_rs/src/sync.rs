use crate::{
    client::ScoutClient,
    models::{AncestorLocal, ConnectivityLocal, EventLocal, SessionLocal, Syncable, TagLocal},
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
        // first lets get all sessions from the database
        // Open a read-only transaction
        let _raw_sessions_batch: Result<BatchSync<SessionLocal>, Error> = self
            .get_batch::<SessionLocal>(
                DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS,
                DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS,
            );
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
}

#[cfg(test)]
// to run thids specific test suite use cargo test --test sync
mod tests {
    use super::*;
    use crate::models::SessionLocal;
    use tempfile::tempdir;

    fn create_test_sync_engine() -> Result<SyncEngine> {
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

        // Create a mock scout client - this will depend on your ScoutClient implementation
        // For now, assuming it has a default or test constructor
        let scout_client = ScoutClient::new(String::new())?;
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
}
