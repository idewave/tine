use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use tentacli::packet::idewave::LoginPacket;

use crate::primary::server::auth::types::AccountFlags;
use crate::primary::server::opcodes::Opcode;

use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};
use crate::with_opcode;

with_opcode! {
    @login_opcode(Opcode::LOGIN_PROOF)
    #[derive(LoginPacket, Serialize, Deserialize, Debug)]
    struct Income {
        client_ephemeral: [u8; 32],
        client_proof: [u8; 20],
        crc_hash: [u8; 20],
        keys_count: u8,
        security_flags: u8,
    }
}

with_opcode! {
    @login_opcode(Opcode::LOGIN_PROOF)
    #[derive(LoginPacket, Serialize, Deserialize, Debug)]
    struct Outcome {
        error: u8,
        server_proof: [u8; 20],
        account_flags: u32,
        survey_id: u32,
        unknown_flags: u16,
    }
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();
        let (Income { client_ephemeral, client_proof, .. }, _) = Income::from_binary(&input.data)?;

        let mut srp = input.srp.lock().unwrap();
        srp.calculate_session_key::<Sha1>(&client_ephemeral);

        let server_proof = srp.calculate_proof::<Sha1>(&client_ephemeral);

        if server_proof == client_proof {
            let session_key = srp.session_key.as_ref().unwrap().to_vec();

            let server_proof = {
                let hasher = Sha1::new();

                let result = hasher
                    .chain(client_ephemeral)
                    .chain(server_proof)
                    .chain(session_key.to_vec())
                    .finalize();

                let mut hashed_proof = [0u8; 20];
                hashed_proof.copy_from_slice(&result);
                hashed_proof
            };

            response.push(HandlerOutput::SessionKey(session_key));
            response.push(HandlerOutput::Data(Outcome {
                error: 0,
                server_proof,
                account_flags: AccountFlags::ACCOUNT_FLAG_PROPASS,
                survey_id: 0,
                unknown_flags: 0,
            }.to_binary()?));
        }

        Ok(response)
    }
}