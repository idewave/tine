//! TINE IS NOT EMULATOR, it is just a tiny WoW server (currently for version 3.3.5a only),
//! implemented for testing. It contains only basic functionality, which probably will be extended in the future.
//!
//! The main objectives of the project are: it can be used for testing the sending and receiving of packets;
//! it can replay packets received from external sources (e.g., it allows connecting a regular WoW client to tine,
//! which is linked to tentacli, and observing the packets in real-time,
//! effectively letting you see the world as tentacli does in the area it is spawned).

use std::sync::Arc;

use anyhow::Result as AnyResult;
use async_broadcast::{Receiver as BroadcastReceiver, Sender as BroadcastSender};
use futures::future::join_all;
use tentacli_traits::types::HandlerOutput as ClientHandlerOutput;
use tokio::sync::Mutex;

pub(crate) use primary::debug;

pub use crate::primary::crypto::srp::Srp;
use crate::primary::server::{LoginServer, WorldServer};
pub use crate::primary::traits::base_server::{BaseServer, RunOptions};

mod primary;

#[derive(Default)]
pub struct Options {
    pub sender: Option<BroadcastSender<ClientHandlerOutput>>,
    pub receiver: Option<BroadcastReceiver<ClientHandlerOutput>>,
    pub login_port: u16,
    pub world_port: u16,
}

pub struct Server;
impl Server {
    pub async fn run(
        Options {
            sender,
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
            sender,
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
