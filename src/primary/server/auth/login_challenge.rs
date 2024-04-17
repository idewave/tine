use std::io::BufRead;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use tentacli_packet::LoginPacket;
use tentacli_traits::types::custom_fields::TerminatedString;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncBufRead, AsyncReadExt};

use crate::primary::traits::packet_handler::PacketHandler;
use crate::primary::types::{HandlerInput, HandlerOutput, HandlerResult};

const VERSION_CHALLENGE: [u8; 16] = [
    0xBA, 0xA3, 0x1E, 0x99, 0xA0, 0x0B, 0x21, 0x57,
    0xFC, 0x37, 0x3F, 0xB3, 0x69, 0xCD, 0xD2, 0xF1
];

#[derive(LoginPacket, Serialize, Deserialize, Debug)]
#[options(with_async)]
pub struct LoginChallengeIncome {
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
    #[dynamic_field]
    account: String,
}

impl LoginChallengeIncome {
    fn locale<R: BufRead>(mut reader: R, _: &mut Self) -> String {
        let mut buffer = vec![0u8; 4];
        reader.read_exact(&mut buffer).unwrap();
        buffer.reverse();
        String::from_utf8(buffer).unwrap()
    }

    async fn async_locale<R>(mut reader: R, _: &mut Self) -> String
        where R: AsyncBufRead + Unpin + Send
    {
        let mut buffer = vec![0u8; 4];
        reader.read_exact(&mut buffer).await.unwrap();
        buffer.reverse();
        String::from_utf8(buffer).unwrap()
    }

    fn account<R: BufRead>(mut reader: R, cache: &mut Self) -> String {
        let mut buffer = vec![0u8; cache.account_length as usize];
        reader.read_exact(&mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }

    async fn async_account<R>(mut reader: R, cache: &mut Self) -> String
        where R: AsyncBufRead + Unpin + Send
    {
        let mut buffer = vec![0u8; cache.account_length as usize];
        reader.read_exact(&mut buffer).await.unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

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

pub struct Handler;
#[async_trait]
impl PacketHandler for Handler {
    async fn handle(&mut self, input: &mut HandlerInput) -> HandlerResult {
        let mut response = Vec::new();

        let (LoginChallengeIncome { account, .. }, _) = LoginChallengeIncome::from_binary(&input.data)?;
        println!("ACC: {}", account);
        let mut srp = input.srp.lock().await;
        srp.set_account(account);
        srp.generate_verifier::<Sha1>();
        srp.generate_server_ephemeral::<Sha1>();

        let (_, server_ephemeral) = srp.server_ephemeral.as_ref().unwrap().to_bytes_le();
        let (_, generator) = srp.generator.to_bytes_le();
        let (_, modulus) = srp.modulus.to_bytes_le();

        response.push(HandlerOutput::Data(Outcome {
            unknown: 0,
            code: 0,
            server_ephemeral: server_ephemeral.into(),
            g_len: 1,
            g: generator.into(),
            n_len: 32,
            n: modulus.into(),
            salt: srp.salt,
            version_challenge: VERSION_CHALLENGE,
            unknown2: 0,
        }.to_binary_with_opcode(Opcode::LOGIN_CHALLENGE)?));

        Ok(response)
    }
}