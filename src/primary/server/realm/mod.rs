use tentacli_traits::types::opcodes::Opcode;

mod auth_response;
mod addon_info;
mod client_cache_version;
mod tutorial_flags;
mod set_encryption;
mod account_data_times;

use crate::primary::traits::processor::Processor;
use crate::primary::types::{HandlerInput, ProcessorResult};

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
                    // Box::new(auth_response::Handler),
                ]
            },
            Opcode::CMSG_READY_FOR_ACCOUNT_DATA_TIMES => {
                vec![
                    Box::new(account_data_times::Handler),
                ]
            }
            _ => vec![]
        };

        handlers
    }
}