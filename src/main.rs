use std::sync::{Arc};
use anyhow::{Result as AnyResult};
use futures::future::join_all;
use tokio::sync::Mutex;

use crate::primary::crypto::srp::Srp;
use crate::primary::server::{LoginServer, WorldServer};
use crate::primary::traits::server::{RunOptions, Server};

mod primary;

#[tokio::main]
async fn main() -> AnyResult<()> {
    let options = Arc::new(RunOptions { srp: Arc::new(Mutex::new(Srp::new())) });

    let run_login_server = || {
        let options = options.clone();
        tokio::spawn(async move {
            if let Err(err) = LoginServer::new().run(options).await {
                println!("Error running Login Server: {}", err);
            }
        })
    };

    let run_world_server = || {
        let options = options.clone();
        tokio::spawn(async move {
            if let Err(err) = WorldServer::new().run(options).await {
                println!("Error running World Server: {}", err);
            }
        })
    };

    join_all(vec![run_login_server(), run_world_server()]).await;

    Ok(())
}
