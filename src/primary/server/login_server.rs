use std::sync::Arc;

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::primary::server::auth::{
    AuthProcessor, LoginChallengeIncoming, LoginProofIncoming, RealmlistIncoming,
};
use crate::primary::server::types::Packet;
use crate::primary::traits::base_server::{BaseServer, Connection};
use crate::primary::traits::processor::Processor;
use crate::primary::types::ProcessorFunction;
use crate::RunOptions;

pub struct LoginServer {}
impl LoginServer {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl BaseServer for LoginServer {
    async fn read_packet(socket: &mut TcpStream, _: Arc<Mutex<Connection>>) -> AnyResult<Packet> {
        let opcode = socket.read_u8().await?;
        let mut reader = BufReader::new(socket);

        let data = match opcode {
            Opcode::LOGIN_CHALLENGE => LoginChallengeIncoming::from_stream(&mut reader).await?,
            Opcode::LOGIN_PROOF => LoginProofIncoming::from_stream(&mut reader).await?,
            Opcode::REALM_LIST => RealmlistIncoming::from_stream(&mut reader).await?,
            _ => vec![],
        };

        Ok(Packet {
            opcode: opcode as u32,
            data,
        })
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![Box::new(AuthProcessor::get_handlers)]
    }

    fn server_name<'a>() -> &'a str {
        "Login Server"
    }

    fn port(options: Arc<RunOptions>) -> u16 {
        options.login_port
    }
}
