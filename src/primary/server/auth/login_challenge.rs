use std::io::BufRead;
use async_trait::async_trait;
use tentacli::packet::custom_fields::TerminatedString;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use tentacli::packet::idewave::LoginPacket;

use crate::with_opcode;
use crate::primary::server::opcodes::Opcode;
use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

const VERSION_CHALLENGE: [u8; 16] = [
    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57,
    0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1
];

with_opcode! {
    @login_opcode(Opcode::LOGIN_CHALLENGE)
    #[derive(LoginPacket, Serialize, Deserialize, Debug)]
    struct Income {
        unknown: u8,
        packet_size: u16,
        game_name: TerminatedString,
        version: [u8; 3],
        build: u16,
        platform: TerminatedString,
        os: TerminatedString,
        #[dynamic_field]
        locale: String,
        timezone: u32,
        ip: [u8; 4],
        account_length: u8,
        account: String,
    }

    impl Income {
        fn locale<R: BufRead>(mut reader: R, _: &mut Self) -> String {
            let mut buffer = vec![0u8; 4];
            reader.read_exact(&mut buffer).unwrap();
            buffer.reverse();
            String::from_utf8(buffer).unwrap()
        }
    }
}

with_opcode! {
    @login_opcode(Opcode::LOGIN_CHALLENGE)
    #[derive(LoginPacket, Serialize, Deserialize, Debug)]
    struct Outcome {
        unknown: u8,
        code: u8,
        server_ephemeral: Vec<u8>,
        g_len: u8,
        g: Vec<u8>,
        n_len: u8,
        n: Vec<u8>,
        salt: [u8; 32],
        // seems like this field was added in wotlk
        version_challenge: [u8; 16],
        unknown2: u8,
    }
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();

        let (Income { account, .. }, _) = Income::from_binary(&input.data)?;
        let mut srp = input.srp.lock().unwrap();
        srp.set_account(account);
        srp.generate_verifier::<Sha1>();
        srp.generate_server_ephemeral::<Sha1>();

        let (_, server_ephemeral) = srp.server_ephemeral.as_ref().unwrap().to_bytes_le();
        let (_, generator) = srp.generator.to_bytes_le();
        let (_, modulus) = srp.modulus.to_bytes_le();

        response.push(HandlerOutput::Data(Outcome {
            unknown: 0,
            code: 0,
            server_ephemeral,
            g_len: 1,
            g: generator,
            n_len: 32,
            n: modulus,
            salt: srp.salt,
            version_challenge: VERSION_CHALLENGE,
            unknown2: 0,
        }.to_binary()?));

        Ok(response)
    }
}