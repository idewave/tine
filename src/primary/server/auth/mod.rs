use tentacli_traits::types::opcodes::Opcode;

pub use auth_challenge::handle as auth_challenge;
pub use login_challenge::LoginChallengeIncoming;
pub use login_proof::LoginProofIncoming;
pub use realmlist::RealmlistIncoming;

use crate::primary::traits::Processor;
use crate::primary::types::{HandlerInput, ProcessorResult};

mod auth_challenge;
mod login_challenge;
mod login_proof;
mod realmlist;
mod types;

pub struct AuthProcessor;

impl Processor for AuthProcessor {
    fn get_handlers(input: &mut HandlerInput) -> ProcessorResult {
        let opcode = input.opcode as u8;

        let handlers: ProcessorResult = match opcode {
            Opcode::LOGIN_CHALLENGE => {
                vec![Box::new(login_challenge::Handler)]
            }
            Opcode::LOGIN_PROOF => {
                vec![Box::new(login_proof::Handler)]
            }
            Opcode::REALM_LIST => {
                vec![Box::new(realmlist::Handler)]
            }
            _ => vec![],
        };

        handlers
    }
}
