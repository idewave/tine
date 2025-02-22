use std::cmp::PartialEq;
use std::sync::Arc;

use async_trait::async_trait;
use colored::Colorize;
use futures::future::join_all;
use tentacli_traits::types::opcodes::Opcode;
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpListener;
use tokio::signal;
use tokio::sync::{mpsc, Mutex};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;

use crate::primary::crypto::header_crypt::{HeaderDecryptor, HeaderEncryptor};
use crate::primary::crypto::srp::Srp;
use crate::primary::debug;
use crate::primary::server::types::Packet;
use crate::primary::types::{HandlerInput, HandlerOutput, ProcessorFunction, ProcessorResult};

#[derive(Default, Eq, PartialEq)]
pub enum RelayFlag {
    #[default]
    None,
    Send,
    Receive,
}

#[derive(Default)]
pub struct RunOptions {
    pub srp: Arc<Mutex<Srp>>,
    pub port: u16,
    pub world_port: u16,
    pub relay: RelayFlag,
}

#[async_trait]
pub trait BaseServer: Send {
    async fn start(
        options: RunOptions,
        relay_sender: Sender<(u32, Vec<u8>)>,
        relay_receiver: Arc<Mutex<Receiver<(u32, Vec<u8>)>>>,
    ) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", options.port)).await?;
        debug!(
            "[{}] Started on port {}",
            Self::server_name().green(),
            options.port
        );

        let mut connections = vec![];

        loop {
            tokio::select! {
                Ok((socket, _)) = listener.accept() => {
                    let peer_addr = socket.peer_addr().unwrap();
                    {
                        let message = format!(
                            "[{}] Client connected: {:?}",
                            Self::server_name().green(),
                            peer_addr
                        );
                        debug!("{}", message.bright_yellow());
                    }

                    let (input_sender, input_receiver) = mpsc::channel::<Packet>(100);
                    let (output_sender, output_receiver) = mpsc::channel::<(u32, Vec<u8>)>(100);
                    let (rx, tx) = socket.into_split();

                    let reader = BufReader::new(rx);
                    let writer = Arc::new(Mutex::new(tx));

                    connections.push(writer.clone());

                    let encryptor: Arc<Mutex<Option<HeaderEncryptor>>> = Arc::new(Mutex::new(None));
                    let decryptor: Arc<Mutex<Option<HeaderDecryptor>>> = Arc::new(Mutex::new(None));

                    let mut tasks = vec![
                        Self::handle_setup(
                            writer.clone()
                        ),
                        Self::handle_read(
                            input_sender,
                            output_sender.clone(),
                            reader,
                            decryptor.clone()
                        ),
                        Self::handle_input(
                            input_receiver,
                            output_sender.clone(),
                            options.srp.clone(),
                            encryptor.clone(),
                            decryptor.clone(),
                            options.world_port,
                        ),
                        Self::handle_write(
                            output_receiver,
                            relay_sender.clone(),
                            writer.clone(),
                            encryptor.clone(),
                            options.relay == RelayFlag::Send,
                        ),
                    ];

                    if options.relay == RelayFlag::Receive {
                        tasks.push(
                            Self::handle_relay_accept(
                                relay_receiver.clone(),
                                output_sender.clone(),
                            )
                        );
                    }

                    tokio::spawn(async move {
                        join_all(tasks).await;
                    });
                },
                _ = signal::ctrl_c() => {
                    for writer in connections.into_iter() {
                        writer.lock().await.shutdown().await?;
                    }

                    break;
                },
            }
        }

        Ok(())
    }

    fn handle_setup(writer: Arc<Mutex<OwnedWriteHalf>>) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move { Self::setup(writer.clone()).await })
    }

    fn handle_read(
        input_sender: Sender<Packet>,
        output_sender: Sender<(u32, Vec<u8>)>,
        reader: BufReader<OwnedReadHalf>,
        decryptor: Arc<Mutex<Option<HeaderDecryptor>>>,
    ) -> JoinHandle<anyhow::Result<()>>;

    fn handle_input(
        mut input_receiver: Receiver<Packet>,
        output_sender: Sender<(u32, Vec<u8>)>,
        srp: Arc<Mutex<Srp>>,
        encryptor: Arc<Mutex<Option<HeaderEncryptor>>>,
        decryptor: Arc<Mutex<Option<HeaderDecryptor>>>,
        world_port: u16,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            while let Some(packet) = input_receiver.recv().await {
                let Packet { body: data, opcode } = packet;
                debug!(
                    "[{}] [RECV]: {}",
                    Self::server_name().green(),
                    Opcode::get_opcode_name(opcode).unwrap().blue(),
                );

                let mut input = HandlerInput {
                    data,
                    opcode,
                    srp: Arc::clone(&srp),
                    world_port,
                };

                let handler_list = Self::get_processors()
                    .iter()
                    .flat_map(|processor| processor(&mut input))
                    .collect::<ProcessorResult>();

                for mut handler in handler_list {
                    let outputs = handler.handle(&mut input).await?;
                    for output in outputs {
                        match output {
                            HandlerOutput::Data(opcode, packet) => {
                                debug!(
                                    "[{}] [SEND]: {}",
                                    Self::server_name().green(),
                                    Opcode::get_opcode_name(opcode as u32).unwrap().magenta(),
                                );
                                output_sender.send((opcode as u32, packet)).await?;
                            }
                            HandlerOutput::HeaderCrypt(enc, dec) => {
                                *encryptor.lock().await = Some(enc);
                                *decryptor.lock().await = Some(dec);
                            }
                            _ => {}
                        }
                    }
                }
            }

            Ok(())
        })
    }

    fn handle_write(
        mut output_receiver: Receiver<(u32, Vec<u8>)>,
        relay_sender: Sender<(u32, Vec<u8>)>,
        writer: Arc<Mutex<OwnedWriteHalf>>,
        encryptor: Arc<Mutex<Option<HeaderEncryptor>>>,
        relay: bool,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            while let Some((opcode, mut packet)) = output_receiver.recv().await {
                if relay {
                    relay_sender.send((opcode, packet)).await?;
                } else {
                    if let Some(header_crypt) = encryptor.lock().await.as_mut() {
                        let is_large_packet = packet[0] >= 0x80;
                        let header_size: usize = if is_large_packet { 5 } else { 4 };
                        header_crypt.encrypt(&mut packet[..header_size])
                    }

                    writer.lock().await.write_all(&packet).await?;
                }
            }

            let peer_addr = writer.lock().await.peer_addr()?;
            let message = format!(
                "[{}] Client disconnected: {:?}",
                Self::server_name().green(),
                peer_addr
            );
            debug!("{}", message.yellow());

            Ok(())
        })
    }

    fn handle_relay_accept(
        relay_receiver: Arc<Mutex<Receiver<(u32, Vec<u8>)>>>,
        output_sender: Sender<(u32, Vec<u8>)>,
    ) -> JoinHandle<anyhow::Result<()>> {
        tokio::spawn(async move {
            while let Some((opcode, packet)) = relay_receiver.lock().await.recv().await {
                debug!(
                    "[{}] [SEND:RELAY]: {}",
                    Self::server_name().green(),
                    Opcode::get_opcode_name(opcode).unwrap().magenta(),
                );
                output_sender.send((opcode, packet)).await?;
            }

            Ok(())
        })
    }

    async fn setup(_writer: Arc<Mutex<OwnedWriteHalf>>) -> anyhow::Result<()> {
        Ok(())
    }

    fn get_processors() -> Vec<ProcessorFunction>;

    fn server_name<'a>() -> &'a str;
}
