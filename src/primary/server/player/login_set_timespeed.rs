use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    uptime: u32,
    game_speed: f32,
    unknown: u32,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        response.push(HandlerOutput::Data(
            Outgoing {
                uptime: 406030890,
                game_speed: 0.01666667,
                unknown: 0,
            }
            .to_binary_with_server_opcode(Opcode::SMSG_LOGIN_SETTIMESPEED)?,
        ));

        println!("SEND Opcode::SMSG_LOGIN_SETTIMESPEED");

        Ok(response)
    }
}
