use std::sync::{Arc, Mutex as SyncMutex};
use anyhow::{Result as AnyResult};
use futures::future::join_all;
use crate::primary::crypto::srp::Srp;
use crate::primary::server::{LoginServer, WorldServer};
use crate::primary::traits::server::{RunOptions, Server};

mod primary;

#[tokio::main]
async fn main() -> AnyResult<()> {
    let options = Arc::new(RunOptions { srp: Arc::new(SyncMutex::new(Srp::new())) });

    let run_login_server = || {
        let options = options.clone();
        tokio::spawn(async move {
            LoginServer::new().run(options).await
        })
    };

    let run_world_server = || {
        let options = options.clone();
        tokio::spawn(async move {
            WorldServer::new().run(options).await
        })
    };

    join_all(vec![run_login_server(), run_world_server()]).await;

    Ok(())
}
