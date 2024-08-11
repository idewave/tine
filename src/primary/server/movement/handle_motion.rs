use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::custom_fields::PackedGuid;
use tentacli_traits::types::movement::MovementInfo;
use tentacli_traits::types::opcodes::Opcode;
use crate::primary::server::mock_data::CurrentPlayer;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Debug)]
struct Incoming {
    packed_guid: PackedGuid,
    movement_info: MovementInfo,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        let (Incoming { packed_guid, movement_info }, _) = Incoming::from_binary(&input.data)?;

        response.push(HandlerOutput::Data(
            Incoming {
                packed_guid,
                movement_info,
            }.to_binary_with_server_opcode(input.opcode as u16)?
        ));

        println!("{:?}", Opcode::get_opcode_name(input.opcode));

        Ok(response)
    }
}