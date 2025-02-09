use std::sync::Arc;

use anyhow::Result as AnyResult;
use async_broadcast::Receiver as BroadcastReceiver;
use futures::future::join_all;
use tentacli_traits::types::HandlerOutput;
use tokio::sync::Mutex;

pub(crate) use primary::debug;

pub use crate::primary::crypto::srp::Srp;
use crate::primary::server::{LoginServer, WorldServer};
pub use crate::primary::traits::base_server::{BaseServer, RunOptions};

mod primary;

#[derive(Default)]
pub struct Options {
    pub receiver: Option<BroadcastReceiver<HandlerOutput>>,
    pub login_port: u16,
    pub world_port: u16,
}

pub struct Server;
impl Server {
    pub async fn run(
        Options {
            receiver,
            login_port,
            world_port,
        }: Options,
    ) -> AnyResult<()> {
        let srp = Arc::new(Mutex::new(Srp::new()));

        let login_options = Arc::new(RunOptions {
            srp: srp.clone(),
            login_port,
            world_port,
            ..RunOptions::default()
        });

        let world_options = Arc::new(RunOptions {
            srp,
            receiver,
            login_port,
            world_port,
        });

        let run_login_server = || {
            tokio::spawn(async move {
                if let Err(err) = LoginServer::new().run(login_options).await {
                    debug!("Error running Login Server: {}", err);
                }
            })
        };

        let run_world_server = || {
            tokio::spawn(async move {
                if let Err(err) = WorldServer::new().run(world_options).await {
                    debug!("Error running World Server: {}", err);
                }
            })
        };

        join_all(vec![run_login_server(), run_world_server()]).await;

        Ok(())
    }
}
