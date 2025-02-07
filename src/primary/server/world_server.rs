use std::io::Cursor;
use std::sync::Arc;

use anyhow::Result as AnyResult;
use async_trait::async_trait;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::primary::server::{SERVER_HOST, WORLD_PORT};
use crate::primary::server::auth::auth_challenge;
use crate::primary::server::movement::MovementProcessor;
use crate::primary::server::player::PlayerProcessor;
use crate::primary::server::realm::RealmProcessor;
use crate::primary::server::types::Packet;
use crate::primary::traits::processor::Processor;
use crate::primary::traits::server::{Connection, RunOptions, Server};
use crate::primary::types::ProcessorFunction;

const HEADER_SIZE: usize = 6;
const OPCODE_SIZE: usize = 4;

pub struct WorldServer {}
#[async_trait]
impl Server for WorldServer {
    fn new() -> Self {
        Self {}
    }

    async fn read_packet(
        socket: &mut TcpStream,
        connection: Arc<Mutex<Connection>>,
    ) -> AnyResult<Packet> {
        let mut buffer = vec![0u8; HEADER_SIZE];
        socket.read_exact(&mut buffer).await?;

        let mut header_reader = {
            let mut guard = connection.lock().await;
            if let Some(header_crypt) = guard.header_crypt.as_mut() {
                Cursor::new(header_crypt.decrypt(&buffer))
            } else {
                Cursor::new(buffer)
            }
        };

        let size = ReadBytesExt::read_u16::<BigEndian>(&mut header_reader)? as usize;
        let opcode = ReadBytesExt::read_u32::<LittleEndian>(&mut header_reader)?;

        let mut body = vec![0u8; size - OPCODE_SIZE];
        socket.read_exact(&mut body).await?;

        Ok(Packet { opcode, data: body })
    }

    async fn init(&mut self, socket: &mut TcpStream, _: Arc<RunOptions>) {
        let packet = auth_challenge().await.unwrap();
        socket.write_all(&packet).await.unwrap();
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![
            Box::new(RealmProcessor::get_handlers),
            Box::new(PlayerProcessor::get_handlers),
            Box::new(MovementProcessor::get_handlers),
        ]
    }

    fn host<'a>() -> &'a str {
        SERVER_HOST
    }

    fn port() -> u16 {
        WORLD_PORT
    }

    fn server_name<'a>() -> &'a str {
        "World Server"
    }
}
