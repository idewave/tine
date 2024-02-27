use futures::future::join_all;
use crate::primary::server::LoginServer;

mod primary;

#[tokio::main]
async fn main() {
    let run_login_server = tokio::spawn(async move {
        LoginServer::new().run().await;
    });

    join_all(vec![run_login_server]).await;
}
