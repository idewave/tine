use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outcome {
    flags: [u8; 32],
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        response.push(HandlerOutput::Data(
            Outcome {
                flags: [0u8; 32],
            }.to_binary_with_server_opcode(Opcode::SMSG_TUTORIAL_FLAGS)?
        ));

        println!("[SEND] Opcode::SMSG_TUTORIAL_FLAGS");

        Ok(response)
    }
}