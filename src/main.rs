use anyhow::Result as AnyResult;

use crate::primary::{Options, Server};

mod primary;

#[tokio::main]
async fn main() -> AnyResult<()> {
    Server::run(Options {
        login_port: 3724,
        world_port: 19999,
    })
    .await?;

    Ok(())
}
