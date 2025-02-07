use anyhow::Result as AnyResult;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

#[derive(WorldPacket, Serialize, Deserialize, Debug, Default)]
struct Outgoing {
    unknown: u32,
    server_seed: u32,
    seed: [u8; 32],
}

pub async fn handle() -> AnyResult<Vec<u8>> {
    let server_seed = rand::random();

    let packet = Outgoing {
        unknown: 0,
        server_seed,
        seed: rand::random(),
    }
    .to_binary_with_server_opcode(Opcode::SMSG_AUTH_CHALLENGE)?;

    println!("[SEND] Opcode::SMSG_AUTH_CHALLENGE");

    Ok(packet)
}
