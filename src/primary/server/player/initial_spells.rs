use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    unknown: u8,
    spells_count: u16,
    // TODO: need to add write_into for Vec<Spell>
    spells: Vec<u8>,
    cooldowns_count: u16,
    // TODO: need to add write_into for Vec<CooldownInfo>
    cooldowns: Vec<u8>,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        response.push(HandlerOutput::Data(
            Outgoing {
                unknown: 0,
                spells_count: 0,
                spells: vec![],
                cooldowns_count: 0,
                cooldowns: vec![],
            }
            .to_binary_with_server_opcode(Opcode::SMSG_INITIAL_SPELLS)?,
        ));

        println!("SEND Opcode::SMSG_INITIAL_SPELLS");

        Ok(response)
    }
}
