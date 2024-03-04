use std::collections::BTreeMap;
use std::sync::{Arc};
use async_trait::async_trait;
use tokio::io::{AsyncWriteExt};
use tokio::net::TcpStream;

mod opcodes;
mod auth;

use crate::primary::server::auth::{auth_challenge, AuthProcessor};
use crate::primary::traits::processor::Processor;
use crate::primary::traits::server::{RunOptions, Server};
use crate::primary::types::{HandlerInput, ProcessorFunction};

const HOST: &str = "127.0.0.1";
const LOGIN_PORT: u16 = 3724;
pub const WORLD_PORT: u16 = 8999;

type SessionKey = Vec<u8>;
type Sessions = BTreeMap<String, Option<SessionKey>>;

pub struct LoginServer {}

#[async_trait]
impl Server for LoginServer {
    fn new() -> Self {
        Self {}
    }

    fn generate_input(packet: &[u8], options: &RunOptions) -> HandlerInput {
        HandlerInput {
            data: packet[1..].to_vec(),
            opcode: packet[0] as u16,
            srp: Arc::clone(&options.srp),
        }
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![Box::new(AuthProcessor::get_handlers)]
    }

    fn host<'a>() -> &'a str {
        HOST
    }

    fn port() -> u16 {
        LOGIN_PORT
    }

    fn server_name<'a>() -> &'a str {
        "Login Server"
    }
}

pub struct WorldServer {}

#[async_trait]
impl Server for WorldServer {
    fn new() -> Self {
        Self {}
    }

    async fn init(socket: &mut TcpStream) {
        let packet = auth_challenge().await.unwrap();
        socket.write_all(&packet).await.unwrap();
    }

    fn generate_input(_packet: &[u8], options: &RunOptions) -> HandlerInput {
        HandlerInput {
            data: vec![],
            opcode: 0,
            srp: Arc::clone(&options.srp),
        }
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![]
    }

    fn host<'a>() -> &'a str {
        HOST
    }

    fn port() -> u16 {
        WORLD_PORT
    }

    fn server_name<'a>() -> &'a str {
        "World Server"
    }
}