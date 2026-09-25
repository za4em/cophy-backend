use std::{error::Error, time::Duration};

use crate::{
    core::{config::Config, db, storage::Storage},
    features::{
        beans::Beans,
        recognition::{BagReader, Recognition},
        uploads::Uploads,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub(crate) beans: Beans,
    pub(crate) uploads: Uploads,
    pub(crate) recognition: Recognition,
}

impl AppState {
    pub async fn connect(config: Config) -> Result<Self, Box<dyn Error>> {
        let pool = db::connect(&config.database).await?;
        db::migrate(&pool).await?;

        let storage = Storage::connect(config.storage).await?;
        let reader = BagReader::new(config.openrouter)?;
        let beans = Beans::new(pool.clone(), storage.clone(), config.beans);
        let cleanup_interval = config.uploads.cleanup_interval;
        let uploads = Uploads::new(pool, storage, config.uploads);
        uploads.release_processing().await?;
        let recognition = Recognition::new(uploads.clone(), reader, beans.clone());
        let state = Self {
            beans,
            uploads,
            recognition,
        };

        spawn_upload_cleanup(state.uploads.clone(), cleanup_interval);

        Ok(state)
    }
}

fn spawn_upload_cleanup(uploads: Uploads, interval: Duration) {
    tokio::spawn(async move {
        loop {
            if let Err(err) = uploads.expire_abandoned().await {
                tracing::error!(error = %err, "could not expire abandoned uploads");
            }
            tokio::time::sleep(interval).await;
        }
    });
}
