use std::sync::Arc;

use async_trait::async_trait;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::primary::crypto::header_crypt::HeaderDecryptor;
use crate::primary::server::auth::{
    AuthProcessor, LoginChallengeIncoming, LoginProofIncoming, RealmlistIncoming,
};
use crate::primary::server::types::Packet;
use crate::primary::traits::processor::Processor;
use crate::primary::traits::server::BaseServer;
use crate::primary::types::ProcessorFunction;

#[derive(Default)]
pub struct LoginServer;

#[async_trait]
impl BaseServer for LoginServer {
    fn handle_read(
        input_sender: Sender<Packet>,
        mut reader: BufReader<OwnedReadHalf>,
        _: Arc<Mutex<Option<HeaderDecryptor>>>,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            loop {
                let opcode = reader.read_u8().await?;

                let body = match opcode {
                    Opcode::LOGIN_CHALLENGE => {
                        LoginChallengeIncoming::from_stream(&mut reader).await?
                    }
                    Opcode::LOGIN_PROOF => LoginProofIncoming::from_stream(&mut reader).await?,
                    Opcode::REALM_LIST => RealmlistIncoming::from_stream(&mut reader).await?,
                    _ => vec![],
                };

                if !body.is_empty() {
                    input_sender.send(Packet { opcode: opcode as u32, body }).await?;
                }
            }
        })
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![Box::new(AuthProcessor::get_handlers)]
    }

    fn server_name<'a>() -> &'a str {
        "Login Server"
    }
}
