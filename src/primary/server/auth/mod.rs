mod login_challenge;
mod realmlist;
mod login_proof;
mod types;
mod auth_challenge;

use crate::primary::server::opcodes::Opcode;
use crate::primary::traits::processor::Processor;
use crate::primary::types::{HandlerInput, ProcessorResult};
pub use auth_challenge::handle as auth_challenge;

pub struct AuthProcessor;

impl Processor for AuthProcessor {
    fn get_handlers(input: &mut HandlerInput) -> ProcessorResult {
        let opcode = input.opcode as u8;

        let handlers: ProcessorResult = match opcode {
            Opcode::LOGIN_CHALLENGE => {
                vec![Box::new(login_challenge::Handler)]
            },
            Opcode::LOGIN_PROOF => {
                vec![Box::new(login_proof::Handler)]
            },
            Opcode::REALM_LIST => {
                vec![Box::new(realmlist::Handler)]
            }
            _ => vec![],
        };

        handlers
    }
}