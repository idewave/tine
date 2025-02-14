use std::sync::Arc;

use futures::future::join_all;
use tokio::sync::Mutex;

use crate::primary::crypto::srp::Srp;
use crate::primary::server::{LoginServer, WorldServer};
use crate::primary::traits::server::{BaseServer, RunOptions};

pub mod crypto;
pub mod server;
pub mod traits;
pub mod types;

macro_rules! debug {
    ($($rest:tt)+) => {
        #[cfg(feature = "debug")]
        println!($($rest)*)
    }
}

pub(crate) use debug;

#[derive(Default)]
pub struct Options {
    pub login_port: u16,
    pub world_port: u16,
}

pub struct Server;
impl Server {
    pub async fn run(
        Options {
            login_port,
            world_port,
        }: Options,
    ) -> anyhow::Result<()> {
        let srp = Arc::new(Mutex::new(Srp::new()));

        let run_login_server = |port: u16| {
            let options = RunOptions { srp: srp.clone(), port, world_port };

            tokio::spawn(async move {
                if let Err(err) = LoginServer::default().start(options).await {
                    debug!("Error running Login Server: {}", err);
                }
            })
        };

        let run_world_server = |port: u16| {
            let options = RunOptions { srp: srp.clone(), port, world_port };

            tokio::spawn(async move {
                if let Err(err) = WorldServer::default().start(options).await {
                    debug!("Error running World Server: {}", err);
                }
            })
        };

        join_all(vec![run_login_server(login_port), run_world_server(world_port)]).await;

        Ok(())
    }
}
