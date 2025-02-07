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

const SERVER_HOST: &str = "127.0.0.1";
const LOGIN_PORT: u16 = 3724;
const WORLD_PORT: u16 = 8999;

#[allow(dead_code)]
type SessionKey = Vec<u8>;
#[allow(dead_code)]
type Sessions = BTreeMap<String, Option<SessionKey>>;
