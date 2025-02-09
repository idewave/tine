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
    unknown: u8,
    mask: [u8; 4],
    data: Vec<u8>,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        Ok(vec![
            HandlerOutput::Data(
                Opcode::SMSG_ACCOUNT_DATA_TIMES,
                Outgoing {
                    time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32,
                    unknown: 1,
                    mask: [0x15, 0x00, 0x00, 0x00],
                    data: vec![
                        0x99, 0xE5, 0xDD, 0x66, 0x00, 0x00, 0x00, 0x00, 0x79, 0xDE, 0x48, 0x66,
                    ],
                }
                .to_binary_with_server_opcode(Opcode::SMSG_ACCOUNT_DATA_TIMES)?,
            ),
            HandlerOutput::Data(
                Opcode::SMSG_ACCOUNT_DATA_TIMES,
                Outgoing {
                    time: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32,
                    unknown: 1,
                    mask: [0xEA, 0x00, 0x00, 0x00],
                    data: vec![
                        0x94, 0x51, 0xA6, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x7A, 0xDE, 0x48, 0x66,
                    ],
                }
                .to_binary_with_server_opcode(Opcode::SMSG_ACCOUNT_DATA_TIMES)?,
            ),
        ])
    }
}
