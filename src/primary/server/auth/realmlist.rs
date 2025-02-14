use async_trait::async_trait;
use rand::random_range;
use serde::{Deserialize, Serialize};
use tentacli_packet::{LoginPacket, Segment};
use tentacli_traits::types::opcodes::Opcode;
use tentacli_traits::types::realm::Realm;

use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(LoginPacket, Serialize, Deserialize, Debug)]
pub struct RealmlistIncoming {
    skip: u32,
}

#[derive(LoginPacket, Serialize, Deserialize, Debug, Default)]
struct Outgoing {
    size: u16,
    unknown: u32,
    realms_count: u16,
    realms: Vec<u8>,
    unknown2: u16,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let realms_bytes = {
            #[derive(Segment, Serialize)]
            struct RealmsSerializer {
                realms: Vec<Realm>,
            }

            RealmsSerializer {
                realms: vec![Self::generate_realm(input.world_port)],
            }
            .to_binary()
            .unwrap()
        };

        Ok(vec![HandlerOutput::Data(
            Opcode::REALM_LIST as u16,
            Outgoing {
                size: (realms_bytes.len() + 8) as u16,
                unknown: 0,
                realms_count: 1,
                realms: realms_bytes,
                unknown2: 0x0010,
            }
            .to_binary_with_opcode(Opcode::REALM_LIST)?,
        )])
    }
}

impl Handler {
    fn generate_realm(port: u16) -> Realm {
        Realm {
            icon: 1,
            flags: 1,
            name: String::from("TINE Aspect Server"),
            address: format!("127.0.0.1:{}", port),
            timezone: 1,
            server_id: random_range(0..=100),
            ..Realm::default()
        }
    }
}
