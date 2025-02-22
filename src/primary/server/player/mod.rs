use tentacli_traits::types::opcodes::Opcode;

pub use player_spawn::UpdatePacket;

use crate::primary::traits::Processor;
use crate::primary::types::{HandlerInput, ProcessorResult};

mod account_data_times;
mod bind_point_update;
mod char_enum;
mod init_world_states;
mod initial_spells;
mod load_equipment_set;
mod login_set_timespeed;
mod login_verify_world;
mod played_time;
mod player_spawn;
mod set_buttons;
mod time_synq_req;

#[non_exhaustive]
pub struct CurrentPlayer;

#[allow(dead_code)]
impl CurrentPlayer {
    pub const GUID: u64 = 123;
    pub const LEVEL: u8 = 80;
    pub const DISPLAY_ID: i32 = 49;

    // pub const POS_X: f32 = 10349.6;
    pub const POS_X: f32 = 16226.1;
    // pub const POS_Y: f32 = -6357.29;
    pub const POS_Y: f32 = 16257.8;
    // pub const POS_Z: f32 = 33.4026;
    pub const POS_Z: f32 = 13.2474;

    // pub const ZONE_ID: u32 = 3430;
    pub const ZONE_ID: u32 = 876;
    // pub const MAP_ID: u32 = 530;
    pub const MAP_ID: u32 = 1;
}

pub struct PlayerProcessor;
impl Processor for PlayerProcessor {
    fn get_handlers(input: &mut HandlerInput) -> ProcessorResult {
        let handlers: ProcessorResult = match input.opcode {
            Opcode::CMSG_CHAR_ENUM => {
                vec![Box::new(char_enum::Handler)]
            }
            Opcode::CMSG_PLAYER_LOGIN => {
                vec![
                    Box::new(login_verify_world::Handler),
                    Box::new(bind_point_update::Handler),
                    Box::new(initial_spells::Handler),
                    Box::new(set_buttons::Handler),
                    Box::new(load_equipment_set::Handler),
                    Box::new(login_set_timespeed::Handler),
                    Box::new(player_spawn::Handler),
                    Box::new(account_data_times::Handler),
                    Box::new(init_world_states::Handler),
                    Box::new(time_synq_req::Handler),
                ]
            }
            Opcode::CMSG_PLAYED_TIME => {
                vec![Box::new(played_time::Handler)]
            }
            _ => vec![],
        };

        handlers
    }
}
