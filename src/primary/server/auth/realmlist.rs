use std::io::BufRead;
use async_trait::async_trait;
use idewave_packet::LoginPacket;
use rand::Rng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::ser::SerializeStruct;
use tentacli::realm::Realm;
use tentacli::traits::BinaryConverter;
use tentacli::errors::FieldError;

use crate::primary::server::opcodes::Opcode;
use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};
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

#[derive(Clone, Default, Debug)]
pub struct Realms(pub Vec<Realm>);

impl BinaryConverter for Realms {
    fn write_into(&mut self, buffer: &mut Vec<u8>) -> Result<(), FieldError> {
        buffer.extend(&(self.0.len() as i16).to_le_bytes());

        for realm in self.0.iter_mut() {
            buffer.extend(&realm.icon.to_le_bytes());
            buffer.push(realm.flags);
            buffer.extend(realm.name.as_bytes());
            buffer.push(0);
            buffer.extend(realm.address.as_bytes());
            buffer.push(0);
            buffer.extend(&realm.population.to_le_bytes());
            buffer.push(realm.characters);
            buffer.push(realm.timezone);
            buffer.push(realm.server_id);
        }

        Ok(())
    }

    fn read_from<R: BufRead>(_reader: R) -> Result<Self, FieldError> {
        todo!()
    }
}

impl<'de> Deserialize<'de> for Realms {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        todo!()
    }
}

impl Serialize for Realms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        self.0.serialize(serializer)
    }
}