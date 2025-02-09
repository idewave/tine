use anyhow::Result as AnyResult;

use tine::{Options, Server};

#[tokio::main]
async fn main() -> AnyResult<()> {
    Server::run(Options {
        login_port: 3724,
        world_port: 19999,
        ..Options::default()
    })
    .await?;

    Ok(())
}
