use std::io::Cursor;
use std::sync::Arc;

use async_trait::async_trait;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::primary::crypto::header_crypt::HeaderDecryptor;
use crate::primary::server::types::Packet;
use crate::primary::traits::server::BaseServer;
use crate::primary::types::ProcessorFunction;

const WHITELISTED_OPCODES: &[u16] = &[
    Opcode::SMSG_COMPRESSED_UPDATE_OBJECT,
    Opcode::SMSG_UPDATE_OBJECT,
    Opcode::SMSG_MONSTER_MOVE,
    Opcode::SMSG_MESSAGECHAT,
    Opcode::SMSG_EMOTE,
    Opcode::SMSG_TEXT_EMOTE,
    Opcode::MSG_MOVE_START_FORWARD,
    Opcode::MSG_MOVE_START_BACKWARD,
    Opcode::MSG_MOVE_JUMP,
    Opcode::MSG_MOVE_HEARTBEAT,
    Opcode::MSG_MOVE_START_TURN_LEFT,
    Opcode::MSG_MOVE_START_TURN_RIGHT,
    Opcode::MSG_MOVE_STOP,
    Opcode::MSG_MOVE_STOP_STRAFE,
    Opcode::MSG_MOVE_STOP_TURN,
    Opcode::MSG_MOVE_START_PITCH_UP,
    Opcode::MSG_MOVE_START_PITCH_DOWN,
    Opcode::MSG_MOVE_STOP_PITCH,
    Opcode::MSG_MOVE_FALL_LAND,
    Opcode::MSG_MOVE_SET_PITCH,
    Opcode::MSG_MOVE_START_SWIM,
    Opcode::MSG_MOVE_STOP_SWIM,
    Opcode::MSG_MOVE_SET_FACING,
    Opcode::SMSG_MONSTER_MOVE,
    Opcode::SMSG_MONSTER_MOVE_TRANSPORT,
    Opcode::SMSG_GM_MESSAGECHAT,
    Opcode::SMSG_SPELL_GO,
    Opcode::SMSG_SPELL_START,
    Opcode::SMSG_WEATHER,
    Opcode::SMSG_SERVERTIME,
    Opcode::SMSG_NAME_QUERY_RESPONSE,
    Opcode::SMSG_PET_NAME_QUERY_RESPONSE,
    Opcode::SMSG_ITEM_NAME_QUERY_RESPONSE,
    Opcode::MSG_RANDOM_ROLL,
    Opcode::SMSG_AURA_UPDATE_ALL,
    Opcode::SMSG_DESTROY_OBJECT,
    Opcode::SMSG_ITEM_TEXT_QUERY_RESPONSE,
];

pub struct RelayServer;

#[async_trait]
impl BaseServer for RelayServer {
    fn handle_read(
        _: Sender<Packet>,
        output_sender: Sender<(u32, Vec<u8>)>,
        mut reader: BufReader<OwnedReadHalf>,
        _: Arc<Mutex<Option<HeaderDecryptor>>>,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            loop {
                let mut packet = vec![0u8; 4];
                reader.read_exact(&mut packet).await?;

                let is_long_packet = packet[0] >= 0x80;
                if is_long_packet {
                    packet.push(reader.read_u8().await?);
                }

                let mut header_reader = Cursor::new(&packet);
                let size = if is_long_packet {
                    ReadBytesExt::read_u24::<BigEndian>(&mut header_reader)? as usize
                } else {
                    ReadBytesExt::read_u16::<BigEndian>(&mut header_reader)? as usize
                };

                let opcode = ReadBytesExt::read_u16::<LittleEndian>(&mut header_reader)?;

                let mut body = vec![0u8; size - 2];
                reader.read_exact(&mut body).await?;

                packet.extend_from_slice(&body);

                if WHITELISTED_OPCODES.contains(&opcode) {
                    // we send the data directly to handle_write
                    output_sender.send((opcode as u32, packet.clone())).await?;
                }
            }
        })
    }

    fn get_processors() -> Vec<ProcessorFunction> {
        vec![]
    }

    fn server_name<'a>() -> &'a str {
        "Relay Server"
    }
}
