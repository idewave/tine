use tentacli_traits::types::opcodes::Opcode;

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
mod motd;
mod played_time;
mod player_spawn;
mod set_buttons;
mod time_synq_req;

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
                    Box::new(motd::Handler),
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
