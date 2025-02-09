use std::collections::BTreeMap;

pub use login_server::LoginServer;
pub use world_server::WorldServer;

mod auth;
mod login_server;
mod mock_data;
mod movement;
mod player;
mod realm;
pub mod types;
mod world_server;

#[allow(dead_code)]
type SessionKey = Vec<u8>;
#[allow(dead_code)]
type Sessions = BTreeMap<String, Option<SessionKey>>;
