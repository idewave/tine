use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tentacli_packet::WorldPacket;
use tentacli_traits::types::custom_fields::TerminatedString;
use tentacli_traits::types::opcodes::Opcode;
use tentacli_traits::types::player::{Class, Gender, Race};
use tentacli_utils::generate_random_number;

use crate::primary::traits::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

#[derive(WorldPacket, Serialize, Deserialize, Debug)]
struct Outcome {
    characters_count: u8,
    guid: u64,
    name: TerminatedString,
    race: u8,
    class: u8,
    gender: u8,
    skin: u8,
    face: u8,
    hair_style: u8,
    hair_color: u8,
    facial_hair: u8,
    level: u8,
    zone_id: u32,
    map_id: u32,
    x: f32,
    y: f32,
    z: f32,
    guild_id: u32,
    char_flags: u32,
    char_customize_flags: u32,
    first_login: u8,
    pet_display_id: u32,
    pet_level: u32,
    pet_family: u32,
    inventory: Vec<u8>
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, _: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        response.push(HandlerOutput::Data(
            Outcome {
                characters_count: 1,
                guid: generate_random_number(),
                name: TerminatedString("Test".to_string()),
                race: Race::HUMAN,
                class: Class::PRIEST,
                gender: Gender::GENDER_MALE,
                skin: 1,
                face: 5,
                hair_style: 1,
                hair_color: 1,
                facial_hair: 2,
                level: 80,
                zone_id: 3430,
                map_id: 530,
                x: 10349.6,
                y: -6357.29,
                z: 33.4026,
                guild_id: 0,
                char_flags: 0,
                char_customize_flags: 0,
                first_login: 1,
                pet_display_id: 0,
                pet_level: 0,
                pet_family: 0,
                inventory: vec![0u8; 9 * 23],
            }.to_binary_with_server_opcode(Opcode::SMSG_CHAR_ENUM)?
        ));

        println!("SEND Opcode::SMSG_CHAR_ENUM");

        Ok(response)
    }
}