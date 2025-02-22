use std::io::Cursor;
use std::sync::Arc;

use async_trait::async_trait;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use serde::Serialize;
use tentacli_packet::WorldPacket;
use tentacli_traits::types::chat::{Language, MessageType};
use tentacli_traits::types::movement::ObjectUpdateFlags;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use crate::primary::crypto::header_crypt::HeaderDecryptor;
use crate::primary::server::player::UpdatePacket;
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
    Opcode::MSG_MOVE_START_STRAFE_RIGHT,
    Opcode::MSG_MOVE_START_STRAFE_LEFT,
    Opcode::MSG_MOVE_START_ASCEND,
    Opcode::MSG_MOVE_STOP_ASCEND,
    Opcode::MSG_MOVE_HOVER,
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
    Opcode::SMSG_TIME_SYNC_REQ,
    Opcode::SMSG_NAME_QUERY_RESPONSE,
    Opcode::SMSG_PET_NAME_QUERY_RESPONSE,
    Opcode::SMSG_ITEM_NAME_QUERY_RESPONSE,
    Opcode::MSG_RANDOM_ROLL,
    Opcode::SMSG_AURA_UPDATE_ALL,
    Opcode::SMSG_AURA_UPDATE,
    Opcode::SMSG_DESTROY_OBJECT,
    Opcode::SMSG_ITEM_TEXT_QUERY_RESPONSE,
    // FOR TESTING
    Opcode::SMSG_POWER_UPDATE,
    Opcode::SMSG_SPELLLOGEXECUTE,
    Opcode::SMSG_INIT_WORLD_STATES,
    Opcode::SMSG_LEARNED_DANCE_MOVES,
    Opcode::SMSG_CHANNEL_NOTIFY,
    Opcode::SMSG_ATTACKSTART,
    Opcode::SMSG_ATTACKSTOP,
    Opcode::SMSG_FORCE_MOVE_ROOT,
    Opcode::SMSG_FORCE_MOVE_UNROOT,
    Opcode::SMSG_MOVE_LAND_WALK,
];

pub struct RelayServer;

impl RelayServer {
    fn transform(opcode: u16, packet: &mut [u8]) -> anyhow::Result<Vec<u8>> {
        // TODO: later adapt this as a pattern
        match opcode {
            Opcode::SMSG_UPDATE_OBJECT | Opcode::SMSG_COMPRESSED_UPDATE_OBJECT => {
                let (
                    UpdatePacket {
                        mut blocks,
                        blocks_amount,
                    },
                    _,
                ) = {
                    if opcode == Opcode::SMSG_UPDATE_OBJECT {
                        UpdatePacket::from_binary(&packet[4..])?
                    } else {
                        UpdatePacket::from_compressed_binary(&packet[4..])?
                    }
                };

                for block in blocks.iter_mut() {
                    block
                        .movement
                        .object_update_flags
                        .set(ObjectUpdateFlags::SELF, false);
                }

                Ok(UpdatePacket {
                    blocks,
                    blocks_amount,
                }
                .to_binary_with_server_opcode(Opcode::SMSG_UPDATE_OBJECT)?)
            }
            Opcode::SMSG_MESSAGECHAT => {
                // overriding chat messages to understand each language
                #[derive(WorldPacket, Serialize, Debug)]
                struct Incoming {
                    message_type: u8,
                    language: u32,
                    sender_guid: u64,
                    skip: u32,
                    #[conditional]
                    channel_name: String,
                    target_guid: u64,
                    message_length: u32,
                    #[depends_on(message_length)]
                    message: String,
                    unknown: u16,
                }

                impl Incoming {
                    fn channel_name(instance: &mut Self) -> bool {
                        instance.message_type == MessageType::CHANNEL
                    }
                }

                let (fields, _) = Incoming::from_binary(&packet[4..])?;

                Ok(Incoming {
                    language: Language::UNIVERSAL,
                    ..fields
                }
                .to_binary_with_server_opcode(Opcode::SMSG_MESSAGECHAT)?)
            }
            _ => Ok(packet.to_vec()),
        }
    }
}

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
                    let packet = Self::transform(opcode, &mut packet)?;
                    // we send the data directly to handle_write
                    output_sender.send((opcode as u32, packet)).await?;
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
