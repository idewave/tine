use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    flags: [u8; 32],
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        crate::debug!("[SEND] Opcode::SMSG_TUTORIAL_FLAGS");

        Ok(vec![HandlerOutput::Data(
            Outgoing {
                flags: [0xFFu8; 32],
            }
            .to_binary_with_server_opcode(Opcode::SMSG_TUTORIAL_FLAGS)?,
        )])
    }
}
