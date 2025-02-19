use anyhow::Result as AnyResult;

use crate::primary::{Options, Server};

mod primary;

#[tokio::main]
async fn main() -> AnyResult<()> {
    Server::run(Options {
        relay_port: 3788,
        login_port: 3725,
        world_port: 19999,
        with_relay: true,
    })
    .await?;

    Ok(())
}
