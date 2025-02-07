use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    total_played_time: u32,
    level_played_time: u32,
    unknown: u8,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        response.push(HandlerOutput::Data(
            Outgoing {
                total_played_time: 0,
                level_played_time: 0,
                unknown: 0,
            }
            .to_binary_with_server_opcode(Opcode::SMSG_PLAYED_TIME)?,
        ));

        println!("SEND Opcode::SMSG_PLAYED_TIME");

        Ok(response)
    }
}
