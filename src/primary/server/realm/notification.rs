use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    text: String,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        println!("SEND Opcode::SMSG_NOTIFICATION");

        Ok(vec![HandlerOutput::Data(
            Outgoing {
                text: String::from("TINE is in development...\0"),
            }
            .to_binary_with_server_opcode(Opcode::SMSG_NOTIFICATION)?,
        )])
    }
}
