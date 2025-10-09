use crate::{
    client::ScoutClient,
    models::{AncestorLocal, Connectivity, Event, Session, Syncable, Tag},
};
use anyhow::{Error, Result};
use native_db::{Builder, Database, Models, ToInput};
use tokio::time::Interval;
use tracing::error;

pub struct SyncEngine {
    scout_client: ScoutClient,
    db_local_path: String,
    database: Database<'static>,
    interval_flushing_sessions: Option<Interval>,
    max_session_life_secs: u64,
}

pub enum EnumSyncAction {
    Upsert,
    Insert,
    Delete,
}

const DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS: EnumSyncAction = EnumSyncAction::Upsert;
const DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS: EnumSyncAction = EnumSyncAction::Insert;

#[derive(Default)]
pub struct BatchSync<T: ToInput + Syncable> {
    upsert: Vec<T>,
    insert: Vec<T>,
    delete: Vec<T>,
    sync_descendants: Vec<T>,
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
    pub fn new(scout_client: ScoutClient, db_local_path: String) -> Result<Self> {
        let mut models = Models::new();

        // Define all models for local database
        models.define::<Session>()?;
        models.define::<Event>()?;
        models.define::<Tag>()?;
        models.define::<Connectivity>()?;

        // Create database - use Box::leak to get 'static lifetime
        let models_static = Box::leak(Box::new(models));
        let database = Builder::new().create(models_static, &db_local_path)?;
        // initialize tracing
        tracing_subscriber::fmt::init();
        Ok(Self {
            scout_client,
            db_local_path,
            database,
        })
    }

    pub fn get_batch<T: Syncable + ToInput>(
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

    pub fn get_batch_for_session_ancestor<T: ToInput + Syncable + AncestorLocal>(
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
        let r = self.database.r_transaction()?;
        let raw_sessions_batch: Result<BatchSync<Session>, Error> = self.get_batch::<Session>(
            DEFAULT_ACTION_FOR_ITEMS_WITH_EXISTING_REMOTE_IDS,
            DEFAULT_ACTION_FOR_ITEMS_WITHOUT_EXISTING_REMOTE_IDS,
        );
        Ok(())
    }
    pub fn get_db_path(&self) -> &str {
        &self.db_local_path
    }

    pub fn add_items<T: ToInput>(&mut self, items: Vec<T>) -> Result<(), Error> {
        let rw = self.database.rw_transaction();
        match rw {
            Ok(rw) => {
                for item in items {
                    rw.insert(item)?;
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
}
