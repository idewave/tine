use async_trait::async_trait;
use idewave_packet::LoginPacket;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tentacli::realm::Realm;
use tentacli::traits::BinaryConverter;

use crate::primary::server::opcodes::Opcode;
use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};
use crate::primary::types::fields::realms::Realms;
use crate::with_opcode;

const REALM_SIZE: u16 = 64;

with_opcode! {
    @login_opcode(Opcode::REALM_LIST)
    #[derive(LoginPacket, Serialize, Deserialize, Debug, Default)]
    struct Outcome {
        size: u16,
        unknown: u32,
        realms_count: u16,
        realms: Realms,
    }
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        let realms = Self::generate_unique_realms();
        let realms_count = realms.len() as u16;

        response.push(HandlerOutput::Data(Outcome {
            size: REALM_SIZE * realms_count,
            unknown: 0,
            realms_count,
            realms: Realms(realms),
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

        realm.name = name;

        let port = rand::thread_rng().gen_range(16000..=25000);
        realm.address = format!("127.0.0.1:{}", port);

        realm.server_id = rand::thread_rng().gen_range(0..=100);

        realm
    }

    fn generate_unique_realms() -> Vec<Realm> {
        let mut rng = rand::thread_rng();
        let random_count: usize = rng.gen_range(0..=10);

        let mut realms = Vec::new();
        let mut generated_addresses = std::collections::HashSet::new();
        let mut generated_names = std::collections::HashSet::new();

        while realms.len() < random_count {
            let mut realm = Self::generate_realm();

            if generated_addresses.insert(realm.address.clone())
                && generated_names.insert(realm.name.clone())
            {
                realms.push(realm);
            }
        }

        realms
    }
}