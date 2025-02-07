use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use tentacli_packet::LoginPacket;
use tentacli_traits::types::opcodes::Opcode;

use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

const VERSION_CHALLENGE: [u8; 16] = [
    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57, 0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1,
];

#[derive(LoginPacket, Serialize, Deserialize, Debug)]
pub struct LoginChallengeIncoming {
    unknown: u8,
    packet_size: u16,
    game_name: [u8; 4],
    version: [u8; 3],
    build: u16,
    platform: [u8; 4],
    os: [u8; 4],
    locale: [u8; 4],
    timezone: u32,
    ip: [u8; 4],
    account_length: u8,
    #[depends_on(account_length)]
    account: String,
}

#[derive(LoginPacket, Serialize, Deserialize, Debug)]
struct Outgoing {
    unknown: u8,
    code: u8,
    server_ephemeral: Vec<u8>,
    g_len: u8,
    g: Vec<u8>,
    n_len: u8,
    n: Vec<u8>,
    salt: [u8; 32],
    // added in wotlk ?
    version_challenge: [u8; 16],
    unknown2: u8,
}

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let (LoginChallengeIncoming { account, .. }, _) =
            LoginChallengeIncoming::from_binary(&input.data)?;
        let mut srp = input.srp.lock().await;
        srp.set_account(account);
        srp.generate_verifier::<Sha1>();
        srp.generate_server_ephemeral();

        let (_, server_ephemeral) = srp.server_ephemeral.as_ref().unwrap().to_bytes_le();
        let (_, generator) = srp.generator.to_bytes_le();
        let (_, modulus) = srp.modulus.to_bytes_le();

        Ok(vec![HandlerOutput::Data(
            Outgoing {
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
            }
            .to_binary_with_opcode(Opcode::LOGIN_CHALLENGE)?,
        )])
    }
}
