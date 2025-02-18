use anyhow::Result as AnyResult;

use crate::primary::{Options, Server};

mod primary;

#[tokio::main]
async fn main() -> AnyResult<()> {
    Server::run(Options {
        proxy_port: 3788,
        login_port: 3725,
        world_port: 19999,
        with_proxy: true,
    })
    .await?;

    Ok(())
}
