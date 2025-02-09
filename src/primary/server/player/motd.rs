use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    lines_count: u32,
    lines: String,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        Ok(vec![HandlerOutput::Data(
            Opcode::SMSG_MOTD,
            Outgoing {
                lines_count: 1,
                lines: String::from("Welcome to the TINE Test Server\0"),
            }
            .to_binary_with_server_opcode(Opcode::SMSG_MOTD)?,
        )])
    }
}
