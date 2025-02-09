use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;
use tentacli_traits::types::world::WeatherState;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    state: u32,
    grade: f32,
    unknown: u8,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        Ok(vec![HandlerOutput::Data(
            Opcode::SMSG_WEATHER,
            Outgoing {
                state: WeatherState::MEDIUM_RAIN,
                grade: 0.8,
                unknown: 0,
            }
            .to_binary_with_server_opcode(Opcode::SMSG_WEATHER)?,
        )])
    }
}
