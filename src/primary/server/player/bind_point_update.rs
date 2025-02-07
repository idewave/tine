use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::server::mock_data::CurrentPlayer;
use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    x: f32,
    y: f32,
    z: f32,
    map_id: u32,
    area_id: u32,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        println!("SEND Opcode::SMSG_BINDPOINTUPDATE");

        Ok(vec![HandlerOutput::Data(
            Outgoing {
                x: CurrentPlayer::POS_X,
                y: CurrentPlayer::POS_Y,
                z: CurrentPlayer::POS_Z,
                map_id: CurrentPlayer::MAP_ID,
                area_id: CurrentPlayer::ZONE_ID,
            }
            .to_binary_with_server_opcode(Opcode::SMSG_BINDPOINTUPDATE)?,
        )])
    }
}
