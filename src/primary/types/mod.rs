use std::sync::{Arc, Mutex as SyncMutex};
use anyhow::{Result as AnyResult};
use crate::primary::crypto::srp::Srp;

use crate::primary::traits::packet_handler::PacketHandler;

#[derive(Debug)]
pub struct HandlerInput {
    pub data: Vec<u8>,
    pub opcode: u16,
    pub srp: Arc<SyncMutex<Srp>>,
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

#[derive(Default, Debug, Clone)]
pub struct IncomingPacket {
    pub opcode: u16,
    pub body: Vec<u8>,
}

#[derive(Default, Debug, Clone)]
pub struct OutgoingPacket {
    pub opcode: u32,
    pub data: Vec<u8>,
    pub json_details: String,
}