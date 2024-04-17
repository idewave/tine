use std::sync::{Arc};
use anyhow::{Result as AnyResult};
use async_trait::async_trait;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::primary::server::auth::{AuthProcessor, LoginChallengeIncome, LoginProofIncome, RealmlistIncome};
use crate::primary::server::{SERVER_HOST, LOGIN_PORT};
use crate::primary::server::types::Packet;
use crate::primary::traits::processor::Processor;
use crate::primary::traits::server::{Connection, Server};
use crate::primary::types::{ProcessorFunction};

pub struct LoginServer {}

#[async_trait]
impl Server for LoginServer {
    fn new() -> Self {
        Self {}
    }

    async fn read_packet(
        socket: &mut TcpStream,
        _: Arc<Mutex<Connection>>
    ) -> AnyResult<Packet> {
        let opcode = socket.read_u8().await?;
        let mut reader = BufReader::new(socket);

        let data = match opcode {
            Opcode::LOGIN_CHALLENGE => LoginChallengeIncome::from_stream(&mut reader).await?,
            Opcode::LOGIN_PROOF => LoginProofIncome::from_stream(&mut reader).await?,
            Opcode::REALM_LIST => RealmlistIncome::from_stream(&mut reader).await?,
            _ => vec![],
        };

        println!("OPCODE: {} has been read", opcode);

        Ok(Packet { opcode: opcode as u32, data })
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![Box::new(AuthProcessor::get_handlers)]
    }

    fn host<'a>() -> &'a str {
        SERVER_HOST
    }

    fn port() -> u16 {
        LOGIN_PORT
    }

    fn server_name<'a>() -> &'a str {
        "Login Server"
    }
}