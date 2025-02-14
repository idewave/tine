use std::sync::Arc;

use anyhow::Result as AnyResult;
use tokio::sync::Mutex;

use crate::primary::crypto::header_crypt::{HeaderDecryptor, HeaderEncryptor};
use crate::primary::crypto::srp::Srp;
use crate::primary::traits::packet_handler::PacketHandler;

#[derive(Debug)]
pub struct HandlerInput {
    pub data: Vec<u8>,
    pub opcode: u32,
    pub srp: Arc<Mutex<Srp>>,
    pub world_port: u16,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum HandlerOutput {
    // data transfer
    Data(u16, Vec<u8>),
    SessionKey(Vec<u8>),
    HeaderCrypt(HeaderEncryptor, HeaderDecryptor),
}

pub type HandlerResult = AnyResult<Vec<HandlerOutput>>;

pub type ProcessorResult = Vec<Box<dyn PacketHandler + Send>>;

pub type ProcessorFunction = Box<dyn Fn(&mut HandlerInput) -> ProcessorResult + Send>;
