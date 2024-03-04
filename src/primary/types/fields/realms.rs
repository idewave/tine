use std::io::BufRead;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tentacli::errors::FieldError;
use tentacli::traits::BinaryConverter;

use std::fmt::{Debug};
use tentacli::realm::Realm;

#[derive(Clone, Default, Debug)]
pub struct Realms(pub Vec<Realm>);

impl BinaryConverter for Realms {
    fn write_into(&mut self, buffer: &mut Vec<u8>) -> Result<(), FieldError> {
        for realm in self.0.iter_mut() {
            buffer.push(realm.icon);
            buffer.push(realm.lock);
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