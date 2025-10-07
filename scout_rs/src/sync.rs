use crate::{client::ScoutClient, models::Herd};
use anyhow::Result;
use native_db::{Builder, Database, Models};

pub struct SyncEngine<'a> {
    scout_client: ScoutClient,
    db_local_path: String,
    database: Database<'a>,
}

impl<'a> SyncEngine<'a> {
    pub fn new(scout_client: ScoutClient, db_local_path: String) -> Result<SyncEngine<'a>> {
        let mut models = Models::new();
        match models.define::<Herd>() {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        }
        let db = Builder::new().create(&models, db_local_path)?;
        Ok(Self {
            scout_client,
            db_local_path,
            database: db,
        })
    }
}
