mod char_enum;
mod login_verify_world;
mod motd;
mod bind_point_update;
mod initial_spells;
mod init_world_states;
mod login_set_timespeed;
mod player_spawn;
mod time_synq_req;
mod load_equipment_set;
mod played_time;

use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::processor::Processor;
use crate::primary::types::{HandlerInput, ProcessorResult};

pub struct PlayerProcessor;
impl Processor for PlayerProcessor {
    fn get_handlers(input: &mut HandlerInput) -> ProcessorResult {
        let handlers: ProcessorResult = match input.opcode {
            Opcode::CMSG_CHAR_ENUM => {
                vec![
                    Box::new(char_enum::Handler),
                ]
            },
            Opcode::CMSG_PLAYER_LOGIN => {
                vec![
                    Box::new(login_verify_world::Handler),
                    Box::new(motd::Handler),
                    Box::new(bind_point_update::Handler),
                    Box::new(initial_spells::Handler),
                    Box::new(init_world_states::Handler),
                    Box::new(login_set_timespeed::Handler),
                    Box::new(player_spawn::Handler),
                    Box::new(time_synq_req::Handler),
                    Box::new(load_equipment_set::Handler),
                    Box::new(played_time::Handler),
                ]
            },
            _ => vec![]
        };

        handlers
    }
}