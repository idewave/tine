use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::server::mock_data::CurrentPlayer;
use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
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
        println!("SEND Opcode::SMSG_LOGIN_VERIFY_WORLD");

        Ok(vec![HandlerOutput::Data(
            Outgoing {
                map_id: CurrentPlayer::MAP_ID,
                x: CurrentPlayer::POS_X,
                y: CurrentPlayer::POS_Y,
                z: CurrentPlayer::POS_Z,
                orientation: 0.0,
            }
            .to_binary_with_server_opcode(Opcode::SMSG_LOGIN_VERIFY_WORLD)?,
        )])
    }
}
