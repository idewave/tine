use async_trait::async_trait;
use rand::Rng;
use serde::{Deserialize, Serialize};
use tentacli::packet::idewave::{FieldsSerializer, LoginPacket};
use tentacli::realm::Realm;

use crate::primary::server::opcodes::Opcode;
use crate::primary::server::WORLD_PORT;
use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};
use crate::primary::types::fields::realms::Realms;
use crate::with_opcode;

with_opcode! {
    @login_opcode(Opcode::REALM_LIST)
    #[derive(LoginPacket, Serialize, Deserialize, Debug, Default)]
    struct Outcome {
        size: u16,
        unknown: u32,
        realms_count: u16,
        realms: Vec<u8>,
        unknown2: u16,
    }
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        let realms = Self::generate_unique_realms();
        let realms_count = realms.len() as u16;

        let realms_bytes = {
            #[derive(FieldsSerializer, Serialize, Deserialize)]
            struct RealmsSerializer {
                realms: Realms,
            }

            RealmsSerializer { realms: Realms(realms.clone()) }.to_binary().unwrap()
        };

        response.push(HandlerOutput::Data(Outcome {
            size: (realms_bytes.len() + 8) as u16,
            unknown: 0,
            realms_count,
            realms: realms_bytes,
            unknown2: 0x0010,
        }.to_binary()?));

        Ok(response)
    }
}

impl Handler {
    fn generate_realm() -> Realm {
        let mut realm = Realm::default();

        let name: String = rand::thread_rng()
            .sample_iter(rand::distributions::Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        realm.icon = 1;
        realm.lock = 0;
        realm.flags = 1;
        realm.name = name;
        realm.address = format!("127.0.0.1:{}", WORLD_PORT);
        realm.timezone = 1;
        realm.server_id = rand::thread_rng().gen_range(0..=100);

        realm
    }

    fn generate_unique_realms() -> Vec<Realm> {
        let mut rng = rand::thread_rng();
        let random_count: usize = rng.gen_range(1..=10);

        let mut realms = Vec::new();
        let mut generated_names = std::collections::HashSet::new();

        while realms.len() < random_count {
            let realm = Self::generate_realm();

            if generated_names.insert(realm.name.clone()) {
                realms.push(realm);
            }
        }

        realms
    }
}