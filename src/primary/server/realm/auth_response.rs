use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

// Opcode::CMSG_AUTH_SESSION
#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Incoming {
    build: u32,
    unknown: u32,
    account: String,
    unknown2: u32,
    client_seed: [u8; 4],
    unknown3: u64,
    server_id: u32,
    unknown4: u64,
    digest: [u8; 20],
    // addons_count: u32,
    // addons: Vec<u8>,
}

// Opcode::SMSG_AUTH_RESPONSE
#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    status: u8,
    billing_time_remaining: u32,
    billing_plan_flags: u8,
    billing_time_rested: u32,
    expansion: u8,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let (Incoming { .. }, _) = Incoming::from_binary(&input.data)?;

        Ok(vec![HandlerOutput::Data(
            Opcode::SMSG_AUTH_RESPONSE,
            Outgoing {
                status: 0x0C,
                billing_time_remaining: 0,
                billing_plan_flags: 0,
                billing_time_rested: 0,
                expansion: 2,
            }
            .to_binary_with_server_opcode(Opcode::SMSG_AUTH_RESPONSE)?,
        )])
    }
}
