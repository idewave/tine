use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    time: u32,
    time2: u32,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        Ok(vec![HandlerOutput::Data(
            Opcode::SMSG_QUERY_TIME_RESPONSE,
            Outgoing {
                time: current_time as u32,
                time2: current_time as u32,
            }
            .to_binary_with_server_opcode(Opcode::SMSG_QUERY_TIME_RESPONSE)?,
        )])
    }
}
