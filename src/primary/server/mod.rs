use std::collections::BTreeMap;

mod auth;
mod realm;
mod login_server;
mod world_server;
pub mod types;
mod player;
mod mock_data;
mod movement;

use crate::primary::traits::processor::Processor;

const SERVER_HOST: &str = "127.0.0.1";
const LOGIN_PORT: u16 = 3724;
const WORLD_PORT: u16 = 8999;

type SessionKey = Vec<u8>;
type Sessions = BTreeMap<String, Option<SessionKey>>;

pub use login_server::LoginServer;
pub use world_server::WorldServer;