use crate::{
    error::RfcErrorInfo,
    rfc::{RfcConnection, RfcConnectionBuilder},
};
use async_trait::async_trait;
use deadpool::{
    managed::{self, RecycleError},
    Runtime,
};
use deadpool_sync::SyncWrapper;

pub struct Manager {
    builder: RfcConnectionBuilder,
    runtime: Runtime,
}

impl Manager {
    pub fn new(builder: RfcConnectionBuilder, runtime: Runtime) -> Manager {
        Self { builder, runtime }
    }
}

#[async_trait]
impl managed::Manager for Manager {
    type Type = SyncWrapper<RfcConnection>;
    type Error = RfcErrorInfo;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        let builder = self.builder.clone();
        SyncWrapper::new(self.runtime, move || builder.build()).await
    }

    async fn recycle(&self, conn: &mut Self::Type) -> managed::RecycleResult<Self::Error> {
        if conn.is_mutex_poisoned() {
            return Err(RecycleError::StaticMessage(
                "Mutex is poisoned. Connection is considered unusable.",
            ));
        }
        conn.interact(|conn| conn.ping())
            .await
            .map_err(|err| RecycleError::Message(err.to_string()))??;
        Ok(())
    }
}
