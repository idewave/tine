use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    timestamp: u32,
    unknown: u8,
    cache_mask: u32,
    data: [u8; 12],
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        crate::debug!("SENT SMSG_ACCOUNT_DATA_TIMES");

        Ok(vec![HandlerOutput::Data(
            Outgoing {
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32,
                unknown: 1,
                cache_mask: 0x15,
                data: [0u8; 12],
            }
            .to_binary_with_server_opcode(Opcode::SMSG_ACCOUNT_DATA_TIMES)?,
        )])
    }
}
