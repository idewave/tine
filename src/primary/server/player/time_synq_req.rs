use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::custom_fields::TerminatedString;
use tentacli_traits::types::opcodes::Opcode;
use tentacli_traits::types::player::{Class, Gender, Race};
use tentacli_utils::generate_random_number;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outcome {
    unknown: u32,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        response.push(HandlerOutput::Data(
            Outcome {
                unknown: 0,
            }.to_binary_with_server_opcode(Opcode::SMSG_TIME_SYNC_REQ)?
        ));

        println!("SEND Opcode::SMSG_TIME_SYNC_REQ");

        Ok(response)
    }
}