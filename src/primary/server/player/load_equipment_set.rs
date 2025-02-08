use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    count: u32,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        crate::debug!("SEND Opcode::SMSG_LOAD_EQUIPMENT_SET");

        Ok(vec![HandlerOutput::Data(
            Outgoing { count: 0 }.to_binary_with_server_opcode(Opcode::SMSG_LOAD_EQUIPMENT_SET)?,
        )])
    }
}
