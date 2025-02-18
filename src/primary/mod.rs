use std::sync::Arc;

use futures::future::join_all;
use tokio::sync::{mpsc, Mutex};

use crate::primary::crypto::srp::Srp;
use crate::primary::server::{LoginServer, ProxyServer, WorldServer};
use crate::primary::traits::server::{BaseServer, RelayFlag, RunOptions};

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
    pub proxy_port: u16,
    pub login_port: u16,
    pub world_port: u16,
    pub with_proxy: bool,
}

pub struct Server;
impl Server {
    pub async fn run(
        Options {
            proxy_port,
            login_port,
            world_port,
            with_proxy,
        }: Options,
    ) -> anyhow::Result<()> {
        let (relay_sender, relay_receiver) = mpsc::channel::<(u32, Vec<u8>)>(100);
        let relay_receiver = Arc::new(Mutex::new(relay_receiver));

        let srp = Arc::new(Mutex::new(Srp::new()));

        let run_login_server = |port: u16| {
            let options = RunOptions {
                srp: srp.clone(),
                port,
                world_port,
                ..Default::default()
            };

            let tx = relay_sender.clone();
            let rx = relay_receiver.clone();

            tokio::spawn(async move {
                if let Err(err) = LoginServer::start(options, tx, rx).await {
                    debug!("Error running Login Server: {}", err);
                }
            })
        };

        let run_world_server = |port: u16| {
            let options = RunOptions {
                srp: srp.clone(),
                port,
                world_port,
                relay: if with_proxy {
                    RelayFlag::Receive
                } else {
                    RelayFlag::None
                },
            };

            let tx = relay_sender.clone();
            let rx = relay_receiver.clone();

            tokio::spawn(async move {
                if let Err(err) = WorldServer::start(options, tx, rx).await {
                    debug!("Error running World Server: {}", err);
                }
            })
        };

        let mut tasks = vec![run_login_server(login_port), run_world_server(world_port)];

        if with_proxy {
            let run_proxy_server = |port: u16| {
                let options = RunOptions {
                    srp: srp.clone(),
                    port,
                    world_port,
                    relay: RelayFlag::Send,
                };

                let tx = relay_sender.clone();
                let rx = relay_receiver.clone();

                tokio::spawn(async move {
                    if let Err(err) = ProxyServer::start(options, tx, rx).await {
                        debug!("Error running Login Server: {}", err);
                    }
                })
            };

            tasks.push(run_proxy_server(proxy_port));
        }

        join_all(tasks).await;

        Ok(())
    }
}
