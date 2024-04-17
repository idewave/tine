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
    map_id: u32,
    x: f32,
    y: f32,
    z: f32,
    orientation: f32,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        response.push(HandlerOutput::Data(
            Outcome {
                map_id: 530,
                x: 10349.6,
                y: -6357.29,
                z: 33.4026,
                orientation: 0.0,
            }.to_binary_with_server_opcode(Opcode::SMSG_LOGIN_VERIFY_WORLD)?
        ));

        println!("SEND Opcode::SMSG_LOGIN_VERIFY_WORLD");

        Ok(response)
    }
}