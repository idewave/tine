use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::Processor;
use crate::primary::types::{HandlerInput, ProcessorResult};

mod addon_info;
mod auth_response;
mod client_cache_version;
mod motd;
mod notification;
mod set_encryption;
mod set_time;
mod set_ui_time;
mod set_weather;
mod tutorial_flags;

pub struct RealmProcessor;

impl Processor for RealmProcessor {
    fn get_handlers(input: &mut HandlerInput) -> ProcessorResult {
        let handlers: ProcessorResult = match input.opcode {
            Opcode::CMSG_AUTH_SESSION => {
                vec![
                    Box::new(set_encryption::Handler),
                    Box::new(auth_response::Handler),
                    Box::new(addon_info::Handler),
                    Box::new(client_cache_version::Handler),
                    Box::new(tutorial_flags::Handler),
                ]
            }
            Opcode::CMSG_UI_TIME_REQUEST => {
                vec![Box::new(set_ui_time::Handler)]
            }
            Opcode::CMSG_QUERY_TIME => {
                vec![Box::new(set_time::Handler)]
            }
            Opcode::CMSG_PLAYER_LOGIN => {
                vec![
                    // Box::new(notification::Handler),
                    // Box::new(motd::Handler),
                    // Box::new(set_weather::Handler),
                ]
            }
            _ => vec![],
        };

        handlers
    }
}
