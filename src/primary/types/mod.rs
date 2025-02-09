use std::sync::Arc;

use anyhow::Result as AnyResult;
use tokio::sync::Mutex;

use crate::primary::crypto::srp::Srp;
use crate::primary::traits::base_server::Connection;
use crate::primary::traits::packet_handler::PacketHandler;

#[derive(Debug, Copy, Clone)]
pub struct ServerInfo {
    pub login_port: u16,
    pub world_port: u16,
}

#[derive(Debug)]
pub struct HandlerInput {
    pub data: Vec<u8>,
    pub opcode: u32,
    pub srp: Arc<Mutex<Srp>>,
    pub connection: Arc<Mutex<Connection>>,
    pub server_info: ServerInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum HandlerOutput {
    // data transfer
    Data(Vec<u8>),
    SessionKey(Vec<u8>),
}

pub type HandlerResult = AnyResult<Vec<HandlerOutput>>;

pub type ProcessorResult = Vec<Box<dyn PacketHandler + Send>>;

pub type ProcessorFunction = Box<dyn Fn(&mut HandlerInput) -> ProcessorResult + Send>;
