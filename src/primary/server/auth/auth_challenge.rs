use anyhow::{Result as AnyResult};
use serde::{Deserialize, Serialize};
use tentacli::packet::idewave::WorldPacket;

use crate::primary::server::opcodes::Opcode;
use crate::with_opcode;

with_opcode! {
    @world_opcode(Opcode::SMSG_AUTH_CHALLENGE)
    #[derive(WorldPacket, Serialize, Deserialize, Debug, Default)]
    struct Outcome {
        unknown: u32,
        server_seed: u32,
        seed: [u8; 32],
    }
}

pub async fn handle() -> AnyResult<Vec<u8>> {
    let packet = Outcome {
        unknown: 0,
        server_seed: rand::random(),
        seed: rand::random(),
    }.to_binary()?;

    Ok(packet)
}