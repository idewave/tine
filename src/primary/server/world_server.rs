use std::io::Cursor;
use std::sync::Arc;

use async_trait::async_trait;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::primary::crypto::header_crypt::HeaderDecryptor;
use crate::primary::server::auth::auth_challenge;
use crate::primary::server::player::PlayerProcessor;
use crate::primary::server::realm::RealmProcessor;
use crate::primary::server::types::Packet;
use crate::primary::traits::processor::Processor;
use crate::primary::traits::server::BaseServer;
use crate::primary::types::ProcessorFunction;

const HEADER_SIZE: usize = 6;
const OPCODE_SIZE: usize = 4;

#[derive(Default)]
pub struct WorldServer;

#[async_trait]
impl BaseServer for WorldServer {
    fn handle_read(
        input_sender: Sender<Packet>,
        mut reader: BufReader<OwnedReadHalf>,
        decryptor: Arc<Mutex<Option<HeaderDecryptor>>>,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            loop {
                let mut buffer = vec![0u8; HEADER_SIZE];
                reader.read_exact(&mut buffer).await?;

                let mut header_reader = {
                    if let Some(header_crypt) = decryptor.lock().await.as_mut() {
                        Cursor::new(header_crypt.decrypt(&buffer))
                    } else {
                        Cursor::new(buffer)
                    }
                };

                let size = ReadBytesExt::read_u16::<BigEndian>(&mut header_reader)? as usize;
                let opcode = ReadBytesExt::read_u32::<LittleEndian>(&mut header_reader)?;

                let mut body = vec![0u8; size - OPCODE_SIZE];
                reader.read_exact(&mut body).await?;

                input_sender.send(Packet { opcode, body }).await?;
            }
        })
    }

    async fn setup(writer: Arc<Mutex<OwnedWriteHalf>>) -> anyhow::Result<()> {
        let packet = auth_challenge().await?;
        writer.lock().await.write_all(&packet).await?;

        Ok(())
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![
            Box::new(RealmProcessor::get_handlers),
            Box::new(PlayerProcessor::get_handlers),
        ]
    }

    fn server_name<'a>() -> &'a str {
        "World Server"
    }
}
