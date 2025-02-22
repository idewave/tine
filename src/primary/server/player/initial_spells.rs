use async_trait::async_trait;
use serde::Serialize;
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;
use tentacli_traits::types::spell::{CooldownInfo, Spell};

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Debug)]
struct Outgoing {
    unknown: u8,
    spells_count: u16,
    spells: Vec<Spell>,
    cooldowns_count: u16,
    cooldowns: Vec<CooldownInfo>,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let spells = vec![
            Spell { spell_id: 6603 }, // auto attack
        ];

        Ok(vec![HandlerOutput::Data(
            Opcode::SMSG_INITIAL_SPELLS,
            Outgoing {
                unknown: 0,
                spells_count: spells.len() as u16,
                spells,
                cooldowns_count: 0,
                cooldowns: vec![],
            }
            .to_binary_with_server_opcode(Opcode::SMSG_INITIAL_SPELLS)?,
        )])
    }
}
